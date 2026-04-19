mod auth;
mod cli;
mod mgmt;
mod preflight;
mod daemon;
mod utils;

#[tokio::main]
async fn main() {
    if let Err(error) = cli::run().await {
        tracing::error!("{}", error);
        std::process::exit(1);
    }
}
