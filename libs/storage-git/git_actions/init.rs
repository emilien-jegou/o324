use std::path::PathBuf;

use git2::Signature;

use crate::utils::files;

pub fn init(repository_path: &PathBuf, remote_origin_url: &str) -> eyre::Result<()> {
    files::create_dir_if_not_exists_deep(repository_path)?;
    let repo = git2::Repository::init(repository_path)?;
    repo.remote("origin", remote_origin_url)?;

    // Set up the user for the commit.
    let sig = Signature::now("Your Name", "your_email@example.com")?;

    // Create an initial empty tree.
    let tree_id = {
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        repo.find_tree(tree_id)?
    };

    // Create an empty commit using the empty tree.
    // Note: As this is an initial commit, there are no parents.
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree_id, &[])?;

    Ok(())
}
