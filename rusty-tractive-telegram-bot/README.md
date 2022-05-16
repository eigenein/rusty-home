# 🐈📲 `rusty-tractive-telegram`

Connects [Tractive](https://tractive.com) to [Telegram](https://core.telegram.org/bots/api).

## Features

- [x] Maintains a pinned [live location](https://telegram.org/blog/live-locations) in the Telegram chat
- [x] Sends out battery notifications (charged, low and critical) with customizable levels and texts

## 💓 Heartbeat

The heartbeat is expected every time the tracker position gets updated.

| Expect a heartbeat every | with a grace period of |
|--------------------------|------------------------|
| 1 hour                   | 10 minutes             |
