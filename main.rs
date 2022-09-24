use tokio;
mod dithering;
mod minecraftify;
mod cli;
#[tokio::main]
async fn main() {
    cli::run().await;
}
