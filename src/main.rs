mod cli;
mod mgmt;
mod preflight;

#[tokio::main]
async fn main() {
    cli::run().await;
}
