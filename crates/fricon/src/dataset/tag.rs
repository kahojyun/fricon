use std::collections::BTreeSet;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum TagError {
    #[error("tag must not be empty")]
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NormalizedTag(String);

impl NormalizedTag {
    pub(crate) fn parse(value: impl AsRef<str>) -> Result<Self, TagError> {
        let normalized = value.as_ref().trim();
        if normalized.is_empty() {
            return Err(TagError::Empty);
        }
        Ok(Self(normalized.to_string()))
    }

    pub(crate) fn parse_many(values: Vec<String>) -> Vec<Self> {
        let mut tags = BTreeSet::new();
        for value in values {
            let normalized = value.trim();
            if normalized.is_empty() {
                continue;
            }
            tags.insert(Self(normalized.to_string()));
        }
        tags.into_iter().collect()
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NormalizedTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizedTag, TagError};

    #[test]
    fn parse_rejects_blank_tags() {
        let error = NormalizedTag::parse("   ").expect_err("blank tag should be rejected");
        assert_eq!(error, TagError::Empty);
    }

    #[test]
    fn parse_trims_tag_names() -> Result<(), TagError> {
        let tag = NormalizedTag::parse(" vision ")?;
        assert_eq!(tag.as_str(), "vision");
        Ok(())
    }

    #[test]
    fn parse_many_deduplicates_trimmed_tags() {
        let tags = NormalizedTag::parse_many(vec![
            " vision ".to_string(),
            String::new(),
            "audio".to_string(),
            "vision".to_string(),
        ]);
        let normalized: Vec<String> = tags
            .into_iter()
            .map(|tag| tag.as_str().to_string())
            .collect();
        assert_eq!(normalized, vec!["audio".to_string(), "vision".to_string()]);
    }
}
