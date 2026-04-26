//! Repo tests

#[cfg(test)]
mod tests {
    use super::super::GitRepo;

    #[test]
    #[ignore]
    fn test_open_current_repo() {
        let repo = GitRepo::open(".").unwrap();
        let commit = repo.head_commit().unwrap();
        println!("Current HEAD: {} - {}", commit.oid, commit.message);
    }

    #[test]
    #[ignore]
    fn test_walk_tree() {
        let repo = GitRepo::open(".").unwrap();
        let head = repo.head_commit().unwrap();
        let tree = repo.commit_to_tree(head.oid.inner()).unwrap();
        let files = repo.walk_tree(&tree).unwrap();
        println!("Found {} files", files.len());
        for file in files.iter().take(10) {
            println!("  {}", file.path);
        }
    }
}
