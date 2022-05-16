# 🐈 `rusty-tractive`

Integrates [Tractive](https://tractive.com/) and streams the pet location and hardware status.

## Streams

### `rusty:tractive:<tracker_id>:hardware`

| key       | type    | value                                          |
|-----------|---------|------------------------------------------------|
| `ts`      | integer | Timestamp as received from Tractive, unix time |
| `battery` | integer | Battery level, percentage                      |

### `rusty:tractive:<tracker_id>:position`

| key        | type              | value                   |
|------------|-------------------|-------------------------|
| `ts`       | integer           | Unix time               |
| `lat`      | float             | Latitude                |
| `lon`      | float             | Longitude               |
| `accuracy` | integer           |                         |
| `course`   | integer, optional | Heading, degrees        |

## 💓 Heartbeat

The heartbeat is expected every time the tracker status gets updated.

| Expect a heartbeat every | with a grace period of |
|--------------------------|------------------------|
| 1 hour                   | 10 minutes             |
