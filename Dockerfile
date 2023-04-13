FROM rust:latest as builder

RUN USER=root cargo new --bin noalbs
WORKDIR /noalbs
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD ./src ./src

RUN rm ./target/release/deps/noalbs*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

# EXPOSE ports here when needed

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /noalbs/target/release/noalbs ${APP}/noalbs

COPY docker-entrypoint.sh ${APP}
RUN chmod +x ${APP}/docker-entrypoint.sh

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER

WORKDIR ${APP}

VOLUME ["/app/config"]

ENTRYPOINT ["/app/docker-entrypoint.sh"]
