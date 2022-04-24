# 🐈 `rusty-tractive`

Connects Rusty Home to [Tractive](https://tractive.com/).

## Heartbeat

The heartbeat is being sent every time the microservice receives a message from Tractive, and that is normally every 5 seconds. However, the keep-alive TTL is 5 minutes as reported by Tractive, which means that in theory it may be up to 5 minutes.
