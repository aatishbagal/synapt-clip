use std::collections::HashSet;

use crate::search::trie::Trie;

pub enum ClipCategory {
    Link,
    FilePath,
    Code,
    Email,
    Color,
    PlainText,
}

impl ClipCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ClipCategory::Link => "Link",
            ClipCategory::FilePath => "File Path",
            ClipCategory::Code => "Code",
            ClipCategory::Email => "Email",
            ClipCategory::Color => "Color",
            ClipCategory::PlainText => "Plain Text",
        }
    }
}

pub struct ClipClassifier {
    prefix_trie: Trie,
    code_keywords: HashSet<String>,
}

impl ClipClassifier {
    pub fn new() -> Self {
        let mut prefix_trie = Trie::new();

        for prefix in &[
            "http://", "https://", "ftp://", "www.",
            "/", "~/", "./", "../", "c:\\", "d:\\", "e:\\",
        ] {
            prefix_trie.insert(prefix, 0);
        }

        let keywords = [
            "fn ", "def ", "class ", "import ", "#include", "const ", "let ", "var ",
            "return ", "if (", "for (", "while (", "struct ", "pub ", "async ", "await ",
            "=>", "{}", "();",
        ];
        let code_keywords = keywords.iter().map(|k| k.to_string()).collect();

        Self { prefix_trie, code_keywords }
    }

    pub fn classify(&self, content: &str) -> ClipCategory {
        if content.trim().is_empty() {
            return ClipCategory::PlainText;
        }

        let content_lower = content.to_lowercase();

        if self.trie_prefix_hit(&content_lower) {
            if content_lower.starts_with("http://")
                || content_lower.starts_with("https://")
                || content_lower.starts_with("ftp://")
                || content_lower.starts_with("www.")
            {
                return ClipCategory::Link;
            }
            if content_lower.starts_with('/')
                || content_lower.starts_with("~/")
                || content_lower.starts_with("./")
                || content_lower.starts_with("../")
                || content_lower.starts_with("c:\\")
                || content_lower.starts_with("d:\\")
                || content_lower.starts_with("e:\\")
            {
                return ClipCategory::FilePath;
            }
        }

        if let Some(at_pos) = content.find('@') {
            if at_pos > 0 {
                let after_at = &content[at_pos + 1..];
                if after_at.contains('.') && !after_at.is_empty() {
                    return ClipCategory::Email;
                }
            }
        }

        if content.starts_with('#') && (content.len() == 7 || content.len() == 4) {
            let hex_part = &content[1..];
            if hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                return ClipCategory::Color;
            }
        }

        for kw in &self.code_keywords {
            if content.contains(kw.as_str()) {
                return ClipCategory::Code;
            }
        }

        ClipCategory::PlainText
    }

    fn trie_prefix_hit(&self, content_lower: &str) -> bool {
        let chars: Vec<char> = content_lower.chars().take(9).collect();
        for len in 1..=chars.len() {
            let q: String = chars[..len].iter().collect();
            if !self.prefix_trie.prefix_search(&q).is_empty() {
                return true;
            }
        }
        false
    }
}

impl Default for ClipClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat(content: &str) -> &'static str {
        ClipClassifier::new().classify(content).label()
    }

    #[test]
    fn link_https() {
        assert_eq!(cat("https://github.com/user/repo"), "Link");
    }

    #[test]
    fn link_http() {
        assert_eq!(cat("http://example.com"), "Link");
    }

    #[test]
    fn link_www() {
        assert_eq!(cat("www.google.com"), "Link");
    }

    #[test]
    fn filepath_absolute() {
        assert_eq!(cat("/home/user/documents/file.txt"), "File Path");
    }

    #[test]
    fn filepath_home() {
        assert_eq!(cat("~/projects/synaptclip"), "File Path");
    }

    #[test]
    fn filepath_relative() {
        assert_eq!(cat("./src/main.rs"), "File Path");
    }

    #[test]
    fn code_fn() {
        assert_eq!(cat("fn main() { println!(\"hello\"); }"), "Code");
    }

    #[test]
    fn code_import() {
        assert_eq!(cat("import React from react"), "Code");
    }

    #[test]
    fn email() {
        assert_eq!(cat("user@example.com"), "Email");
    }

    #[test]
    fn color_hex7() {
        assert_eq!(cat("#1a2b3c"), "Color");
    }

    #[test]
    fn color_hex4() {
        assert_eq!(cat("#fff"), "Color");
    }

    #[test]
    fn color_invalid_chars_not_color() {
        assert_ne!(cat("#xyz123"), "Color");
    }

    #[test]
    fn plain_text() {
        assert_eq!(cat("hello world this is plain text"), "Plain Text");
    }

    #[test]
    fn empty_string_plain_text() {
        assert_eq!(cat(""), "Plain Text");
    }
}
