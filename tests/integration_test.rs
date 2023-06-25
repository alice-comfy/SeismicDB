extern crate sdb_server_core;
extern crate sdb_client;

use sdb_server_core::async_std::task;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn it_works() {
    let host = "0.0.0.0";
    let port = "9001";

    let settings = Arc::new(sdb_server_core::settings::Settings {
        autoflush: false,
        dtf_folder: "./testdb".to_owned(),
        flush_interval: 1000,
        granularity: 1000,
        q_capacity: 1000,
        influx: None,
    });

    task::block_on(async move {
        let _server = task::spawn(sdb_server_core::server::run_server(&host, &port, settings));

        let cli = sdb_client::client_from_env();
        sdb_client::benchmark(cli, 10_000);

        let mut cli = sdb_client::client_from_env();
        cli.use_db("benchmark").unwrap();
        task::sleep(Duration::from_secs(15)).await;
        let ret = cli.cmd("COUNT ALL IN MEM\n").unwrap();
        assert_eq!(ret, "10000");

    });
}
