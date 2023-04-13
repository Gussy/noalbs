use std::{collections::HashMap, error::Error, fmt};

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MaxCleanup {
    max_file_age_seconds: i64,
    max_files: i64,
    pattern: String,
    purge_on_delete: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputOutput {
    address: String,
    cleanup: Option<Vec<MaxCleanup>>,
    id: String,
    options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvStream {
    aqueue: i64,
    drop: i64,
    dup: i64,
    duplicating: bool,
    enc: i64,
    gop: String,
    input: Input,
    looping: bool,
    output: Output,
    queue: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    packet: i64,
    size_kb: i64,
    state: String,
    time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    packet: i64,
    size_kb: i64,
    state: String,
    time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stream {
    address: String,
    avstream: Option<AvStream>,
    bitrate_kbit: f64,
    channels: Option<i64>,
    codec: String,
    coder: String,
    format: String,
    fps: f64,
    frame: u64,
    height: Option<u64>,
    id: String,
    index: u64,
    layout: Option<String>,
    packet: u64,
    pix_fmt: Option<String>,
    pps: f64,
    q: f64,
    sampling_hz: Option<f64>,
    size_kb: u64,
    stream: u64,
    type_field: Option<String>,
    width: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Progress {
    pub bitrate_kbit: f64,
    pub drop: u64,
    dup: u64,
    fps: f64,
    frame: u64,
    inputs: Vec<Stream>,
    outputs: Vec<Stream>,
    packet: u64,
    q: f64,
    size_kb: u64,
    speed: u64,
    time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Limits {
    cpu_usage: f64,
    memory_mbytes: f64,
    waitfor_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    autostart: bool,
    id: String,
    input: Vec<InputOutput>,
    limits: Limits,
    options: Vec<String>,
    output: Vec<InputOutput>,
    reconnect: bool,
    reconnect_delay_seconds: u64,
    reference: String,
    stale_timeout_seconds: u64,
    type_field: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    created_at: u64,
    log: Vec<Vec<String>>,
    prelude: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    created_at: u64,
    history: Vec<History>,
    log: Vec<Vec<String>>,
    prelude: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    command: Vec<String>,
    cpu_usage: f64,
    exec: String,
    last_logline: String,
    memory_bytes: u64,
    order: String,
    pub progress: Progress,
    reconnect_seconds: i64,
    runtime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetProcess {
    config: Option<Config>,
    created_at: Option<i64>,
    id: Option<String>,
    metadata: Option<String>,
    reference: Option<String>,
    report: Option<Report>,
    pub state: Option<State>,

    #[serde(rename = "type")]
    type_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetAPI {
    app: String,
    auths: Vec<String>,
    version: Version,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessCommand {
    command: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Token {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug)]
pub struct APIError(String);

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for APIError {}

pub struct RestreamerAPI {
    base_url: String,
    http_client: Client,
    token: Token,
}

impl RestreamerAPI {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http_client: Client::new(),
            token: Token::default(),
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
        let api_response: GetAPI = self
            .http_client
            .get(format!("{}/api", self.base_url))
            .send()
            .await?
            .json()
            .await?;

        if api_response.auths.contains(&"localjwt".to_owned()) {
            match self.basic_login(username, password).await {
                Ok(t) => self.token = t,
                Err(e) => return Err(e),
            };
        } else {
            return Err(Box::new(APIError("incorrect auth method".into())));
        }

        tracing::info!("got jwt tokens");

        Ok(())
    }

    async fn basic_login(&self, username: &str, password: &str) -> Result<Token, Box<dyn Error>> {
        let mut post_values = HashMap::new();
        post_values.insert("username", username);
        post_values.insert("password", password);

        let login_uri = format!("{}/api/login", self.base_url);

        let login_response = match self
            .http_client
            .post(login_uri)
            .json(&post_values)
            .send()
            .await
        {
            Ok(a) => a,
            Err(e) => {
                tracing::error!("auth error {}", e);
                return Err(Box::new(e));
            }
        };

        let token: Token = login_response.json().await?;

        Ok(token)
    }

    pub async fn v3_process_get(
        &mut self,
        id: &str,
        filter: &str,
    ) -> Result<GetProcess, Box<dyn Error>> {
        let response: GetProcess = self
            .http_client
            .get(format!(
                "{}/api/v3/process/{}?filter={}",
                self.base_url, id, filter
            ))
            .header(
                "authorization",
                format!("Bearer {}", self.token.access_token),
            )
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}
