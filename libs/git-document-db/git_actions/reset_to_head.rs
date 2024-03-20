use crate::git_actions::stage_and_commit_changes::find_last_commit;

pub fn reset_to_head(repo: &git2::Repository, glob: &[&str]) -> eyre::Result<()> {
    // Add all unstaged files to index
    let mut index = repo.index()?;
    index.add_all(glob.iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Get the tree from the HEAD commit
    let head_commit = find_last_commit(repo)?;
    let head_tree = head_commit.tree()?;

    // Perform a checkout to update the working directory and index to match the HEAD tree
    // Create options for the checkout process
    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    checkout_builder.force();

    // Use the checkout_tree function to apply the changes to the working directory
    let obj = head_tree.as_object();
    repo.checkout_tree(obj, Some(&mut checkout_builder))?;

    Ok(())
}
