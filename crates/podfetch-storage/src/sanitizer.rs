use regex::{Regex, RegexBuilder};

#[derive(Clone)]
pub struct Options {
    pub windows: bool,
    pub truncate: bool,
    pub replacement: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            windows: cfg!(windows),
            truncate: true,
            replacement: String::new(),
        }
    }
}

impl Options {
    pub fn default_with_replacement(replacement: &str) -> Self {
        Self {
            windows: cfg!(windows),
            truncate: true,
            replacement: replacement.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Sanitizer {
    illegal_re: Regex,
    control_re: Regex,
    reserved_re: Regex,
    windows_reserved_re: Regex,
    windows_trailing_re: Regex,
    options: Options,
}

impl Sanitizer {
    pub fn new(option: Option<Options>) -> Self {
        let options_retrieved = option.unwrap_or_default();

        Self {
            illegal_re: Regex::new(r#"[/?<>\\:*|":]"#).unwrap(),
            control_re: Regex::new(r#"[\x00-\x1f\x80-\x9f]"#).unwrap(),
            reserved_re: Regex::new(r#"^\.+$"#).unwrap(),
            windows_reserved_re: RegexBuilder::new(
                r#"(?i)^(con|prn|aux|nul|com[0-9]|lpt[0-9])(\..*)?$"#,
            )
            .case_insensitive(true)
            .build()
            .unwrap(),
            windows_trailing_re: Regex::new(r#"[\. ]+$"#).unwrap(),
            options: options_retrieved,
        }
    }

    pub fn sanitize<S: AsRef<str>>(&self, name: S) -> String {
        self.sanitize_with_options(name)
    }

    pub fn sanitize_with_options<S: AsRef<str>>(&self, name: S) -> String {
        let Options {
            windows,
            truncate,
            replacement,
        } = self.options.clone();
        let name = name.as_ref();

        let name = self.illegal_re.replace_all(name, &replacement);
        let name = self.control_re.replace_all(&name, &replacement);
        let name = self.reserved_re.replace(&name, &replacement);

        let collect = |name: ::std::borrow::Cow<str>| {
            if truncate && name.len() > 255 {
                let mut end = 255;
                while !name.is_char_boundary(end) {
                    end -= 1;
                }
                String::from(&name[..end])
            } else {
                String::from(name)
            }
        };

        if windows {
            let name = self.windows_reserved_re.replace(&name, &replacement);
            let name = self.windows_trailing_re.replace(&name, &replacement);
            collect(name)
        } else {
            collect(name)
        }
    }

    pub fn is_sanitized_with_options<S: AsRef<str>>(&self, name: S) -> bool {
        let Options {
            windows, truncate, ..
        } = self.options.clone();
        let name = name.as_ref();

        if self.illegal_re.is_match(name)
            || self.control_re.is_match(name)
            || self.reserved_re.is_match(name)
            || (truncate && name.len() > 255)
        {
            return false;
        }
        if windows
            && (self.windows_reserved_re.is_match(name) || self.windows_trailing_re.is_match(name))
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Options, Sanitizer};

    #[test]
    fn sanitizes_basic_name() {
        let sanitizer = Sanitizer::new(Some(Options {
            windows: true,
            truncate: true,
            replacement: String::new(),
        }));

        assert_eq!(sanitizer.sanitize("slash/.js"), "slash.js");
        assert!(!sanitizer.is_sanitized_with_options("question?.js"));
    }

    #[test]
    fn truncates_long_strings() {
        let sanitizer = Sanitizer::new(None);
        let long = "a".repeat(300);
        assert_eq!(sanitizer.sanitize(long), "a".repeat(255));
    }
}
