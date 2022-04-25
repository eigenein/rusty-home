# 🐈 `rusty-tractive`

Connects Rusty Home to [Tractive](https://tractive.com/).

## Heartbeat

The heartbeat is being sent every time the microservice receives a message from Tractive, and normally Tractive sends a keep-alive message every **5 seconds**. However, the keep-alive TTL is **5 minutes** as reported by Tractive.

## [Better Uptime](https://betteruptime.com/)

| Expect a heartbeat every | with a grace period of  |
|--------------------------|-------------------------|
| 30 seconds               | 5 minutes               |
