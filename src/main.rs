mod cli;
mod mgmt;

#[tokio::main]
async fn main() {
    CryptoProvider::install_default();
    cli::run().await;
}
