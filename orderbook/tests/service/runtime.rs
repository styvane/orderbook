use orderbook::prelude::runtime::run_until_stopped;
use orderbook::prelude::Book;
use tokio::sync::{mpsc::channel, oneshot};
use tokio::time::{self, Duration};

use crate::force_lazy;

#[tokio::test]
async fn runtime_can_publish_books() {
    force_lazy();

    let (stop_tx, stop_rx) = oneshot::channel();
    let mut interval = time::interval(Duration::from_secs(20));

    interval.tick().await;
    tokio::spawn(async move {
        interval.tick().await;
        let _ = stop_tx.send(true);
    });

    let (tx, mut rx) = channel(10);
    run_until_stopped(tx, stop_rx).await;
    while let Some(b) = rx.recv().await {
        assert!(matches!(b, Book { .. }));
    }
}
