# 🐈📲 `rusty-tractive-telegram-bot`

Connects [Tractive](https://tractive.com) to [Telegram](https://core.telegram.org/bots/api).

## Features

- [x] Maintains a pinned [live location](https://telegram.org/blog/live-locations) in the Telegram chat
- [x] Sends out battery notifications (charged, low and critical) with customizable levels and texts
- [ ] Unusual location notifications

## Redundancy

The bot uses the [long polling](https://core.telegram.org/bots/api#getupdates) to receive the updates, and the bot API allows only one active long polling request at a time. In order to work around this, the bot maintains a lock in Redis to ensure only one instance would try to call `getUpdates` at a same time. Thus, it's safe to run as many instances of the bot as you wish.

## 💓 Heartbeat

The heartbeat is expected every time the tracker position gets updated.

| Expect a heartbeat every | with a grace period of |
|--------------------------|------------------------|
| 2 hours                  | 10 minutes             |
