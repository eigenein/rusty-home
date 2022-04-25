# 🐈 `rusty-tractive`

Connects Rusty Home to [Tractive](https://tractive.com/).

## Heartbeat

The heartbeat is being sent every time the microservice receives a message from Tractive, but no more than once a minute. Normally Tractive sends a keep-alive message every **5 seconds**. However, the keep-alive TTL is **5 minutes** as reported by Tractive. To summarize, the heartbeat is **expected every 1-5 minutes**.

## [Better Uptime](https://betteruptime.com/)

| Expect a heartbeat every | with a grace period of  |
|--------------------------|-------------------------|
| 1 minute                 | 5 minutes               |
