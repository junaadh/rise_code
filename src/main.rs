pub mod client;
pub mod events;
pub mod interface;
pub mod listener;
pub mod loader;

#[cfg(unix)]
#[tokio::main]
async fn main() {
    use tokio::sync::mpsc;

    loader::helpers::setup_log(".cache/rise_code.log");
    let (tx, rx) = mpsc::channel::<interface::code::Code>(1);

    tokio::spawn(async move {
        listener::start(tx).await;
    });

    client::run::run(rx).await;
}
