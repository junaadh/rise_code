pub enum Language {
    C,
    Cpp,
    Css,
    Go,
    Html,
    Java,
    Javascript,
    Lua,
    Python,
    R,
    Rust,
    Typescript,
}

impl Language {
    fn match_lang(&self) -> &str {
        match self {
            Self::C => "c-logo",
            Self::Cpp => "cpp-logo",
            Self::Css => "css-logo",
            Self::Go => "go-logo",
            Self::Html => "html-logo",
            Self::Java => "java-logo",
            Self::Javascript => "javascript-logo",
            Self::Lua => "lua-logo",
            Self::Python => "python-logo",
            Self::R => "r-logo",
            Self::Rust => "rust-logo",
            Self::Typescript => "typescript-logo",
        }
    }

    pub fn get_logo(lang: &str) -> &str {
        match lang {
            "c" => Self::C.match_lang(),
            "cpp" => Self::Cpp.match_lang(),
            "css" => Self::Css.match_lang(),
            "go" => Self::Go.match_lang(),
            "html" => Self::Html.match_lang(),
            "java" => Self::Java.match_lang(),
            "javascript" => Self::Javascript.match_lang(),
            "lua" => Self::Lua.match_lang(),
            "python" => Self::Python.match_lang(),
            "r" => Self::R.match_lang(),
            "rust" => Self::Rust.match_lang(),
            "typescript" => Self::Typescript.match_lang(),
            _ => "wild-card",
        }
    }

    // TODO: get language by extension
}
