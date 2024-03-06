pub fn fetch(repo: &git2::Repository) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin")?;
    // Correct the refspec to update remote-tracking branches, not local branches
    remote.fetch(&["+refs/heads/*:refs/remotes/origin/*"], None, None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_branch_eq_json,
        utils::test_utilities::{add_commit_on_head, create_repository_test_setup},
    };
    use sugars::hmap;

    #[test]
    fn test_fetch_new_remotes_changes_detected() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        add_commit_on_head(&origin, "NEW", hmap![]).unwrap();

        // Remote should now have a new commit
        assert_branch_eq_json!(origin, git2::BranchType::Local, "main", ["NEW", "INIT"]);

        // Verify local1 state before fetch
        assert_branch_eq_json!(local1, git2::BranchType::Remote, "origin/main", ["INIT"]);
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["INIT"]);

        super::fetch(&local1).unwrap();

        // Remotes commits are now visible to local1
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Remote,
            "origin/main",
            ["NEW", "INIT"]
        );

        // Local still unchanged
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["INIT"]);
    }
}
