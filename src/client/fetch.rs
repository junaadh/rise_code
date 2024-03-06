use crate::{loader::parser, sleep, traits::DirCreator};

pub async fn fetch_info(code: &mut crate::interface::code::Code) -> Result<(), ()> {
    sleep!(5);
    let info = parser::get_window_id(&code.tmux_session).unwrap_or_default();
    let mut language = code.language.clone();
    let pane_content = if !info.is_empty() {
        parser::get_pane_content(info, code)
    } else {
        "".to_string()
    };
    let mut active_file = String::new();

    if !pane_content.is_empty() {
        active_file = parser::parse_pane(pane_content);
        let (_body, ext) = active_file.rsplit_once('.').unwrap_or_default();
        if !ext.is_empty() {
            language.push_ext(ext);
        } else {
            language.get_max_ext();
        }
    }
    code.language(language);
    code.file(&active_file);
    Ok(())
}

// pub async fn get_git(path: &str) -> String {
//     let git = format!("{path}/.git");
//     if !git.check() {
//         "https://www.github.com".to_string()
//     } else {
//     }
// }
