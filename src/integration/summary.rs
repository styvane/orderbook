//! Summary service type.
//!
//! This module implement the summary service.

use std::pin::Pin;
use std::task::{Context, Poll};

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use super::runtime::run_until_stopped;
use super::transport::StopSender;
use crate::prelude::{Book, BookKind, BookQueue, Configuration, Empty, OrderBook, Summary};

pub struct SummaryService {
    pub config: Configuration,
}

impl Default for SummaryService {
    fn default() -> Self {
        Self::new()
    }
}

impl SummaryService {
    /// Creates new summary service.
    pub fn new() -> Self {
        let config = Configuration::new().expect("failed to get configuration");
        Self { config }
    }
}

#[async_trait]
impl OrderBook for SummaryService {
    type BookSummaryStream = SummaryStream;
    #[tracing::instrument(name = "Book Summary", skip(self, _request))]
    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (book_tx, book_rx) = mpsc::channel(self.config.result_size);
        let (stop_tx, stop_rx) = oneshot::channel();

        let config = self.config.exchanges.clone();
        let size = self.config.result_size;

        tokio::spawn(async move {
            run_until_stopped(size, config, book_tx, stop_rx).await;
        });
        let (tx, rx) = mpsc::channel(size);

        tokio::spawn(async move {
            stream_books(tx, book_rx, size).await;
        });
        let stream = SummaryStream {
            inner: ReceiverStream::new(rx),
            stop_request: StopSender::new(stop_tx),
        };

        Ok(Response::new(stream))
    }
}

pub struct SummaryStream {
    inner: ReceiverStream<Result<Summary, Status>>,
    stop_request: StopSender,
}

impl Drop for SummaryStream {
    fn drop(&mut self) {
        let _ = self.stop_request.try_stop();
    }
}

impl Stream for SummaryStream {
    type Item = Result<Summary, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

#[tracing::instrument(name = "Streams book", skip(summary, books, size))]
async fn stream_books(
    summary: mpsc::Sender<Result<Summary, Status>>,
    mut books: mpsc::Receiver<(BookKind, Book)>,
    size: usize,
) {
    let mut bid_book = BookQueue::with_capacity(BookKind::Bids, size);
    let mut ask_book = BookQueue::with_capacity(BookKind::Asks, size);

    while let Some((kind, book)) = books.recv().await {
        tracing::info!(
            "received book '{}' book {:?} from exchange: {}'",
            kind.as_ref(),
            book,
            book.exchange,
        );
        let exchange = book.exchange.parse().unwrap();

        match kind {
            BookKind::Asks => ask_book.push(exchange, book),
            BookKind::Bids => bid_book.push(exchange, book),
        };

        let spread = ask_book.max_price() - bid_book.max_price();

        if let Err(e) = summary
            .send(Ok(Summary {
                spread: spread.abs().to_string(),
                asks: ask_book.take(size),
                bids: bid_book.take(size),
            }))
            .await
        {
            tracing::error!("failed so send summary: {}", e);
        }
    }
}
