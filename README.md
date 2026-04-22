[![MotorbotHeader](https://github.com/motorlatitude/MotorBot/blob/main/assets/header.png?raw=true)]()

[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![Docker](https://badgen.net/badge/icon/docker?icon=docker&label)](https://github.com/motorlatitude/MotorBot/pkgs/container/motorbot)
[![GitHub release](https://img.shields.io/github/release/motorlatitude/motorbot)](https://GitHub.com/motorlatitude/MotorBot/releases/)
[![GitHub issues](https://img.shields.io/github/issues/motorlatitude/MotorBot.svg)](https://GitHub.com/motorlatitude/MotorBot/issues/)

MotorBot is a simple Discord Bot written using Rust and the [Serenity](https://github.com/serenity-rs/serenity) library. It compiles into a Docker Container and can be deployed to any Docker Host. It is currently in development and is not ready for production use.

## Features

- [x] Tell Jokes
- [x] Notify Game Patch Releases
- [x] Apply voting to messages and keep track of scores assigned to users
- [x] Ping pong
- [x] Heads or Tails
- [x] Roll Dice
- [x] Tell the time

## Deployment

MotorBot can be deployed as a Docker Container and can be run on any Docker Host. It is recommended to use [Docker Compose](https://docs.docker.com/compose/) for deployment.

```YAML
services:
    motorbot:
        name: motorbot
        image: ghcr.io/motorlatitude/motorbot:latest
        restart: unless-stopped
        environment:
            - DATABASE_PATH=${DATABASE_PATH:-/data/}
            - DISCORD_TOKEN=${DISCORD_TOKEN}
            - RAPID_API_KEY=${RAPID_API_KEY}
            - LOG_LEVEL=${LOG_LEVEL:-info}
            - TZ=${TZ:-UTC}
        volumes:
            - data:/data

volumes:
    data:
```

## Environment Variables

MotorBot requires there to be several environmental variables set.

#### `DISCORD_TOKEN` \[REQUIRED]

This should contain a Discord Bot token and can be registered [here](https://discord.com/developers/home).

#### `RAPID_API_KEY`

This should contain a Rapid API key and can be registered [here](https://rapidapi.com/). This is used for the joke plugin.

#### `DATABASE_PATH`

This should contain the path to the database file. The default is `/data/` and it will be created if it does not exist.

#### `LOG_LEVEL`

This should contain the log level for the application. The default is `info` and can be set to `debug`, `error`, `warn`, or `trace`.

#### `TZ`

This should contain the timezone for the application. The default is `UTC` and can be set to any valid timezone string (e.g. `America/New_York`).

## Build

The bot can be run locally using `cargo run` or using Docker.

To run on Docker, build the image using `docker build -t motorbot .` and run using `docker run -d -it --env-file .\.env motorbot`. The bot will automatically restart if it crashes.
