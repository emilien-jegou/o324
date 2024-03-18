fn find_last_commit(repo: &git2::Repository) -> Result<git2::Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

pub fn stage_and_commit_changes(
    repo: &git2::Repository,
    commit_message: &str,
    glob: &[&str],
) -> eyre::Result<()> {
    let mut index = repo.index()?;
    index.add_all(glob.iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Check if there are changes in the index compared to the HEAD commit
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let head_commit = find_last_commit(repo)?;
    let head_tree = head_commit.tree()?;

    // Compare the current tree with the HEAD tree
    if tree.id() == head_tree.id() {
        // No changes to commit, so we simply return Ok
        return Ok(());
    }

    // Prepare signature for commit
    let signature = repo.signature()?;

    // Use HEAD as the parent commit
    let parents = vec![&head_commit];

    // Create the commit
    let _commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &parents,
    )?;

    Ok(())
}
