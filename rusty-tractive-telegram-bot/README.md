# 🐈📲 `rusty-tractive-telegram-bot`

Connects [Tractive](https://tractive.com) to [Telegram](https://core.telegram.org/bots/api).

## Features

- [x] Maintains a pinned [live location](https://telegram.org/blog/live-locations) in the Telegram chat
- [x] Sends out battery notifications (charged, low and critical) with customizable levels and texts
- [ ] Unusual location notifications

## 💓 Heartbeat

The heartbeat is expected every time the tracker's position gets updated.

| Expect a heartbeat every | with a grace period of |
|--------------------------|------------------------|
| 2 hours                  | 10 minutes             |

## Health endpoint

You can also monitor `GET` `/health` for availability. It's served by the same web server as the Telegram update handler.
