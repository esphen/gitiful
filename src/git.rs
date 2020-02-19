use tempfile::tempdir;
use git2::{TreeWalkMode,TreeWalkResult,Repository};
use std::io;
use regex::Regex;

#[derive(Debug)]
pub struct Entry {
    pub git_ref: String,
    pub time: i64,
    pub count: u32,
}

pub fn count_repo_files(url: &str, range: &str, pattern: &str) -> Result<Vec<Entry>, io::Error> {
    let dir = tempdir()?;
    let pattern = Regex::new(pattern).unwrap();

    let repo = match Repository::clone(url, &dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    let mut walk = repo.revwalk().unwrap();
    walk.push_range(range).unwrap();

    let mut result = Vec::new();

    for git_ref in walk {
        if let Ok(git_ref) = git_ref {
            let commit = repo.find_commit(git_ref).unwrap();
            let tree = commit.tree().unwrap();

            let mut count = 0;
            tree.walk(TreeWalkMode::PreOrder, |_, entry| {
                if let Some(name) = entry.name() {
                    if name.contains(&pattern) {
                        count += 1;
                    }
                }
                TreeWalkResult::Ok
            }).unwrap();

            result.push(Entry {
                git_ref: format!("{}", git_ref),
                time: commit.time().seconds(),
                count: count,
            });
        }
    }


    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_clone() {
        let result = super::count_repo_files(
            "https://github.com/alexcrichton/git2-rs",
            "42ffe50~..a28a870",
            ".rs"
        ).unwrap();

        assert_eq!(result.get(0).unwrap().count, 68);
        assert_eq!(result.get(1).unwrap().count, 68);

        assert_eq!(result.get(0).unwrap().git_ref, "a28a870a074b6d1d20efabb219fc23f3d85a5770");
        assert_eq!(result.get(1).unwrap().git_ref, "42ffe506f70428fc8f362e64e171f8d8dc5c7343");
    }
}
