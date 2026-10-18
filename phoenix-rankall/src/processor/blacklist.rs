use std::collections::HashSet;
use std::path::Path;

use log::{info, warn};

#[derive(Debug, Default)]
pub struct Blacklist {
    pub blocked_post_ids: HashSet<i64>,
    pub blocked_author_ids: HashSet<i64>,
}

impl Blacklist {
    pub fn load(data_dir: &Path) -> Self {
        let blocked_post_ids = read_ids(&data_dir.join("blacklisted_post_ids.txt"));
        let blocked_author_ids = read_ids(&data_dir.join("blacklisted_author_ids.txt"));

        if !blocked_post_ids.is_empty() || !blocked_author_ids.is_empty() {
            info!(
                "blacklist loaded from {}: {} post IDs, {} author IDs",
                data_dir.display(),
                blocked_post_ids.len(),
                blocked_author_ids.len(),
            );
        }

        Self {
            blocked_post_ids,
            blocked_author_ids,
        }
    }

    pub fn is_blocked(&self, post_id: i64, author_id: i64) -> bool {
        self.blocked_post_ids.contains(&post_id) || self.blocked_author_ids.contains(&author_id)
    }

    pub fn is_empty(&self) -> bool {
        self.blocked_post_ids.is_empty() && self.blocked_author_ids.is_empty()
    }
}

fn read_ids(path: &Path) -> HashSet<i64> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };

    let mut ids = HashSet::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.parse::<i64>() {
            Ok(id) => {
                ids.insert(id);
            }
            Err(_) => {
                warn!("skipping invalid ID in {}: {line}", path.display());
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn load_from_files() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("blacklisted_post_ids.txt"),
            "100\n200\n# comment\n\n300\n",
        )
        .unwrap();
        fs::write(dir.path().join("blacklisted_author_ids.txt"), "10\n20\n").unwrap();

        let bl = Blacklist::load(dir.path());
        assert_eq!(bl.blocked_post_ids.len(), 3);
        assert_eq!(bl.blocked_author_ids.len(), 2);
        assert!(bl.blocked_post_ids.contains(&100));
        assert!(bl.blocked_post_ids.contains(&200));
        assert!(bl.blocked_post_ids.contains(&300));
        assert!(bl.blocked_author_ids.contains(&10));
    }

    #[test]
    fn is_blocked_checks_both() {
        let mut bl = Blacklist::default();
        bl.blocked_post_ids.insert(100);
        bl.blocked_author_ids.insert(20);

        assert!(bl.is_blocked(100, 1));
        assert!(bl.is_blocked(999, 20));
        assert!(!bl.is_blocked(999, 1));
    }

    #[test]
    fn empty_when_files_missing() {
        let dir = TempDir::new().unwrap();
        let bl = Blacklist::load(dir.path());
        assert!(bl.is_empty());
    }

    #[test]
    fn skips_invalid_lines() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("blacklisted_post_ids.txt"),
            "100\nnot_a_number\n200\n",
        )
        .unwrap();

        let bl = Blacklist::load(dir.path());
        assert_eq!(bl.blocked_post_ids.len(), 2);
    }
}
