use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LanguageExt {
    pub name: Language,
    counter: HashMap<Language, usize>,
}

impl Default for LanguageExt {
    fn default() -> Self {
        Self {
            name: Language::Unknown,
            counter: HashMap::new(),
        }
    }
}

impl LanguageExt {
    pub fn push(&mut self, lang: &str) {
        let language = Language::get_language_name(lang);
        self.name = language.clone();
        *self.counter.entry(language).or_insert(1) += 1;
    }

    pub fn push_ext(&mut self, ext: &str) {
        let language = Language::get_language(ext);
        self.push(&language)
    }

    pub fn get_max(&self) -> Language {
        self.counter
            .iter()
            .filter(|&(lang, _)| *lang != Language::Unknown)
            .max_by_key(|&(_, count)| *count)
            .map(|(lang, _)| lang.clone())
            .unwrap_or_default()
    }

    pub fn get_max_ext(&mut self) {
        self.name = self.get_max();
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
    OCaml,
    Unknown,
}

impl Default for Language {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Language {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let res = match self {
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Css => "css",
            Self::Go => "go",
            Self::Html => "html",
            Self::Java => "java",
            Self::Javascript => "javascript",
            Self::Lua => "lua",
            Self::Python => "python",
            Self::R => "r",
            Self::Rust => "rust",
            Self::Typescript => "typescript",
            &Self::OCaml => "ocaml",
            _ => "unknown",
        };
        res.to_string()
    }

    pub fn get_logo(&self) -> String {
        let lang = self.to_string();
        if lang == *"unknown" {
            "wild-card".to_owned()
        } else {
            format!("{lang}-logo")
        }
    }

    pub fn get_language_name(lang: &str) -> Self {
        match lang {
            "c" => Self::C,
            "cpp" => Self::Cpp,
            "css" => Self::Css,
            "go" => Self::Go,
            "html" => Self::Html,
            "java" => Self::Java,
            "javascript" => Self::Javascript,
            "lua" => Self::Lua,
            "python" => Self::Python,
            "r" => Self::R,
            "rust" => Self::Rust,
            "typescript" => Self::Typescript,
            "ocaml" => Self::OCaml,
            _ => Self::Unknown,
        }
    }

    pub fn get_language(ext: &str) -> String {
        match ext {
            "c" | "h" => Self::C.to_string(),
            "cpp" | "hpp" | "cc" | "hh" | "cxx" | "hxx" => Self::Cpp.to_string(),
            "css" => Self::Css.to_string(),
            "go" => Self::Go.to_string(),
            "html" | "htm" => Self::Html.to_string(),
            "java" => Self::Java.to_string(),
            "js" | "jsx" => Self::Javascript.to_string(),
            "lua" => Self::Lua.to_string(),
            "py" => Self::Python.to_string(),
            "r" => Self::R.to_string(),
            "rs" => Self::Rust.to_string(),
            "ts" | "tsx" => Self::Typescript.to_string(),
            "ml" | "mli" => Self::OCaml.to_string(),
            _ => "unknown".to_string(),
        }
    }
}
