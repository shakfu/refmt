//! Case format definitions and conversion logic

/// Supported case formats for identifier conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseFormat {
    /// camelCase: firstName, lastName
    CamelCase,
    /// PascalCase: FirstName, LastName
    PascalCase,
    /// snake_case: first_name, last_name
    SnakeCase,
    /// SCREAMING_SNAKE_CASE: FIRST_NAME, LAST_NAME
    ScreamingSnakeCase,
    /// kebab-case: first-name, last-name
    KebabCase,
    /// SCREAMING-KEBAB-CASE: FIRST-NAME, LAST-NAME
    ScreamingKebabCase,
}

impl CaseFormat {
    /// Returns the regex pattern for identifying this case format
    pub fn pattern(&self) -> &str {
        match self {
            CaseFormat::CamelCase => r"\b[a-z]+(?:[A-Z]+[a-z0-9]*)+\b",
            CaseFormat::PascalCase => r"\b[A-Z]+[a-z0-9]+(?:[A-Z]+[a-z0-9]*)+\b",
            CaseFormat::SnakeCase => r"\b[a-z]+(?:_[a-z0-9]+)+\b",
            CaseFormat::ScreamingSnakeCase => r"\b[A-Z]+(?:_[A-Z0-9]+)+\b",
            CaseFormat::KebabCase => r"\b[a-z]+(?:-[a-z0-9]+)+\b",
            CaseFormat::ScreamingKebabCase => r"\b[A-Z]+(?:-[A-Z0-9]+)+\b",
        }
    }

    /// Splits a string into words based on this case format
    pub fn split_words(&self, text: &str) -> Vec<String> {
        match self {
            CaseFormat::CamelCase | CaseFormat::PascalCase => {
                // Splitting on *every* capital shatters acronyms:
                // `parseHTTPResponse` became parse_h_t_t_p_response. A run of
                // capitals is one word, ending one character early when the
                // last capital starts the next word (`HTTPResponse`).
                let chars: Vec<char> = text.chars().collect();
                let mut words = Vec::new();
                let mut current_word = String::new();

                for (i, &ch) in chars.iter().enumerate() {
                    if ch.is_uppercase() && !current_word.is_empty() {
                        let prev_is_upper = chars[i - 1].is_uppercase();
                        let next_is_lower = chars.get(i + 1).is_some_and(|c| c.is_lowercase());
                        // Boundary when leaving a lowercase run (`parse|HTTP`),
                        // or at the tail of a capital run that begins a new
                        // word (`HTTP|Response`).
                        if !prev_is_upper || next_is_lower {
                            words.push(current_word.to_lowercase());
                            current_word = String::new();
                        }
                    }
                    current_word.push(ch);
                }

                if !current_word.is_empty() {
                    words.push(current_word.to_lowercase());
                }

                words
            }
            CaseFormat::SnakeCase | CaseFormat::ScreamingSnakeCase => {
                // Split on underscores
                text.split('_')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_lowercase())
                    .collect()
            }
            CaseFormat::KebabCase | CaseFormat::ScreamingKebabCase => {
                // Split on hyphens
                text.split('-')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_lowercase())
                    .collect()
            }
        }
    }

    /// Joins words into this case format with optional prefix and suffix
    pub fn join_words(&self, words: &[String], prefix: &str, suffix: &str) -> String {
        if words.is_empty() {
            return String::new();
        }

        let result = match self {
            CaseFormat::CamelCase => {
                let first = words[0].to_lowercase();
                let rest: String = words[1..]
                    .iter()
                    .map(|w| {
                        let mut chars = w.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().chain(chars).collect(),
                        }
                    })
                    .collect();
                format!("{}{}", first, rest)
            }
            CaseFormat::PascalCase => words
                .iter()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<String>(),
            CaseFormat::SnakeCase => words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("_"),
            CaseFormat::ScreamingSnakeCase => words
                .iter()
                .map(|w| w.to_uppercase())
                .collect::<Vec<_>>()
                .join("_"),
            CaseFormat::KebabCase => words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("-"),
            CaseFormat::ScreamingKebabCase => words
                .iter()
                .map(|w| w.to_uppercase())
                .collect::<Vec<_>>()
                .join("-"),
        };

        format!("{}{}{}", prefix, result, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs of capitals are single words. Splitting on every capital turned
    /// `parseHTTPResponse` into `parse_h_t_t_p_response`.
    #[test]
    fn test_acronyms_are_kept_whole() {
        let cases: [(&str, &[&str]); 6] = [
            ("parseHTTPResponse", &["parse", "http", "response"]),
            ("XMLHttpRequest", &["xml", "http", "request"]),
            ("userName", &["user", "name"]),
            ("parseJSON", &["parse", "json"]),
            ("IOError", &["io", "error"]),
            ("aB", &["a", "b"]),
        ];
        for (input, expected) in cases {
            assert_eq!(
                CaseFormat::CamelCase.split_words(input),
                expected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                "splitting {}",
                input
            );
        }
    }

    /// The end-to-end shape users actually see.
    #[test]
    fn test_acronym_round_trip_to_snake_case() {
        let words = CaseFormat::CamelCase.split_words("parseHTTPResponse");
        assert_eq!(
            CaseFormat::SnakeCase.join_words(&words, "", ""),
            "parse_http_response"
        );
    }

    /// The candidate patterns must match acronym-bearing identifiers at all,
    /// otherwise fixing the splitter changes nothing in practice.
    #[test]
    fn test_patterns_match_acronym_identifiers() {
        let camel = regex::Regex::new(CaseFormat::CamelCase.pattern()).unwrap();
        assert!(camel.is_match("parseHTTPResponse"));
        assert!(camel.is_match("userName"));

        let pascal = regex::Regex::new(CaseFormat::PascalCase.pattern()).unwrap();
        assert!(pascal.is_match("XMLHttpRequest"));
        assert!(pascal.is_match("PascalCase"));
        // A single capitalised word is still not a conversion candidate --
        // otherwise ordinary prose in .md files would be rewritten.
        assert!(!pascal.is_match("Hello"));
    }

    #[test]
    fn test_camel_split() {
        let words = CaseFormat::CamelCase.split_words("firstName");
        assert_eq!(words, vec!["first", "name"]);
    }

    #[test]
    fn test_snake_split() {
        let words = CaseFormat::SnakeCase.split_words("first_name");
        assert_eq!(words, vec!["first", "name"]);
    }

    #[test]
    fn test_camel_join() {
        let words = vec!["first".to_string(), "name".to_string()];
        assert_eq!(
            CaseFormat::CamelCase.join_words(&words, "", ""),
            "firstName"
        );
    }

    #[test]
    fn test_snake_join() {
        let words = vec!["first".to_string(), "name".to_string()];
        assert_eq!(
            CaseFormat::SnakeCase.join_words(&words, "", ""),
            "first_name"
        );
    }

    #[test]
    fn test_with_prefix_suffix() {
        let words = vec!["first".to_string(), "name".to_string()];
        assert_eq!(
            CaseFormat::SnakeCase.join_words(&words, "old_", "_v1"),
            "old_first_name_v1"
        );
    }
}
