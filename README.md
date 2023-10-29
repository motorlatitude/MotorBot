[![MotorbotHeader](https://github.com/motorlatitude/MotorBot/blob/main/assets/header.png?raw=true)]()

MotorBot is a simple Discord Bot written using Rust and the [serenity](https;//github.com/serenity-rs/serenity) library. It compiles into a Docker Container and can be deployed to any Docker Host. It is currently in development and is not ready for production use.

## Features
- [x] Tell Jokes
- [x] Notify Game Patch Releases
- [x] Apply voting to messages and keep track of scores assigned to users
- [x] Ping pong
- [x] Heads or Tails
- [x] Roll Dice
- [x] Tell the time

## Planned Features
- [ ] Web interface
- [ ] Admin interface
- [ ] Fishing (maybe)
- [ ] Play music (maybe)
- [ ] Game stats (maybe)
- [ ] Game server status (maybe)

## Usage
The bot requires a .env file to be present in the root directory of the project. This file should contain the following variables:
```env
DISCORD_TOKEN={your_discord_token}
MONGO_URL=mongodb+srv://{mongo_url}
RAPID_API_KEY={rapid_api_key}
OPENAI_API_KEY={openai_api_key}
```

Modify main.rs:55:64 as required for your own server and channels. The bot can be run locally using `cargo run` or using Docker.

To run on Docker, build the image using `docker build -t motorbot .` and run using `docker run -d -it --env-file .\.env motorbot`. The bot will automatically restart if it crashes.