use regex::{Regex, RegexBuilder};

#[derive(Clone)]
pub struct Options {
    pub windows: bool,
    pub truncate: bool,
    pub replacement: String,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            windows: cfg!(windows),
            truncate: true,
            replacement: "".to_string(),
        }
    }
}

impl Options {
    pub fn default_with_replacement(replacement: &str) -> Self {
        Options {
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
    pub fn new(option: Option<Options>) -> Sanitizer {
        let options_retrieved = option.unwrap_or_default();


        Sanitizer {
            illegal_re: Regex::new(r#"[/\?<>\\:\*\|":]"#).unwrap(),
            control_re: Regex::new(r#"[\x00-\x1f\x80-\x9f]"#).unwrap(),
            reserved_re: Regex::new(r#"^\.+$"#).unwrap(),
            windows_reserved_re: RegexBuilder::new(r#"(?i)^(con|prn|aux|nul|com[0-9]|lpt[0-9])(\..*)?$"#)
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
        let Options { windows, truncate, replacement } = self.options.clone();
        let name = name.as_ref();

        let name = self.illegal_re.replace_all(name, &replacement);
        let name = self.control_re.replace_all(&name, &replacement);
        let name = self.reserved_re.replace(&name, &replacement);

        let collect = |name: ::std::borrow::Cow<str>| {
            if truncate && name.len() > 255 {
                let mut end = 255;
                loop {
                    if name.is_char_boundary(end) { break; }
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
        let Options { windows, truncate, .. } = self.options;
        let name = name.as_ref();

        if self.illegal_re.is_match(name) {
            return false;
        }
        if self.control_re.is_match(name) {
            return false;
        }
        if self.reserved_re.is_match(name) {
            return false;
        }
        if truncate && name.len() > 255 {
            return false;
        }
        if windows {
            if self.windows_reserved_re.is_match(name) {
                return false;
            }
            if self.windows_trailing_re.is_match(name) {
                return false;
            }
        }

        true
    }
}


#[cfg(test)]
mod tests {
    use crate::utils::file_name_replacement::{Options, Sanitizer};

    // From https://github.com/parshap/node-sanitize-filename/blob/master/test.js
    static NAMES: &[&str] = &[
        "the quick brown fox jumped over the lazy dog",
        "résumé",
        "hello\u{0000}world",
        "hello\nworld",
        "semi;colon.js",
        ";leading-semi.js",
        "slash\\.js",
        "slash/.js",
        "col:on.js",
        "star*.js",
        "question?.js",
        "quote\".js",
        "singlequote'.js",
        "brack<e>ts.js",
        "p|pes.js",
        "plus+.js",
        "'five and six<seven'.js",
        " space at front",
        "space at end ",
        ".period",
        "period.",
        "relative/path/to/some/dir",
        "/abs/path/to/some/dir",
        "~/.\u{0000}notssh/authorized_keys",
        "",
        "h?w",
        "h/w",
        "h*w",
        ".",
        "..",
        "./",
        "../",
        "/..",
        "/../",
        "*.|.",
        "./",
        "./foobar",
        "../foobar",
        "../../foobar",
        "./././foobar",
        "|*.what",
        "LPT9.asdf",
        "foobar..."
    ];

    static NAMES_CLEANED: &[&str] = &[
        "the quick brown fox jumped over the lazy dog",
        "résumé",
        "helloworld",
        "helloworld",
        "semi;colon.js",
        ";leading-semi.js",
        "slash.js",
        "slash.js",
        "colon.js",
        "star.js",
        "question.js",
        "quote.js",
        "singlequote'.js",
        "brackets.js",
        "ppes.js",
        "plus+.js",
        "'five and sixseven'.js",
        " space at front",
        "space at end",
        ".period",
        "period",
        "relativepathtosomedir",
        "abspathtosomedir",
        "~.notsshauthorized_keys",
        "",
        "hw",
        "hw",
        "hw",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        ".foobar",
        "..foobar",
        "....foobar",
        "...foobar",
        ".what",
        "",
        "foobar"
    ];

    static NAMES_IS_SANITIZED: &[bool] = &[
        true,
        true,
        false,
        false,
        true,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        true,
        false,
        false,
        true,
        false,
        true,
        false,
        true,
        false,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false
    ];

    #[test]
    fn it_works() {
        // sanitize
        let options = super::Options {
            windows: true,
            truncate: true,
            replacement: "".to_string(),
        };

        let sanitizer = Sanitizer::new(Some(options));

        for (idx, name) in NAMES.iter().enumerate() {
            assert_eq!(sanitizer.sanitize_with_options(name), NAMES_CLEANED[idx]);
        }

        let long = "a".repeat(300);
        let shorter = "a".repeat(255);
        assert_eq!(sanitizer.sanitize_with_options(long), shorter);

        // is_sanitized
        let options = Options {
            windows: true,
            truncate: true,
            replacement: "".to_string(),
        };
        let sanitizer = Sanitizer::new(Some(options));

        for (idx, name) in NAMES.iter().enumerate() {
            assert_eq!(sanitizer.is_sanitized_with_options(name), NAMES_IS_SANITIZED[idx]);
        }

        let long = "a".repeat(300);
        assert!(!sanitizer.is_sanitized_with_options(long));
    }
}
