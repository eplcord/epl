# Running epl

## Overview
Epl is split up into multiple services, `epl-http` (the HTTP API service), `epl-gateway` (the gateway websocket service), 
and `epl-cdn` (the s3 proxy), this is expected to grow as the project increases in feature completeness. 
It also uses [Postgres](https://www.postgresql.org/) as the database, [NATS](https://nats.io/) as a messaging and KV 
platform, and an [S3 compatible server/provider](https://garagehq.deuxfleurs.fr/) for file storage.

![The basic epl topology as a diagram](assets/topology_diagram.png)

This guide assumes you have basic knowledge in administrating servers, creating services, configuring Postgres and NATS,
configuring a reverse proxy, and compiling Rust programs from source. No binaries are provided at this time due to the
constant additions of new features. Always be prepared to reconfigure services to reflect changes.

### Requirements
Required:
* [Rust](https://www.rust-lang.org/) (v1.75+)
* libwebp and headers
* A working [Postgres](https://www.postgresql.org/) server
* A working [NATS](https://nats.io/) instance
* Some sort of reverse proxy, development is done with [Caddy](https://caddyserver.com/)
* An S3 compatible server or provider, development is done with [Garage](https://garagehq.deuxfleurs.fr/)

Optional:
* [mediaproxy](https://gitlab.com/litecord/mediaproxy)
* [An LVSP Server](https://git.gaycatgirl.sex/litecord/bannana-pho)

#### Development
Required:
* [sea-orm cli](https://www.sea-ql.org/SeaORM/) (`cargo install sea-orm-cli`)

Please note that Windows is **NOT SUPPORTED** currently due to linking issues with libwebp.

## Compiling
Compiling Epl is pretty straight forward, you only need to install the Rust compiler and libwebp with headers, refer to
your distribution's documentation for this. Some common package names for libwebp are `libwebp-dev` and `libwebp-devel`.

You'll want to grab the latest source by cloning the main branch using Git (there are no releases yet), this helps you
keep updated with new changes and also allows you to easily redo/undo database migrations using the sea-orm cli if needed.

Once cloned, you'll be able to run `cargo build --release` to automatically compile all crates in one go, the compiled
binaries will be available in the `target/release` folder as `epl-cdn`, `epl-gateway`, and `epl-http`. You can safely
ignore any warnings that appear during compilation and all the other files in `target/release`.

## Prerequisites
Before actually running Epl, you'll want to configure all the other services now. Ensure you have a working Postgres
database, NATS instance, and an S3 server with a bucket ready.

Badges are loaded from the CDN instead of being embedded into Discord, so we need to download them and re-upload them to 
the S3 bucket, a convenience script located in [examples/badges.sh](examples/badges.sh) can do this for you if you have
the AWS cli installed and configured, be sure to run the script with the bucket name as an argument (`badges.sh epl`).

If you want to provide City locations for sessions, you should also download the 
[MaxMind GeoLite2](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data) City database.

Upon launching Epl, the database migrations will automatically be applied _(note: subject to change)_.

## Running
Running Epl is relatively easy and fast, each service needs a basic set of environment variables to start. You can see
all the available configuration environment variables down in the [Environment Variable Glossary](#environment-variable-glossary),
generally you will be **required** to set `URL`, `GATEWAY_URL`, `DATABASE_URL`, `NATS_ADDR`, `LVSP_SECRET`, and `LISTEN_ADDR` 
but also you may **want** to set `REQUIRE_SSL`, `REGISTRATION`, and `AWS_ENDPOINT_URL`. Make sure to set `LISTEN_ADDR` to
different ports for each service to avoid conflicts.

If all the provided environment variables are correct, all three services should start up without errors and start to
accept connections, you should configure a reverse proxy webserver to point to the various ports that Epl is listening on.
As a reference, an example Caddy config is provided at [examples/Caddyfile](examples/Caddyfile).

An easy way of verifying that Epl is working is by heading to the landing page, it should report the name of the instance,
the description, the version, and some stats.

At this point, you may also want to create services for Epl to be started via your init system, it's preferable to run Epl as its own
user without elevated privileges.

## Connecting
There is currently no public Discord client modification (to my knowledge) that allows you to modify the endpoints used
by Discord, you will either have to create one yourself 
(hint: modify `window.GLOBAL_ENV` and also disable compression and ETF encoding) or use a modified third-party client.

Litecord has a list of some third-party clients that can connect to custom endpoints 
[here](https://gitlab.com/litecord/litecord/-/blob/master/docs/clients.md).

## Environment Variable Glossary

|      Variable      |                                                                      Description                                                                       |                      Example                      | Required? |       Default        |
|:------------------:|:------------------------------------------------------------------------------------------------------------------------------------------------------:|:-------------------------------------------------:|:---------:|:--------------------:|
|     `RUST_LOG`     |                                  Rust logging level (See [env_logger](https://docs.rs/env_logger/latest/env_logger/))                                  |                      `INFO`                       |           |       `ERROR`        |
|       `NAME`       |                                                                  The instance's name                                                                   |                       `Epl`                       |           |        `Epl`         |
|   `DESCRIPTION`    |                                                               The instance's description                                                               |                 `An Epl instance`                 |           |  `An Epl instance`   |
|       `URL`        |                                                                The instance's main URL                                                                 |                     `epl.dev`                     |    [x]    |         N/A          |
|   `GATEWAY_URL`    |                                                 The URL of the Gateway (Must be accessible to clients)                                                 |                 `gateway.epl.dev`                 |    [x]    |         N/A          |
|  `MEDIAPROXY_URL`  |                                                           The URL of the Litecord mediaproxy                                                           |                  `media.epl.dev`                  |           |         N/A          |
|   `LISTEN_ADDR`    |                                                        Listen address of the service configured                                                        |                  `0.0.0.0:3926`                   |           |    `0.0.0.0:3926`    |
|   `DATABASE_URL`   |                                                                PostgreSQL database URL                                                                 | `postgres://username:password@localhost/database` |    [x]    |         N/A          |
|    `NATS_ADDR`     |                                                            [NATS](https://nats.io/) address                                                            |                 `127.0.0.1:4222`                  |           |   `127.0.0.1:4222`   |
|   `LVSP_SECRET`    |                                       LVSP Shared Secret, can be anything (Must be the same on the LVSP server)                                        |                   `supersecret`                   |    [x]    |         N/A          |
|   `REQUIRE_SSL`    |                                              Whether or not SSL protocols will be used (wss:// https://)                                               |                      `true`                       |           |       `false`        |
|   `REGISTRATION`   |                                                         Whether or not registration is allowed                                                         |                      `true`                       |           |       `false`        |
|      `POMELO`      |                                         If Pomelo (the new username system) should be enabled on this instance                                         |                      `true`                       |           |       `false`        |
| `MAXMIND_DB_PATH`  | Path to a local copy of the [MaxMind GeoLite 2](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data) city database used for session locations |               `GeoLite2-City.mmdb`                |           | `GeoLite2-City.mmdb` |
|    `S3_BUCKET`     |                                                       Name of the bucket used for storing files                                                        |                       `epl`                       |           |        `epl`         |
| `AWS_ENDPOINT_URL` |                        Used to workaround an [issue](https://github.com/awslabs/aws-sdk-rust/issues/932) with the Rust AWS SDK                         |              `http://127.0.0.1:3900`              |           |         N/A          |
