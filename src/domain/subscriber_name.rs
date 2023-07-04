use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        let s = s.trim();
        let is_empty = s.is_empty();
        let is_long = s.graphemes(true).count() > 256;
        let forbidden_chars = ['/', '\\', '<', '>', '(', ')', '{', '}', '"'];
        let has_forbidden_chars = s.chars().any(|c| forbidden_chars.contains(&c));
        if is_empty || is_long || has_forbidden_chars {
            Err(format!("{} is not a valid name", s))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::subscriber_name::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_257_grapheme_long_name_is_invalid() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_long_name_is_invalid() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_long_name_is_invalid() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn name_with_forbidden_chars_is_invalid() {
        for name in &["/", "\\", "<", ">", "(", ")", "{", "}", "\""] {
            assert_err!(SubscriberName::parse(name.to_string()));
        }
    }

    #[test]
    fn a_valid_name_is_valid() {
        let name = "Some Name".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
