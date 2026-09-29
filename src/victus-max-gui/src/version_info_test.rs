#[cfg(test)]
mod tests {
    #[test]
    fn test_git_metadata_present() {
        let hash = env!("VICTUS_MAX_GIT_HASH");
        assert!(!hash.is_empty(), "VICTUS_MAX_GIT_HASH must not be empty");
        assert!(
            hash.len() >= 7 || hash == "unknown",
            "Commit hash should be at least 7 characters or 'unknown'"
        );

        let build_date = env!("VICTUS_MAX_BUILD_DATE");
        assert!(!build_date.is_empty(), "VICTUS_MAX_BUILD_DATE must not be empty");
    }
}
