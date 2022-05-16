[![Last commit](https://img.shields.io/github/last-commit/eigenein/rusty-home?logo=github)](https://github.com/eigenein/rusty-home/commits/master)
[![Build status](https://github.com/eigenein/rusty-home/actions/workflows/check.yaml/badge.svg)](https://github.com/eigenein/rusty-home/actions)

# 🏚 `rusty-home`

This is a self-educational pet project.

I'm attempting to make a set highly-available microservices that I could deploy onto 3x of mine Raspberry Pi Zero W. Each microservice is going ot focus on a particular task. The microservices would exchange messages with each other through [Redis streams](https://redis.io/docs/manual/data-types/streams/).

## Motivation

I'd be happy to automate some routines, but I wouldn't like to maintain a Home Assistant instance.

I have already maintained a similar project – [`eigenein/my-iot-rs`](https://github.com/eigenein/my-iot-rs) – which was monolithic. But it was quite time-consuming to maintain all the internal interfaces, yet to rebuild the entire binary after a minor change. Also, the only way to extend it was to modify the monolithic source code. In theory, the new design should allow extending it using any other stack – as soon as it's capable of reading and writing Redis streams.

## Design

Each host is running a `redis-server` and [`redis-sentinel`](https://redis.io/docs/manual/sentinel/) with enabled persistence and automatic failovers.

Each host is running a set of microservices – ideally, all of them – via `systemd`. The microservices should use [Redis consumer groups](https://redis.io/docs/manual/data-types/streams/#consumer-groups) to ensure reliable processing of messages.

- Errors and performance are monitored by [Sentry](https://sentry.io/)
- Liveness is monitored by [Better Uptime](https://betteruptime.com/), see also the «Heartbeat» sections in the `README`s
- Logs are handled by `journald`
- Configuration is synced by [Syncthing](https://syncthing.net/)

## Available microservices

- [ ] [tado°](rusty-tado)
- [x] [Tractive](rusty-tractive)
- [x] [Tractive Telegram Bot](rusty-tractive-telegram-bot)

## Installation

For Raspberry Zero W you can grab the binaries from the [releases](https://github.com/eigenein/rusty-home/releases).

```shell
# Choose which binaries you'd like to install.
cargo install --git https://github.com/eigenein/rusty-home.git --locked rusty-tractive rusty-tado
```
