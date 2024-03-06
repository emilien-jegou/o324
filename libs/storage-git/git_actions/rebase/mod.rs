mod diff;
mod rebase;
mod rebase_iterator;
mod rebase_operation;

pub use rebase::Rebase;
pub use rebase_iterator::RebaseIterator;
pub use rebase_operation::RebaseOperation;

pub fn ensure_origin_main_exists(repo: &git2::Repository) -> Result<(), git2::Error> {
    match repo.find_reference("refs/remotes/origin/main") {
        Ok(_) => Ok(()), // `origin/main` exists, no action needed
        Err(_) => {
            // `origin/main` does not exist, attempt to create it
            // First, ensure there's a local `main` branch to push from.
            // This part is highly context-dependent and might need adjustments
            match repo.find_branch("main", git2::BranchType::Local) {
                Ok(branch) => branch,
                Err(_) => {
                    // Create a new local `main` branch pointing to HEAD or another commit as needed
                    let head = repo.head()?.peel_to_commit()?;
                    repo.branch("main", &head, false)?;
                    repo.find_branch("main", git2::BranchType::Local)?
                }
            };

            // Push the local `main` branch to `origin` as `main`
            let mut remote = repo.find_remote("origin")?;
            remote.push(&[&format!("refs/heads/main:refs/heads/main")], None)?;

            Ok(())
        }
    }
}

// TODO: right now we always rebase onto main, this should be changed, we should allow users to
// set a different branch if they feel like it.
pub fn rebase_current_branch(repo: &git2::Repository) -> Result<Rebase<'_>, git2::Error> {
    // Ensure `origin/main` exists or is created
    ensure_origin_main_exists(repo)?;

    // Step 1: Find the remote branch's commit to rebase onto
    let remote_branch_ref = repo.find_reference("refs/remotes/origin/main")?;
    let upstream_commit = repo.reference_to_annotated_commit(&remote_branch_ref)?;

    // Step 2: Identify the current branch's latest commit
    let head = repo.head()?;
    let head_commit = repo.reference_to_annotated_commit(&head)?;

    // Step 3: Set up and perform the rebase
    let mut rebase_options = git2::RebaseOptions::new();

    let rebase = repo.rebase(
        Some(&head_commit),
        Some(&upstream_commit),
        None,
        Some(&mut rebase_options),
    )?;

    Ok(Rebase::new(repo, rebase))
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::{
        assert_branch_eq_json, git_actions,
        utils::test_utilities::{add_commit_on_head, create_repository_test_setup},
    };
    use sugars::hmap;

    use super::rebase_current_branch;

    #[test]
    fn test_rebase_current_branch_1() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        add_commit_on_head(&origin, "NEW", hmap![]).unwrap();

        git_actions::fetch(&local1).unwrap();

        // Verify current repository status
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["INIT"]);
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Remote,
            "origin/main",
            ["NEW", "INIT"]
        );

        let mut rebase = rebase_current_branch(&local1).unwrap();

        // The rebase should have no conflict
        assert!(rebase.iter().next().is_none());

        rebase.finalize().unwrap();

        // Verify current repository status
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["NEW", "INIT"]);
    }

    #[test]
    fn test_rebase_current_branch_2() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        // This tests verify non conflicted changes
        add_commit_on_head(&origin, "REMOTE", hmap![ "Foo" => "Bar" ]).unwrap();
        add_commit_on_head(&local1, "LOCAL", hmap![ "Hello" => "world" ]).unwrap();

        git_actions::fetch(&local1).unwrap();

        // Verify current repository status
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["LOCAL", "INIT"]);
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Remote,
            "origin/main",
            ["REMOTE", "INIT"]
        );

        let mut rebase = rebase_current_branch(&local1).unwrap();

        // only step of the rebase is the conflict
        let mut iter = rebase.iter();
        let op = iter.next().unwrap();
        let mut operation = op.unwrap();
        let conflict = operation.get_conflict().unwrap();

        // No conflict are expected, changes affect different files
        assert_eq!(conflict.files.len(), 0);

        // Works even when no changes need applying
        conflict.write_changes().unwrap().stage_all().unwrap();
        operation.commit_changes().unwrap();

        // no more conflict
        assert!(iter.next().is_none());

        rebase.finalize().unwrap();

        // Verify current repository status
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Local,
            "main",
            ["LOCAL", "REMOTE", "INIT"]
        );
    }

    #[test]
    fn test_rebase_current_branch_previous_file_content_1() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        // This tests verify non conflicted changes
        add_commit_on_head(&origin, "REMOTE 1", hmap![ "Hello" => "World" ]).unwrap();
        add_commit_on_head(&origin, "REMOTE 2", hmap![]).unwrap();
        add_commit_on_head(&origin, "REMOTE 3", hmap![ "Hello" => "Goodbye" ]).unwrap();
        add_commit_on_head(
            &local1,
            "LOCAL",
            hmap![ "Hello" => "Darkness, My Old Friend", ],
        )
        .unwrap();

        git_actions::fetch(&local1).unwrap();

        let mut rebase = rebase_current_branch(&local1).unwrap();

        let mut iter = rebase.iter();
        let op = iter.next().unwrap();
        let operation = op.unwrap();
        let mut conflict = operation.get_conflict().unwrap();

        // Only one conflict between commits 'REMOTE 3' and 'LOCAL'
        assert_eq!(conflict.files.len(), 1);
        let file = &mut conflict.files[0];

        // previous commit is 'REMOTE 2' where file 'Hello' contains 'World'
        assert_eq!(file.previous.clone(), Some("World".to_string()));
    }

    #[test]
    fn test_rebase_current_branch_previous_file_content_2() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        // This tests verify non conflicted changes
        add_commit_on_head(&origin, "REMOTE", hmap![ "Hello" => "World" ]).unwrap();
        add_commit_on_head(
            &local1,
            "LOCAL",
            hmap![ "Hello" => "Darkness, My Old Friend", ],
        )
        .unwrap();

        git_actions::fetch(&local1).unwrap();

        let mut rebase = rebase_current_branch(&local1).unwrap();

        let mut iter = rebase.iter();
        let op = iter.next().unwrap();
        let operation = op.unwrap();
        let mut conflict = operation.get_conflict().unwrap();

        // Only one conflict between commits 'REMOTE' and 'LOCAL'
        assert_eq!(conflict.files.len(), 1);
        let file = &mut conflict.files[0];

        // No previous file content, only one commit on remote
        assert_eq!(file.previous.clone(), None);
    }

    #[test]
    fn test_rebase_current_branch_previous_file_content_3() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        // This tests verify non conflicted changes
        add_commit_on_head(&origin, "REMOTE 1", hmap![]).unwrap();
        add_commit_on_head(&origin, "REMOTE 2", hmap![ "Hello" => "World" ]).unwrap();
        add_commit_on_head(
            &local1,
            "LOCAL",
            hmap![ "Hello" => "Darkness, My Old Friend", ],
        )
        .unwrap();

        git_actions::fetch(&local1).unwrap();

        let mut rebase = rebase_current_branch(&local1).unwrap();

        let mut iter = rebase.iter();
        let op = iter.next().unwrap();
        let operation = op.unwrap();
        let mut conflict = operation.get_conflict().unwrap();

        // Only one conflict between commits 'REMOTE 3' and 'LOCAL'
        assert_eq!(conflict.files.len(), 1);
        let file = &mut conflict.files[0];

        // No previous file content, only 'REMOTE 1' doesn't possess file 'Hello'
        assert_eq!(file.previous.clone(), None);
    }

    #[test]
    fn test_rebase_current_branch_multistep() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        // This tests verify non conflicted changes
        add_commit_on_head(&origin, "REMOTE", hmap![ "Hello" => "World" ]).unwrap();
        add_commit_on_head(&local1, "LOCAL 1", hmap![ "Hello" => "Foo", ]).unwrap();
        add_commit_on_head(&local1, "LOCAL 2", hmap![ "Hello" => "Bar", ]).unwrap();

        git_actions::fetch(&local1).unwrap();

        let mut rebase = rebase_current_branch(&local1).unwrap();

        let mut iter = rebase.iter();

        // First conflict: 'LOCAL 1' and 'REMOTE'
        {
            let op = iter.next().unwrap();
            let mut operation = op.unwrap();
            let mut conflict = operation.get_conflict().unwrap();

            // Only one conflict between commits 'REMOTE' and 'LOCAL 1'
            assert_eq!(conflict.files.len(), 1);
            let file = &mut conflict.files[0];

            assert_eq!(file.left, "World");
            assert_eq!(file.right, "Foo");
            assert_eq!(file.previous, None);

            file.resolve("Hey");
            conflict.write_changes().unwrap().stage_all().unwrap();
            operation.commit_changes().unwrap();
        }

        // Second conflict: 'LOCAL 2' and 'REMOTE'
        {
            let op = iter.next().unwrap();
            let mut operation = op.unwrap();
            let mut conflict = operation.get_conflict().unwrap();

            // Only one conflict between commits 'REMOTE' and 'LOCAL 1'
            assert_eq!(conflict.files.len(), 1);
            let file = &mut conflict.files[0];

            assert_eq!(file.left, "Hey");
            assert_eq!(file.right, "Bar");
            assert_eq!(file.previous, Some("World".to_string()));

            file.resolve("Hey 2");
            conflict.write_changes().unwrap().stage_all().unwrap();
            operation.commit_changes().unwrap();
        }

        assert!(iter.next().is_none());
        rebase.finalize().unwrap();
    }

    #[test]
    fn test_rebase_current_branch_ignore_patch_skip() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        // This tests verify non conflicted changes
        add_commit_on_head(&origin, "REMOTE", hmap![ "Hello" => "World" ]).unwrap();
        add_commit_on_head(&local1, "LOCAL 1", hmap![ "Hello" => "Foo", ]).unwrap();
        add_commit_on_head(&local1, "LOCAL 2", hmap![ "Hello" => "Bar", ]).unwrap();

        git_actions::fetch(&local1).unwrap();

        let mut rebase = rebase_current_branch(&local1).unwrap();

        let mut iter = rebase.iter();

        // First conflict: 'LOCAL 1' and 'REMOTE'
        {
            let op = iter.next().unwrap();
            let mut operation = op.unwrap();
            let mut conflict = operation.get_conflict().unwrap();

            assert_eq!(conflict.files.len(), 1);
            let file = &mut conflict.files[0];

            assert_eq!(file.left, "World");
            assert_eq!(file.right, "Foo");
            assert_eq!(file.previous, None);

            file.resolve(&file.left.clone());
            conflict.write_changes().unwrap().stage_all().unwrap();

            // Since the two commit are identical the local
            // commit will be deleted
            operation.commit_changes().unwrap();
        }

        // Second conflict: 'LOCAL 2' and 'REMOTE'
        {
            let op = iter.next().unwrap();
            let mut operation = op.unwrap();
            let mut conflict = operation.get_conflict().unwrap();

            assert_eq!(conflict.files.len(), 1);
            let file = &mut conflict.files[0];

            assert_eq!(file.left, "World");
            assert_eq!(file.right, "Bar");
            assert_eq!(file.previous, None);

            // We choose local changes here
            file.resolve(&file.right.clone());
            conflict.write_changes().unwrap().stage_all().unwrap();
            operation.commit_changes().unwrap();
        }

        assert!(iter.next().is_none());

        rebase.finalize().unwrap();

        // Verify that 'LOCAL 1' has been skipped
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Local,
            "main",
            ["LOCAL 2", "REMOTE", "INIT"]
        );
    }

    #[test]
    fn test_rebase_current_branch_with_conflict() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        add_commit_on_head(&origin, "REMOTE", hmap![ "text.txt" => "remote content" ]).unwrap();
        add_commit_on_head(&local1, "LOCAL", hmap![ "text.txt" => "local content" ]).unwrap();

        git_actions::fetch(&local1).unwrap();

        // Verify current repository status
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["LOCAL", "INIT"]);
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Remote,
            "origin/main",
            ["REMOTE", "INIT"]
        );

        let mut rebase = rebase_current_branch(&local1).unwrap();

        // Go through every step of the rebase
        for op in rebase.iter() {
            let mut operation = op.unwrap();
            let mut conflict = operation.get_conflict().unwrap();

            // Only one conflict on 'text.txt'
            assert_eq!(conflict.files.len(), 1);

            let file = &mut conflict.files[0];

            assert_eq!(file.relative_file_path, "text.txt");
            assert_eq!(file.left, "remote content");
            assert_eq!(file.right, "local content");
            assert_eq!(file.previous, Some("".to_string()));

            // This will replace the content of all conflicted
            // files with the local changes.
            file.resolve(&file.right.clone());

            conflict.write_changes().unwrap().stage_all().unwrap();
            operation.commit_changes().unwrap();
        }

        // Still no change before finalize
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["LOCAL", "INIT"]);

        rebase.finalize().unwrap();

        // Verify that the changes are now applied
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Local,
            "main",
            ["LOCAL", "REMOTE", "INIT"]
        );

        // Verify file content after rebase
        let repo_path = local1.path().parent().unwrap();
        let content = std::fs::read_to_string(repo_path.join("text.txt")).unwrap();
        assert_eq!(content, "local content");
    }

    #[test]
    fn test_rebase_current_branch_unhandled_conflict() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        add_commit_on_head(&origin, "REMOTE", hmap![ "text.txt" => "remote content" ]).unwrap();
        add_commit_on_head(&local1, "LOCAL", hmap![ "text.txt" => "local content" ]).unwrap();

        git_actions::fetch(&local1).unwrap();

        let mut rebase = rebase_current_branch(&local1).unwrap();

        //// Apply each rebase operation
        for op in rebase.iter() {
            let mut operation = op.unwrap();
            let conflict = operation.get_conflict().unwrap();

            // We do not handle the merge conflict properly here...

            // ..so both staging and commit should fail
            assert!(conflict.stage_all().is_err());
            assert!(operation.commit_changes().is_err());
        }

        // TODO
        // We should not abort the rebase due to the failure above or it will overwrite local
        // changes
    }

    #[test]
    fn test_rebase_current_branch_conflict_local_remote_vars() {
        let (_keep, local1, _local2, origin) = create_repository_test_setup().unwrap();

        // This tests verify non conflicted changes
        add_commit_on_head(&origin, "REMOTE", hmap![ "Foo" => "Bar" ]).unwrap();

        // The timestamps for commit dates are accurate to the second
        thread::sleep(Duration::from_secs(1));
        add_commit_on_head(&local1, "LOCAL", hmap![ "Hello" => "world" ]).unwrap();

        git_actions::fetch(&local1).unwrap();

        // Verify current repository status
        assert_branch_eq_json!(local1, git2::BranchType::Local, "main", ["LOCAL", "INIT"]);
        assert_branch_eq_json!(
            local1,
            git2::BranchType::Remote,
            "origin/main",
            ["REMOTE", "INIT"]
        );

        let mut rebase = rebase_current_branch(&local1).unwrap();

        // only step of the rebase is the conflict
        let mut iter = rebase.iter();
        let op = iter.next().unwrap();
        let operation = op.unwrap();
        let conflict = operation.get_conflict().unwrap();

        // No conflict are expected, changes affect different files
        assert_eq!(conflict.left_commit.summary, Some("REMOTE".to_string()));
        assert_eq!(conflict.right_commit.summary, Some("LOCAL".to_string()));
    }
}
