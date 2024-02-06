use goodreads_api::goodreads_api::search;
use tokio::main;

#[tokio::main]
async fn main() {
    search("Dresden Files").await;
}