use crate::{
    envvar,
    traits::{DirCreator, UnwrapAndLogRes},
};

mod launcher;
mod responder;
mod walker;

pub async fn initialize() {
    let home = envvar!("HOME");
    let path = format!("{}/.cargo/rise_code", home);
    if !path.check() {
        path.mkdir().unwrap_log();
    }

    let launcher_path = format!("{}/launcher.sh", path);
    if !launcher_path.check() {
        launcher::create(&launcher_path).await.unwrap_log();
    }

    let responder_path = format!("{}/responder.sh", path);
    if !responder_path.check() {
        responder::create(&responder_path).await.unwrap_log();
    }

    walker::walk(home).await;
}
