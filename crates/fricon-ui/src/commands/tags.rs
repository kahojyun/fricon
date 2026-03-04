pub(in crate::commands) fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut unique = std::collections::BTreeSet::new();
    for tag in tags {
        let trimmed = tag.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_string());
        }
    }
    unique.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::normalize_tags;

    #[test]
    fn normalize_tags_trims_dedupes_and_sorts() {
        let input = vec![
            " beta".to_string(),
            "alpha".to_string(),
            "alpha".to_string(),
            String::new(),
            "  ".to_string(),
            "gamma".to_string(),
            "beta".to_string(),
        ];

        let normalized = normalize_tags(input);

        assert_eq!(
            normalized,
            vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
        );
    }
}
