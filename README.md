Order Book
==========

[<img alt="https://github.com/styvane/orderbook/actions/workflows/ci.yaml" src="https://img.shields.io/github/workflow/status/styvane/orderbook/CI/main">](https://github.com/styvane/orderbook/actions/workflows/ci.yaml) [<img alt="https://img.shields.io/github/license/styvane/orderbook" src="https://img.shields.io/github/license/styvane/orderbook">](LICENSE.txt) ![GitHub last commit (branch)](https://img.shields.io/github/last-commit/styvane/orderbook/main)


Requirements
------------
The only requirement for this is Rust.

Additional you can install [bunyan-rs](https://crates.io/crates/bunyan)

Run
---

To run this, we need a configuration see [settings](settings). However the default configuration should be enough.
To better visualize the logs.

Run the following command in two different terminals.

```bash
$ cargo run --bin orderbook-server 
$ cargo run --bin orderbook-client
```

To see the server output log, set *RUST_LOG* to a valid log filter before running the server.

```
$ RUST_LOG=debug cargo run --bin orderbook-server
```

Note that you don't have the run the `orderbook-client` any GRPC client should work. For example [grpcurl](https://github.com/fullstorydev/grpcurl) using the following:

```bash
$ grpcurl -vv -plaintext -import-path ./proto -proto orderbook.proto -d '{}' [::1]:12000 orderbook.OrderBook/BookSummary
```



Stats
-----
![Alt](https://repobeats.axiom.co/api/embed/26b0d356ff075d38906976ee7182b1ff86587efe.svg "Repobeats analytics image")
