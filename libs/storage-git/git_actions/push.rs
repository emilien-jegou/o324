#[allow(dead_code)]
pub fn push(repo: &git2::Repository) -> Result<(), git2::Error> {
    // Configure push options (no authentication needed for local file path remotes)
    let mut push_options = git2::PushOptions::new();

    // Locate the remote named "origin" which points to the fake_origin
    let mut origin = repo.find_remote("origin")?;

    // Push the changes to the configured branch
    // Adjust 'refs/heads/master:refs/heads/master' as necessary for your branch names
    origin.push(
        &["refs/heads/main:refs/heads/main"],
        Some(&mut push_options),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use sugars::hmap;

    use crate::{assert_branch_eq_json, test_utilities::{add_commit_on_head, create_repository_test_setup}};

    use super::push;

    #[test]
    fn test_push_local_changes_to_remote() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        add_commit_on_head(&local1, "NEW", hmap![]).unwrap();

        // Local should now have a new commit
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["NEW", "INIT"]);

        // Verify local & remote state before push
        assert_branch_eq_json!(local1, git2::BranchType::Remote, "origin/main", ["INIT"]);
        assert_branch_eq_json!(origin, git2::BranchType::Local, "main", ["INIT"]);

        push(&local1).unwrap();

        // Remote properly received new commit
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Remote,
            "origin/main",
            ["NEW", "INIT"]
        );
        assert_branch_eq_json!(origin, git2::BranchType::Local, "main", ["NEW", "INIT"]);
    }

    #[test]
    fn test_push_local_changes_to_remote_out_of_date() {
        let (_keep, local1, local2, origin) = create_repository_test_setup().unwrap();

        add_commit_on_head(&local1, "local1", hmap![]).unwrap();
        add_commit_on_head(&local2, "local2", hmap![]).unwrap();

        // first push work since local1 remote is up to date
        push(&local1).unwrap();

        // local2 is now behind remote, push fail
        assert!(push(&local2).is_err());

        // Verify that remote only received local1 commit
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Remote,
            "origin/main",
            ["local1", "INIT"]
        );
        assert_branch_eq_json!(origin, git2::BranchType::Local, "main", ["local1", "INIT"]);

        // local2 remote is still out of date due to git push failure
        assert_branch_eq_json!(local2, git2::BranchType::Remote, "origin/main", ["INIT"]);
    }
}
