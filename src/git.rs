use git2::{Repository, TreeWalkMode, TreeWalkResult};
use regex::Regex;
use std::collections::HashMap;
use std::io;
use std::iter::Iterator;
use tempfile::tempdir;

#[derive(Default, Debug, Serialize)]
pub struct Count {
    pub pattern: String,
    pub count: u32,
}

impl Count {
    fn new(pattern: String, count: u32) -> Count {
        Count { pattern, count }
    }
}

#[derive(Default, Debug, Serialize)]
pub struct Author {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Default, Debug, Serialize)]
pub struct Commit {
    pub git_ref: String,
    pub timestamp: i64,
    pub author: Author,
    pub count_set: HashMap<String, Count>,
}

impl Commit {
    fn add_pattern_matches(&mut self, pattern: String, count: Count) {
        self.count_set.insert(pattern, count);
    }
}

impl<'a> From<git2::Commit<'a>> for Commit {
    fn from(commit: git2::Commit) -> Self {
        Commit {
            git_ref: format!("{}", commit.id()),
            timestamp: commit.time().seconds(),
            author: Author {
                name: commit.author().name().map(|s| format!("{}", s)),
                email: commit.author().email().map(|s| format!("{}", s)),
            },
            count_set: HashMap::new(),
        }
    }
}

pub fn count_repo_files(
    url: &str,
    range: &str,
    patterns: Vec<&str>,
) -> Result<Vec<Commit>, io::Error> {
    let dir = tempdir()?;
    let patterns: Vec<Regex> = patterns
        .iter()
        .map(|pattern| Regex::new(pattern).unwrap())
        .collect();

    let repo = match Repository::clone(url, &dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    let mut walk = repo.revwalk().unwrap();
    walk.push_range(range).unwrap();

    let mut commit_range = Vec::new();

    for git_ref in walk {
        if let Ok(git_ref) = git_ref {
            let commit = repo.find_commit(git_ref).unwrap();
            let tree = commit.tree().unwrap();

            // Skip merge commits
            if commit.parent_count() > 1 {
                continue;
            }

            let mut commit = Commit::from(commit);

            for pattern in patterns.iter() {
                let mut count = 0;
                tree.walk(TreeWalkMode::PostOrder, |_, entry| {
                    if let Some(name) = entry.name() {
                        if name.contains(pattern) {
                            count += 1;
                        }
                    }
                    TreeWalkResult::Ok
                })
                .unwrap();

                commit.add_pattern_matches(
                    format!("{}", pattern.as_str()),
                    Count::new(format!("{}", pattern.as_str()), count),
                );
            }

            commit_range.push(commit);
        }
    }

    Ok(commit_range)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_clone() {
        let pattern = "\\.rs$";
        let commit_range = super::count_repo_files(
            "https://github.com/alexcrichton/git2-rs",
            "42ffe50~..a28a870",
            vec![pattern],
        )
        .unwrap();

        assert_eq!(
            commit_range
                .get(0)
                .unwrap()
                .count_set
                .get(pattern)
                .unwrap()
                .count,
            68
        );
        assert_eq!(
            commit_range
                .get(1)
                .unwrap()
                .count_set
                .get(pattern)
                .unwrap()
                .count,
            68
        );

        assert_eq!(
            commit_range.get(0).unwrap().git_ref,
            "a28a870a074b6d1d20efabb219fc23f3d85a5770"
        );
        assert_eq!(
            commit_range.get(1).unwrap().git_ref,
            "42ffe506f70428fc8f362e64e171f8d8dc5c7343"
        );
    }
}
