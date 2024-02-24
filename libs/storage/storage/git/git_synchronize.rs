use git2::{Branch, BranchType, Rebase, RebaseOptions, Repository};
use std::path::Path;

use super::GitStorageConfig;

//enum FetchResult {
//NoOrigin,
//NoOrigin,
//}

pub async fn sync(config: &GitStorageConfig) -> eyre::Result<()> {
    // Attempt to open the repository
    let full_path = &config.get_git_storage_path()?;
    let repo_path = Path::new(&full_path);
    let repo = Repository::open(repo_path)?;

    git_fetch(&repo)?;

    //let mut rebase = git_rebase_current_branch(&repo)?;

    // Apply each rebase operation in turn
    //     while let Some(op) = rebase.next() {
    //         let _ = op?; // In a real application, you might need to handle each operation (e.g., resolving conflicts)
    //     }
    //
    // Complete the rebase process
    //    rebase.finish(None)?;

    Ok(())
}

fn git_fetch(repo: &Repository) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin")?;
    // Correct the refspec to update remote-tracking branches, not local branches
    remote.fetch(&["+refs/heads/*:refs/remotes/origin/*"], None, None)?;
    Ok(())
}

fn _up_to_date(repo: &Repository) -> Result<bool, git2::Error> {
    // Check if the local repository is up to date with the remote
    let is_up_to_date = repo
        .branches(Some(git2::BranchType::Local))?
        .map(|branch| -> Result<_, git2::Error> {
            let (local_branch, _) = branch?;
            let local_branch_name = local_branch
                .name()?
                .ok_or(git2::Error::from_str("Branch name is not valid UTF-8"))?;
            let local_branch_ref =
                repo.find_reference(&format!("refs/heads/{}", local_branch_name))?;
            let local_commit = local_branch_ref.peel_to_commit()?;

            if let Ok(remote_branch_ref) =
                repo.find_reference(&format!("refs/remotes/origin/{}", local_branch_name))
            {
                let remote_commit = remote_branch_ref.peel_to_commit()?;
                Ok(local_commit.id() == remote_commit.id())
            } else {
                // If the remote branch does not exist, consider it as not up to date
                Ok(false)
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .all(|is_branch_up_to_date| is_branch_up_to_date);

    Ok(is_up_to_date)
}

fn git_rebase_current_branch(repo: &Repository) -> Result<(), git2::Error> {
    // Assuming you want to rebase the current branch onto origin/main

    // Step 1: Find the remote branch's commit to rebase onto
    let remote_branch_ref = repo.find_reference("refs/remotes/origin/main")?;
    let upstream_commit = repo.reference_to_annotated_commit(&remote_branch_ref)?;

    // Step 2: Identify the current branch's latest commit
    let head = repo.head()?;
    let head_commit = repo.reference_to_annotated_commit(&head)?;

    // Step 3: Set up and perform the rebase
    let mut rebase_options = RebaseOptions::new();
    let mut rebase = repo.rebase(
        Some(&head_commit),
        Some(&upstream_commit),
        None,
        Some(&mut rebase_options),
    )?;

    // Apply each rebase operation
    while let Some(op) = rebase.next() {
        op?; // Handle each rebase operation (like applying patches). In real use, you might need to handle conflicts here.
    }

    // Step 4: Finish the rebase
    let signature = repo.signature()?; // Use the repository's default signature
    rebase.finish(Some(&signature))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature};
    use serde::{Deserialize, Serialize};
    use tempfile::{tempdir, TempDir};

    #[derive(PartialEq, Debug, Deserialize, Serialize)]
    struct Commit {
        #[serde(skip)]
        hash: String,
        name: String,
    }

    #[derive(PartialEq, Debug, Deserialize, Serialize)]
    struct Branch {
        branch_name: String,
        commits: Vec<Commit>,
    }

    fn add_commit_on_head(repo: &Repository, commit_name: &str) {
        let sig = Signature::now("Unit Test", "unit@example.com").unwrap();

        // Attempt to retrieve the current HEAD commit, handling the case where it does not exist
        let head_result = repo.head();
        let parent_commit = match head_result {
            Ok(head) => Some(head.peel_to_commit().unwrap()),
            Err(e) => match e.code() {
                git2::ErrorCode::NotFound => {
                    panic!("HEAD reference not found. The repository might be empty or the branch is missing.");
                }
                _ => None,
            },
        };

        // Continue with the rest of the function...
        let tree_id = {
            let mut index = repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            oid
        };
        let tree = repo.find_tree(tree_id).unwrap();

        // Create an array of parents for the commit (in this case, just the current HEAD)
        let parents = match &parent_commit {
            Some(x) => vec![x],
            None => Vec::new(),
        };

        // Create the commit
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            commit_name,
            &tree,
            parents.as_slice(),
        )
        .unwrap();
    }

    fn push(repo: &Repository) -> Result<(), git2::Error> {
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

    macro_rules! assert_eq_json {
        ($o1:expr, $o2:expr) => {
            let v: serde_json::Value = serde_json::from_str($o2).unwrap();
            let minified_json_str = serde_json::to_string(&v).unwrap();

            assert_eq!(serde_json::to_string(&$o1).unwrap(), minified_json_str);
        };
    }

    // A utility function to create a temporary directory and initialize a Git repository
    fn create_repository_test_setup() -> (TempDir, Repository, Repository, Repository) {
        let temp = tempdir().unwrap();

        // Create the remote repository
        let origin_repo_path = temp.path().join("origin");
        let origin = Repository::init_bare(&origin_repo_path).unwrap();
        let origin_url = format!("file://{}", origin_repo_path.to_str().unwrap());

        // Create first local repository
        let local1_path = temp.path().join("local_1");
        let local1 = Repository::init(&local1_path).unwrap();
        local1.remote("origin", &origin_url).unwrap();

        // Create second local repository
        let local2_path = temp.path().join("local_2");
        let local2 = Repository::init(&local2_path).unwrap();
        local2.remote("origin", &origin_url).unwrap();

        add_commit_on_head(&origin, "INIT");

        // Get the new added commit
        git_fetch(&local1).unwrap();
        git_fetch(&local2).unwrap();

        // Create the main branch
        {
            let remote_branch_ref = local1.find_reference("refs/remotes/origin/main").unwrap();
            let commit = local1
                .find_commit(remote_branch_ref.peel_to_commit().unwrap().id())
                .unwrap();
            local1.branch("main", &commit, false).unwrap();
        }

        {
            let remote_branch_ref = local2.find_reference("refs/remotes/origin/main").unwrap();
            let commit = local2
                .find_commit(remote_branch_ref.peel_to_commit().unwrap().id())
                .unwrap();
            local2.branch("main", &commit, false).unwrap();
        }

        // Keep tempdir directory in score or it will be destroyed
        (temp, local1, local2, origin)
    }

    fn get_branch_commits(
        repo: &Repository,
        branch_name: &str,
        branch_type: BranchType,
    ) -> eyre::Result<Vec<Commit>> {
        let branch = repo.find_branch(branch_name, branch_type)?;
        let branch_ref = branch.into_reference();
        let commit = repo.find_commit(
            branch_ref
                .target()
                .ok_or(eyre::eyre!("Branch target not found"))?,
        )?;

        let mut commits = Vec::new();

        let mut current_commit = Some(commit);
        while let Some(commit) = current_commit {
            commits.push(Commit {
                hash: commit.id().to_string(),
                name: commit.summary().unwrap_or("No commit message").to_string(),
            });

            current_commit = commit.parent(0).ok();
        }

        Ok(commits)
    }

    // Modify debug_branches to use Vec<Branch>
    fn get_branches_commits(
        repo: &Repository,
        branch_type: BranchType,
    ) -> eyre::Result<Vec<Branch>> {
        let mut branches_vec = Vec::new();

        let mut remote_branches = repo.branches(Some(branch_type))?;
        for branch in remote_branches.by_ref() {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                let commits = get_branch_commits(repo, name, branch_type)?;
                branches_vec.push(Branch {
                    branch_name: name.to_string(),
                    commits,
                });
            }
        }

        Ok(branches_vec)
    }

    #[test]
    fn test_setup() {
        // Initialize three temporary repository, two local and one remote
        let (_keep, local1, local2, origin) = create_repository_test_setup();

        // This is the initial state of repositories after creation...

        //// LOCAL 1
        assert_eq_json!(
            get_branches_commits(&local1, BranchType::Local).unwrap(),
            r#"[{"branch_name":"main","commits":[{"name":"INIT"}]}]"#
        );

        assert_eq_json!(
            get_branches_commits(&local1, BranchType::Remote).unwrap(),
            r#"[{"branch_name":"origin/main","commits":[{"name":"INIT"}]}]"#
        );

        //// LOCAL 2
        assert_eq_json!(
            get_branches_commits(&local2, BranchType::Local).unwrap(),
            r#"[{"branch_name":"main","commits":[{"name":"INIT"}]}]"#
        );

        assert_eq_json!(
            get_branches_commits(&local2, BranchType::Remote).unwrap(),
            r#"[{"branch_name":"origin/main","commits":[{"name":"INIT"}]}]"#
        );

        //// ORIGIN

        // Note that the local of the origin directory should be
        // equal to the remote of the local directory when local
        // as fetched all the changes is up to date
        assert_eq_json!(
            get_branches_commits(&origin, BranchType::Local).unwrap(),
            r#"[{"branch_name":"main","commits":[{"name":"INIT"}]}]"#
        );
    }

    #[test]
    fn test_fetch() {
        // Initialize a temporary repository
        //let (_keep, local1, local2, origin) = create_repository_test_setup();
    }

    #[test]
    fn test_rebase_current_branch() {
        // Initialize a temporary repository
        //let (_keep, local1, local2, origin) = create_repository_test_setup();
        //git_fetch(&repo).unwrap();

        // Setup for the rebase test - this might involve creating branches and setting their upstreams
        // For simplicity, this setup is not shown here

        // Attempt to rebase the current branch
        //let result = git_rebase_current_branch(&repo).unwrap();
    }
}
