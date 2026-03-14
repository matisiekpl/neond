mod cli;
mod mgmt;

#[tokio::main]
async fn main() {
    cli::run().await;
}
