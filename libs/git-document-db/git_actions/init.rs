use std::path::PathBuf;

#[cfg(target_os = "linux")]
use git2::{Repository, Signature};
#[cfg(target_os = "android")]
use gix::Repository;

pub fn init(repository_path: &PathBuf, remote_origin_url: &str) -> eyre::Result<Repository> {
    // Create git directory if not exists
    std::fs::create_dir_all(repository_path)?;
    #[cfg(target_os = "linux")]
    let repo = Repository::init(repository_path)?;
    #[cfg(target_os = "linux")]
    let has_origin = repo.remote("origin", remote_origin_url);
    #[cfg(target_os = "android")]
    let repo = gix::open(repository_path)?;
    #[cfg(target_os = "android")]
    let has_origin: eyre::Result<()> = {
        let parsed_url = gix_url::parse(remote_origin_url.as_ref())?;
        repo.remote_at(parsed_url)?;
        Ok(())
    };
    let e: eyre::Result<()> = match has_origin {
        // Remote already exist
        Err(_) => Ok(()),
        Ok(_) => {
            #[cfg(target_os = "linux")]
            {
                // Set up the user for the commit.
                // TODO: get commiter name from env
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
            }
            #[cfg(target_os = "android")]
            {
                //let sig = Signature::now("Your Name", "your_email@example.com")?;
                let index = repo.index_or_empty()?;

                let index_tree = index
                    .tree()
                    .ok_or_else(|| eyre::eyre!("couldn't find index tree"))?;

                repo.commit(
                    "HEAD",
                    "Initial commit",
                    index_tree.id,
                    Vec::<gix::ObjectId>::new().into_iter(),
                )?;
            }
            Ok(())
        }
    };

    e?;

    Ok(repo)
}

#[cfg(test)]
mod tests {}
