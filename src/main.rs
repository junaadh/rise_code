pub mod client;
pub mod events;
mod init;
pub mod interface;
pub mod listener;
pub mod loader;
pub mod macros;
pub mod traits;

#[cfg(unix)]
#[tokio::main]
async fn main() {
    use tokio::sync::mpsc;

    init::initialize().await;

    loader::helpers::setup_log(".cache/rise_code.log");
    let (tx, rx) = mpsc::channel::<interface::code::Code>(1);

    tokio::spawn(async move {
        listener::start(tx).await;
    });

    client::run::run(rx).await;
}
