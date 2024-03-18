use std::{error::Error, path::PathBuf};

use git2::Signature;

pub fn init(
    repository_path: &PathBuf,
    remote_origin_url: &str,
) -> Result<git2::Repository, Box<dyn Error>> {
    // Create git directory if not exists
    std::fs::create_dir_all(repository_path)?;
    let repo = git2::Repository::init(repository_path)?;
    let e: eyre::Result<()> = match repo.remote("origin", remote_origin_url) {
        Err(_) => Ok(()),
        Ok(_) => {
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
    };

    e?;

    Ok(repo)
}
