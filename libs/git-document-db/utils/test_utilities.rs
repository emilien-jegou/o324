use rand::{distributions::Alphanumeric, Rng};
use serde::{Serialize, Serializer};
use tempfile::TempDir;

#[macro_export]
macro_rules! assert_value_eq_json {
    ($value:expr, $($json:tt)*) => {
        let v: ::serde_json::Value = ::serde_json::json!($($json)*);

        let left = ::serde_json::to_string(&$value).unwrap();
        let right = ::serde_json::to_string(&v).unwrap();

        assert_eq!(left, right, "json are not equals");
    };
}

#[macro_export]
#[cfg(target_os = "linux")]
macro_rules! assert_branches_eq_json {
    ($remote:expr, $branch_type:expr, $($json:tt)*) => {
        let v = $crate::utils::test_utilities::get_branches_commits(
            &$remote,
            $branch_type
        ).unwrap();
        $crate::assert_value_eq_json!(v, $($json)*);
    };
}

#[macro_export]
#[cfg(target_os = "linux")]
macro_rules! assert_branch_eq_json {
    ($remote:expr, $branch_type:expr, $branch_name:expr, $($json:tt)*) => {
        let v = $crate::utils::test_utilities::get_branch_commits(
            &$remote,
            $branch_name,
            $branch_type
        ).unwrap();
        $crate::assert_value_eq_json!(v, $($json)*);
    };
}

#[macro_export]
macro_rules! tasks_vec {
    ($($json:tt)*) => {{
        let val = ::serde_json::json!($($json)*);
        let data: Vec<Task> = ::serde_json::from_value(val).unwrap();
        data
    }};
}

#[cfg(target_os = "linux")]
pub fn get_branch_commits(
    repo: &git2::Repository,
    branch_name: &str,
    branch_type: git2::BranchType,
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

#[cfg(target_os = "linux")]
pub fn get_branches_commits(
    repo: &git2::Repository,
    branch_type: git2::BranchType,
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

pub fn random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let s: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(len)
        .collect();
    s
}

#[derive(PartialEq, Debug, serde_derive::Deserialize)]
pub struct Commit {
    hash: String,
    name: String,
}

impl Serialize for Commit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the `name` field as a string
        serializer.serialize_str(&self.name)
    }
}

#[derive(PartialEq, Debug, serde_derive::Deserialize, serde_derive::Serialize)]
pub struct Branch {
    branch_name: String,
    commits: Vec<Commit>,
}

#[cfg(target_os = "linux")]
pub fn list_uncommitted_changes(repo: &git2::Repository) -> Result<Vec<String>, git2::Error> {
    let statuses = repo.statuses(Some(
        git2::StatusOptions::new()
            .include_untracked(true)
            .include_ignored(false)
            .show(git2::StatusShow::IndexAndWorkdir),
    ))?;

    let mut uncommited_changes = Vec::new();

    for entry in statuses.iter() {
        if entry.status() != git2::Status::CURRENT {
            if let Some(path) = entry.path() {
                uncommited_changes.push(format!("{:?} {}", entry.status(), path));
            }
        }
    }

    Ok(uncommited_changes)
}

#[cfg(target_os = "linux")]
pub fn add_commit_on_head(
    repo: &git2::Repository,
    commit_name: &str,
    files: HashMap<&str, &str>,
) -> eyre::Result<()> {
    let sig = git2::Signature::now("Unit Test", "unit@example.com")?;

    // Attempt to retrieve the current HEAD commit, handling the case where it does not exist
    let head_result = repo.head();
    let parent_commit = match head_result {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(e) => match e.code() {
            git2::ErrorCode::NotFound => {
                panic!("HEAD reference not found. The repository might be empty or the branch is missing.");
            }
            _ => None,
        },
    };
    // Prepare the tree builder
    let mut tree_builder = match parent_commit {
        Some(ref parent) => {
            // If there's a parent commit, use its tree
            let tree = parent.tree()?;
            repo.treebuilder(Some(&tree))?
        }
        None => {
            // If no parent, start with a new tree builder
            repo.treebuilder(None)?
        }
    };

    for (filename, content) in files.iter() {
        let blob_oid = repo.blob(content.as_bytes())?;
        tree_builder.insert(filename, blob_oid, 0o100644)?;
    }

    // Write the tree to the repository
    let tree_oid = tree_builder.write()?;
    let tree = repo.find_tree(tree_oid)?;

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
    )?;

    // This fail on bare repository
    let _ = repo.checkout_head(Some(git2::CheckoutBuilder::default().force()));
    Ok(())
}

#[cfg(target_os = "linux")]
fn force_sync_local_with_origin(repo: &git2::Repository) -> Result<(), git2::Error> {
    // Ensure we are up to date with remote
    let mut remote = repo.find_remote("origin")?;
    remote.fetch(&["+refs/heads/*:refs/remotes/origin/*"], None, None)?;

    // Find the fetched commit from origin/main
    let remote_branch_ref = repo.find_reference("refs/remotes/origin/main")?;
    let commit = remote_branch_ref.peel_to_commit()?;

    // Checkout the local main branch to the commit of origin/main forcefully
    let mut checkout_builder = git2::CheckoutBuilder::new();
    checkout_builder.force();

    repo.branch("main", &commit, false)?;

    repo.checkout_head(Some(&mut checkout_builder))?;
    repo.reset(
        commit.as_object(),
        git2::ResetType::Hard,
        Some(&mut checkout_builder),
    )?;

    Ok(())
}

// A utility function to create a temporary directory and initialize a Git repository
#[cfg(target_os = "linux")]
pub fn create_repository_test_setup() -> eyre::Result<(
    TempDir,
    git2::Repository,
    git2::Repository,
    git2::Repository,
)> {
    let temp = tempdir()?;

    // Create the remote repository
    let origin_repo_path = temp.path().join("origin");
    let origin = git2::Repository::init_bare(&origin_repo_path)?;
    let origin_url = format!(
        "file://{}",
        origin_repo_path
            .to_str()
            .ok_or_else(|| eyre::eyre!("folder must use valid unicode characters"))?
    );

    // Create first local repository
    let local1_path = temp.path().join("local_1");
    let local1 = git2::Repository::init(local1_path)?;
    local1.remote("origin", &origin_url)?;

    // Create second local repository
    let local2_path = temp.path().join("local_2");
    let local2 = git2::Repository::init(local2_path)?;
    local2.remote("origin", &origin_url)?;

    add_commit_on_head(&origin, "INIT", sugars::hmap![ "text.txt" => "" ])?;

    force_sync_local_with_origin(&local1)?;
    force_sync_local_with_origin(&local2)?;

    // Keep tempdir directory in score or it will be destroyed
    Ok((temp, local1, local2, origin))
}

// Prevent tempdir from being deleted and print directory name to stdout
#[allow(dead_code)]
pub fn debug_tempdir(dir: TempDir) {
    let persistent_dir = dir.into_path();

    println!("TEMPDIR: {:?}", persistent_dir);
}

/// Runs a given closure in a separate thread and returns the result.
#[allow(dead_code)]
pub fn thread_runner<F, T>(func: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = func(); // Execute the provided function
        sender
            .send(result)
            .expect("Failed to send result from thread");
    });

    receiver
        .recv()
        .expect("Failed to receive result from thread")
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use crate::utils::test_utilities::create_repository_test_setup;

    #[test]
    fn test_setup() {
        // Initialize three temporary repository, two local and one remote
        let (_keep, local1, local2, origin) = create_repository_test_setup().unwrap();

        // This is the initial state of repositories after creation:

        //// LOCAL 1
        assert_branches_eq_json!(
            local1, git2::BranchType::Local,
            [{"branch_name":"main","commits":["INIT"]}]
        );

        assert_branches_eq_json!(
            local1, git2::BranchType::Remote,
            [{"branch_name":"origin/main","commits":["INIT"]}]
        );

        //// LOCAL 2
        assert_branches_eq_json!(
            local2, git2::BranchType::Local,
            [{"branch_name":"main","commits":["INIT"]}]
        );

        assert_branches_eq_json!(
            local2, git2::BranchType::Remote,
            [{"branch_name":"origin/main","commits":["INIT"]}]
        );

        //// ORIGIN

        // Note that the local of the origin directory should be
        // equal to the remote of the local directory when up to
        // date.
        assert_branches_eq_json!(
            origin, git2::BranchType::Local,
            [{"branch_name":"main","commits":["INIT"]}]
        );
    }
}
