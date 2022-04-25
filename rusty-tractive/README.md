# 🐈 `rusty-tractive`

Integrates [Tractive](https://tractive.com/).

## Streams

### `rusty:tractive:<tracker_id>:hardware`

| key       | type    | value                                          |
|-----------|---------|------------------------------------------------|
| `ts`      | integer | Timestamp as received from Tractive, unix time |
| `battery` | integer | Battery level, percentage                      |

### `rusty:tractive:<tracker_id>:position`

| key        | type    | value            |
|------------|---------|------------------|
| `ts`       | integer | Unix time        |
| `lat`      | float   | Latitude         |
| `lon`      | float   | Longitude        |
| `accuracy` | integer |                  |
| `course`   | integer | Heading, degrees |

## Heartbeat

The heartbeat is being sent every time the microservice receives a message from Tractive, and normally Tractive sends a keep-alive message every **5 seconds**. However, the keep-alive TTL is **5 minutes** as reported by Tractive.

### [Better Uptime](https://betteruptime.com/)

| Expect a heartbeat every | with a grace period of  |
|--------------------------|-------------------------|
| 30 seconds               | 5 minutes               |
