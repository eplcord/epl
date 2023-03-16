# epl
A [Litecord](https://gitlab.com/litecord/litecord) compatible/inspired OSS implementation of Discord's backend for fun and profit.

## Why?
~~Like I said, fun and profit!~~

In all seriousness, this project is intended as a way to understand the Discord API in a more familiar stack to help speed up implementing new features and fixing bugs for Litecord.

As such, the Epl project has the same goals as the original Litecord project.

Being Litecord compatible also means we will be using Litecord's [LVSP protocol](https://gitlab.com/litecord/litecord/-/blob/master/docs/lvsp.md), [mediaproxy](https://gitlab.com/litecord/mediaproxy), and [admin API](https://gitlab.com/litecord/litecord/-/blob/master/docs/admin_api.md).

## Status

Big ol tables of features and their implementation status.

### v1
(To be filled out)

| Feature | Status | Notes |
|---------|--------|-------|
|         |        |       |

### v3
(To be filled out)

| Feature | Status | Notes |
|---------|--------|-------|
|         |        |       |

### v6
(To be filled out)

| Feature | Status | Notes |
|---------|--------|-------|
|         |        |       |

### v9
(To be filled out)

| Feature | Status | Notes |
|---------|--------|-------|
|         |        |       |

## Installation/Running
### Requirements
#### Runtime
Required:
 * Some sort of compatible libc

Optional:
 * [mediaproxy](https://gitlab.com/litecord/mediaproxy)
 * [An LVSP Server](https://git.gaycatgirl.sex/litecord/bannana-pho)

#### Development
Required:
 * Rust (1.56+)
 * [libpq](https://www.postgresql.org/docs/current/libpq.html)
 * [diesel cli](https://diesel.rs) (`cargo install diesel_cli`)

Optional:
 * [mediaproxy](https://gitlab.com/litecord/mediaproxy)
 * [An LVSP Server](https://git.gaycatgirl.sex/litecord/bannana-pho)

## Configuration

|       Variable        |                                     Description                                      |                      Example                      | Required? |         Default          |
|:---------------------:|:------------------------------------------------------------------------------------:|:-------------------------------------------------:|:---------:|:------------------------:|
|      `RUST_LOG`       | Rust logging level (See [env_logger](https://docs.rs/env_logger/latest/env_logger/)) |                      `INFO`                       |           |         `ERROR`          |
|        `NAME`         |                                 The instance's name                                  |                       `Epl`                       |    [x]    |           N/A            |
|         `URL`         |                               The instance's main URL                                |                     `epl.dev`                     |    [x]    |           N/A            |
|     `GATEWAY_URL`     |                The URL of the Gateway (Must be accessible to clients)                |                 `gateway.epl.dev`                 |    [x]    |           N/A            |
|   `MEDIAPROXY_URL`    |                          The URL of the Litecord mediaproxy                          |                  `media.epl.dev`                  |           |           N/A            |
|  `HTTP_LISTEN_ADDR`   |                            Listen address of the HTTP API                            |                  `0.0.0.0:3926`                   |           |      `0.0.0.0:3926`      |
| `GATEWAY_LISTEN_ADDR` |                         Listen address of the gateway socket                         |                  `0.0.0.0:5001`                   |           |      `0.0.0.0:5001`      |
|    `DATABASE_URL`     |                               PostgreSQL database URL                                | `postgres://username:password@localhost/database` |    [x]    |           N/A            |
|     `REDIS_ADDR`      |                                  Redis database URL                                  |             `redis://127.0.0.1:6379`              |           | `redis://127.0.0.1:6379` |
|     `LVSP_SECRET`     |      LVSP Shared Secret, can be anything (Must be the same on the LVSP server)       |                   `supersecret`                   |    [x]    |           N/A            |
|     `REQUIRE_SSL`     |             Whether or not SSL protocols will be used (wss:// https://)              |                       true                        |           |          false           |
|    `REGISTRATION`     |                        Whether or not registration is allowed                        |                       true                        |           |          false           |

## Contributing
Please read the [contributing guide](https://git.gaycatgirl.sex/litecord/epl/src/branch/main/CONTRIBUTING.md) and the [code of conduct](https://git.gaycatgirl.sex/litecord/epl/src/branch/main/CODE_OF_CONDUCT.md).

Especially the commit message style guidelines.