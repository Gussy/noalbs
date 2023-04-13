mod api_v3;

use api_v3::{Progress, RestreamerAPI};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

use super::{Bsl, StreamServersCommands, SwitchLogic};
use crate::switcher::{SwitchType, Triggers};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Restreamer {
    /// Restreamer base url
    pub base_url: String,

    /// Restreamer username
    pub username: String,

    /// Restreamer password
    pub password: String,

    /// ID of the restreamer channel to monitor
    pub channel: String,

    /// Client to make HTTP requests with
    #[serde(skip)]
    pub client: reqwest::Client,
}

impl Restreamer {
    pub async fn get_stats(&self) -> Option<Progress> {
        let mut api = RestreamerAPI::new(&self.base_url);
        match api
            .login(self.username.as_str(), self.password.as_str())
            .await
        {
            Ok(_) => (),
            Err(_) => {
                error!("unable to login");
                return None;
            }
        }

        let process = match api.v3_process_get(&self.channel, "state").await {
            Ok(p) => p,
            Err(e) => {
                error!("unable to get process: {}", e);
                return None;
            }
        };

        let progress = process.state.unwrap().progress;
        trace!("{:#?}", progress);

        Some(progress)
    }
}

#[async_trait]
#[typetag::serde]
impl SwitchLogic for Restreamer {
    /// Which scene to switch to
    async fn switch(&self, triggers: &Triggers) -> SwitchType {
        let stats = match self.get_stats().await {
            Some(b) => b,
            None => return SwitchType::Offline,
        };

        if let Some(offline) = triggers.offline {
            if stats.bitrate_kbit > 0.0 && stats.bitrate_kbit <= offline.into() {
                return SwitchType::Offline;
            }
        }

        if let Some(low) = triggers.low {
            if stats.bitrate_kbit <= low.into() {
                return SwitchType::Low;
            }
        }

        if stats.bitrate_kbit == 0.0 {
            return SwitchType::Previous;
        }

        return SwitchType::Normal;
    }
}

#[async_trait]
#[typetag::serde]
impl StreamServersCommands for Restreamer {
    async fn bitrate(&self) -> super::Bitrate {
        let stats = match self.get_stats().await {
            Some(stats) => stats,
            None => return super::Bitrate { message: None },
        };

        if stats.bitrate_kbit == 0.0 {
            return super::Bitrate { message: None };
        }

        let message = format!("{}", stats.bitrate_kbit as u64);
        super::Bitrate {
            message: Some(message),
        }
    }

    async fn source_info(&self) -> Option<String> {
        let stats = self.get_stats().await?;

        let bitrate = format!("{} Kbps", stats.bitrate_kbit);
        let dropped = format!("dropped {} packets", stats.drop);

        Some(format!("{} | {}", bitrate, dropped))
    }
}

#[typetag::serde]
impl Bsl for Restreamer {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
