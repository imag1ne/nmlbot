#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    nmlbot::start_bot().await;
}
