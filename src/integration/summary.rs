use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use super::runtime::run_until_stopped;
use crate::prelude::{Book, BookKind, BookQueue, Empty, OrderBook, Summary};

const RESULT_SIZE: usize = 10;

pub struct SummaryService;

#[async_trait]
impl OrderBook for SummaryService {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;
    #[tracing::instrument(name = "Book Summary", skip(self, _request))]
    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (book_tx, book_rx) = mpsc::channel(1000);
        let (stop_tx, stop_rx) = oneshot::channel();

        tokio::spawn(async move {
            run_until_stopped(book_tx, stop_rx).await;
        });
        let (tx, rx) = mpsc::channel(1000);
        tokio::spawn(async move {
            push_books(tx, book_rx, RESULT_SIZE).await;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tracing::instrument(name = "Pushes book to the queue", skip(summary, books, size))]
async fn push_books(
    summary: mpsc::Sender<Result<Summary, Status>>,
    mut books: mpsc::Receiver<(BookKind, Book)>,
    size: usize,
) {
    let mut bid_book = BookQueue::with_capacity(BookKind::Bids, size);
    let mut ask_book = BookQueue::with_capacity(BookKind::Asks, size);

    while let Some((kind, book)) = books.recv().await {
        let exchange = book.exchange.parse().unwrap();
        tracing::info!(
            "received book '{}' book {:?} from exchange: {}'",
            kind.as_ref(),
            book,
            book.exchange,
        );
        match kind {
            BookKind::Asks => ask_book.push(exchange, book),
            BookKind::Bids => bid_book.push(exchange, book),
        };

        let spread = ask_book.max_price() - bid_book.max_price();

        if let Err(e) = summary
            .send(Ok(Summary {
                spread: spread.to_string(),
                asks: ask_book.take(size),
                bids: bid_book.take(size),
            }))
            .await
        {
            tracing::error!("failed so send summary: {}", e);
        }
    }
}
