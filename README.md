# SeismicDB
[![Rust](https://github.com/alice-comfy/SeismicDB/actions/workflows/rust.yml/badge.svg)](https://github.com/alice-comfy/SeismicDB/actions/workflows/rust.yml)
[![crate.io](https://img.shields.io/crates/v/sdb-core.svg)](https://crates.io/crates/sdb-core)
[![doc.rs](https://docs.rs/sdb_core/badge.svg)](https://docs.rs/crate/sdb_core)
![Minimum Rust version](https://img.shields.io/badge/rustc-1.40+-yellow.svg)
![Rust stable](https://img.shields.io/badge/rust-stable-green.svg)

| crate | docs.rs | crate.io |
| - | - | ------ |
| seismicdb  | [![doc.rs](https://docs.rs/tectonicdb/badge.svg)](https://docs.rs/crate/tectonicdb) | [![crate.io](https://img.shields.io/crates/v/seismicdb.svg)](https://crates.io/crates/seismicdb)  |
| sdb-core  | [![doc.rs](https://docs.rs/tdb_core/badge.svg)](https://docs.rs/crate/tdb_core) | [![crate.io](https://img.shields.io/crates/v/sdb-core.svg)](https://crates.io/crates/sdb-core)  |
| sdb-server-core  | [![doc.rs](https://docs.rs/tdb_server_core/badge.svg)](https://docs.rs/crate/tdb_server_core) | [![crate.io](https://img.shields.io/crates/v/sdb-server-core.svg)](https://crates.io/crates/sdb-server-core)  |
| sdb-cli  | [![doc.rs](https://docs.rs/tdb_cli/badge.svg)](https://docs.rs/crate/sdb_client) | [![crate.io](https://img.shields.io/crates/v/sdb-client.svg)](https://crates.io/crates/sdb-client)  |

SeismicDB is a fast, highly compressed standalone database and streaming protocol for order book ticks.

SeismicDB Is forked from the inactive, but briliant TectonicDB. https://github.com/0b01/tectonicdb

## Why

* Uses a simple and efficient binary file format: Dense Tick Format(DTF)

* Stores order book tick data tuple of shape: `(timestamp, seq, is_trade, is_bid, price, size)`.

* Sorted by timestamp + seq

* 12 bytes per orderbook event

* 600,000 inserts per thread second

## Installation

There are several ways to install seismicdb.

1.  **Binaries**

Binaries are available for [download](https://github.com/alice-comfy/SeismicDB/releases). Make sure to put the path to the binary into your PATH. Currently only build is for Linux x86_64.

2.  **Crates**

```
cargo install seismicdb
```

This command will download `sdb`, `sdb-server`, `dtftools` binaries from crates.io and build locally.

3.  **GitHub**

To contribute you will need the copy of the source code on your local machine.

    git clone https://github.com/alice-comfy/SeismicDB
    cd seismicdb
    cargo build --release
    cargo run --release sdb-server

The binaries can be found under `target/release`.

## How to use

It's very easy to setup.

```
./sdb-server --help
```

For example:

```bash
./sdb-server -vv -a -i 10000
# run the server on INFO verbosity
# turn on autoflush for every 10000 inserts per orderbook
```

### Configuration

To config the Google Cloud Storage and Data Collection Backend integration, the following environment variables are used:

| Variable Name                 | Default      | Description                                                                                                                                   |
| ----------------------------- | ------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| `SDB_HOST`             | 0.0.0.0      | The host to which the database will bind                                                                                                      |
| `SDB_PORT`             | 9001         | The port that the database will listen on                                                                                                     |
| `SDB_DTF_FOLDER`       | db           | Name of the directory in which DTF files will be stored                                                                                       |
| `SDB_AUTOFLUSH`        | false        | If `true`, recorded orderbook data will automatically be flushed to DTF files every `interval` inserts.                                       |
| `SDB_FLUSH_INTERVAL`   | 1000         | Every `interval` inserts, if `autoflush` is enabled, DTF files will be written from memory to disk.                                           |
| `SDB_GRANULARITY`      | 0            | Record history granularity level                                                                                                              |
| `SDB_LOG_FILE_NAME`    | sdb.log      | Filename of the log file for the database                                                                                                     |
| `SDB_Q_CAPACITY`       | 300          | Capacity of the circular queue for recording history                                                                                          |

## Client API

| Command | Description |
| :--- | :--- |
| HELP | Prints help |
| PING | Responds PONG |
| INFO | Returns info about table schemas |
| PERF | Returns the answercount of items over time |
| LOAD \[orderbook\] | Load orderbook from disk to memory |
| USE \[orderbook\] | Switch the current orderbook |
| CREATE \[orderbook\] | Create orderbook |
| GET \[n\] FROM \[orderbook\] | Returns items |
| GET \[n\] | Returns n items from current orderbook |
| COUNT | Count of items in current orderbook |
| COUNT ALL | Returns total count from all orderbooks |
| CLEAR | Deletes everything in current orderbook |
| CLEAR ALL | Drops everything in memory |
| FLUSH | Flush current orderbook to "Howdisk can|
| FLUSHALL | Flush everything from memory to disk |
| SUBSCRIBE \[orderbook\] | Subscribe to updates from orderbook |
| EXISTS \[orderbook\] | Checks if orderbook exists |
| SUBSCRIBE \[orderbook\] | Subscribe to orderbook |

### Data commands

```
USE [dbname]
ADD [ts], [seq], [is_trade], [is_bid], [price], [size];
INSERT 1505177459.685, 139010, t, f, 0.0703620, 7.65064240; INTO dbname
```

## Monitoring

TectonicDB supports monitoring/alerting by periodically sending its usage info to an InfluxDB instance:

```bash
    --influx-db <influx_db>                        influxdb db
    --influx-host <influx_host>                    influxdb host
    --influx-log-interval <influx_log_interval>    influxdb log interval in seconds (default is 60)
```

As a concrete example,

```bash
...
$ influx
> CREATE DATABASE market_data;
> ^D
$ sdb --influx-db market_data --influx-host http://localhost:8086 --influx-log-interval 20
...
```

TectonicDB will send field values `disk={COUNT_DISK},size={COUNT_MEM}` with tag `ob={ORDERBOOK}` to `market_data` measurement which is the same as the dbname.

Additionally, you can query usage information directly with `INFO` and `PERF` commands:

1. `INFO` reports the current tick count in memory and on disk.

2. `PERF` returns recorded tick count history whose granularity can be configured.

## Logging

Log file defaults to `sdb.log`.

## Testing

```bash
export RUST_TEST_THREADS=1
cargo test
```

Tests must be run sequentially because some tests depend on dtf files that other tests generate.

## Benchmark

sdb client comes with a benchmark mode. This command inserts 1M records into the sdb.

```bash
sdb -b 1000000
```
## Using dtf files

Seismic comes with a commandline tool `dtfcat` to inspect the file metadata and all the stored events into either JSON or CSV.

Options:

```
USAGE:
    dtfcat [FLAGS] --input <INPUT>

FLAGS:
    -c, --csv         output csv
    -h, --help        Prints help information
    -m, --metadata    read only the metadata
    -V, --version     Prints version information

OPTIONS:
    -i, --input <INPUT>    file to read
```

## As a library

It is possible to use the Dense Tick Format streaming protocol / file format in a different application. Works nicely with any buffer implementing the `Write` trait.

## Requirements

TectonicDB is a standalone service.

* Linux

* macOS

Language bindings:

* [x] TypeScript

* [x] Rust

* [x] Python

* [x] JavaScript

## Additional Features

* [x] Usage statistics like Cloud SQL

* [x] Commandline inspection tool for dtf file format

* [x] Logging

* [x] Query by timestamp

# Changelog

* 0.6.0: First seismicDB Fork release. Upgraded dependencies and rust version to 2021 / latest versions. Rebrand and release new version on crates.io. 
* 0.5.0: InfluxDB monitoring plugin and improved command line arguments
* 0.4.0: iterator-based APIs for handling DTF files and various quality of life improvements
* 0.3.0: Refactor to async
