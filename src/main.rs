mod cli;
mod mgmt;
mod preflight;
mod unpacker;
mod daemon;

#[tokio::main]
async fn main() {
    cli::run().await.unwrap_or_else(|e| panic!("{}", e));
}
