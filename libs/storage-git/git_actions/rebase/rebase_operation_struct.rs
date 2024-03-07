use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use super::diff;

pub struct RebaseOperation<'repo> {
    pub repository: &'repo git2::Repository,
    pub operation: git2::RebaseOperation<'repo>,
    pub rebase: Rc<RefCell<git2::Rebase<'repo>>>,
}

#[derive(Clone)]
pub struct ConflictFile {
    pub repository_path: PathBuf,
    pub relative_file_path: String,
    pub left: String,
    pub right: String,
    pub previous: Option<String>,
}

impl ConflictFile {
    #[cfg(test)]
    pub fn write(&mut self, content: &str) -> Result<(), git2::Error> {
        std::fs::write(self.repository_path.join(&self.relative_file_path), content)
            .map_err(|e| git2::Error::from_str(&format!("{}", e)))?;
        Ok(())
    }
}

pub struct ConflictCommit {
    pub summary: Option<String>,
    pub timestamp: u64,
}

impl ConflictCommit {
    pub fn from_git_commit(commit: &git2::Commit<'_>) -> Self {
        Self {
            summary: commit.summary().map(String::from),
            timestamp: commit.time().seconds() as u64,
        }
    }
}

pub struct Conflict<'repo> {
    repository: &'repo git2::Repository,
    pub files: Vec<ConflictFile>,
    pub left_commit: ConflictCommit,
    pub right_commit: ConflictCommit,
}

impl<'repo> Conflict<'repo> {
    pub fn stage_file(&self, filename: &str) -> Result<&Self, git2::Error> {
        let mut index: git2::Index = self.repository.index()?;
        let path = PathBuf::from(&filename);
        index.add_path(&path)?;
        index.write()?;
        Ok(self)
    }

    pub fn stage_conflicted(&self) -> Result<&Self, git2::Error> {
        let mut index: git2::Index = self.repository.index()?;
        for it in self.files.iter() {
            let path = PathBuf::from(&it.relative_file_path);
            index.add_path(&path)?;
        }
        index.write()?;
        Ok(self)
    }
}

/// Get the previous version of a file from a git repository relative to the current branch.
fn get_previous_file_content(
    repository: &git2::Repository,
    path: &Path,
) -> Result<Option<String>, git2::Error> {
    let head = repository.head()?.peel_to_commit()?;
    let parent = head
        .parents()
        .next()
        .ok_or_else(|| git2::Error::from_str("head has no parents"))?;

    let parent_tree = parent.tree()?;

    match parent_tree.get_path(path) {
        Ok(entry) => {
            let blob = repository.find_blob(entry.id())?;
            Ok(std::str::from_utf8(blob.content()).ok().map(String::from))
        }
        Err(e) => Err(e),
    }
}

impl<'repo> RebaseOperation<'repo> {
    pub fn new(
        repository: &'repo git2::Repository,
        operation: git2::RebaseOperation<'repo>,
        rebase: Rc<RefCell<git2::Rebase<'repo>>>,
    ) -> Self {
        Self {
            repository,
            operation,
            rebase,
        }
    }
    pub fn commit_changes(&mut self) -> Result<(), git2::Error> {
        let committer = git2::Signature::now("Committer Name", "committer@example.com")?;
        if let Err(e) = self.rebase.borrow_mut().commit(None, &committer, None) {
            // We can just skip the commit if it is repetitive
            if e.code() == git2::ErrorCode::Applied {
                return Ok(());
            }

            return Err(e);
        };

        Ok(())
    }

    pub fn handle_pick_conflict(&self) -> Result<Vec<ConflictFile>, git2::Error> {
        let conflicts = self
            .repository
            .index()?
            .conflicts()?
            .map(|conflict_result| {
                let conflict = conflict_result?;

                let relative_path = conflict
                    .our
                    .as_ref()
                    .map(|entry| {
                        std::str::from_utf8(&entry.path)
                            .map_err(|e| git2::Error::from_str(&format!("{:?}", e)))
                            .map(PathBuf::from)
                    })
                    .transpose()?;

                Ok(relative_path)
            })
            .collect::<Result<Vec<Option<PathBuf>>, git2::Error>>()?;

        let mut file_conflicts: Vec<ConflictFile> = Vec::new();
        let repo_path = self
            .repository
            .path()
            .parent()
            .ok_or_else(|| git2::Error::from_str("repository is missing a parent directory"))?;

        // Print paths and add the "ours" side to the index
        conflicts
            .into_iter()
            .flatten()
            .try_for_each(|path| -> Result<(), git2::Error> {
                let content = std::fs::read_to_string(repo_path.join(&path))
                    .map_err(|e| git2::Error::from_str(&e.to_string()))?;

                let (left, right) = diff::extract_diff_from_conflict(&content);

                // Get the content of the file in the previous commit
                let previous: Option<String> =
                    get_previous_file_content(self.repository, &path).unwrap_or(None);

                file_conflicts.push(ConflictFile {
                    repository_path: repo_path.to_path_buf(),
                    relative_file_path: path
                        .to_str()
                        .ok_or_else(|| git2::Error::from_str("path isn\'t utf8"))?
                        .to_string(),
                    left,
                    right,
                    previous,
                });
                Ok(())
            })?;

        Ok(file_conflicts)
    }

    pub fn get_conflict(&self) -> Result<Conflict, git2::Error> {
        let result = match self.operation.kind() {
            Some(git2::RebaseOperationType::Pick) => self.handle_pick_conflict(),
            Some(op) => Err(git2::Error::from_str(&format!(
                "Invalid rebase contains operation of type {:?}",
                op
            ))),
            _ => Err(git2::Error::from_str(
                "Received no-op but expected rebase operation of type 'pick'",
            )),
        }?;

        let upstream_branch = self
            .repository
            .find_branch("origin/main", git2::BranchType::Remote)?;
        let remote_commit = upstream_branch.get().peel_to_commit()?;
        let local_commit = self.repository.find_commit(self.operation.id())?;

        Ok(Conflict {
            repository: self.repository,
            files: result,
            left_commit: ConflictCommit::from_git_commit(&remote_commit),
            right_commit: ConflictCommit::from_git_commit(&local_commit),
        })
    }

    // Print the current operation infos
    #[allow(dead_code)]
    pub fn debug(&self) -> Result<(), git2::Error> {
        let op_type = match self
            .operation
            .kind()
            .ok_or(git2::Error::from_str("Operation invalid"))?
        {
            git2::RebaseOperationType::Pick => "Pick",
            git2::RebaseOperationType::Reword => "Reword",
            git2::RebaseOperationType::Edit => "Edit",
            git2::RebaseOperationType::Squash => "Squash",
            git2::RebaseOperationType::Fixup => "Fixup",
            git2::RebaseOperationType::Exec => "Exec",
        };

        // Convert the commit ID to a string for printing
        let id = self.operation.id();

        // Retrieve the commit from the repository using the commit ID
        let commit = self.repository.find_commit(id)?;

        // Get the commit's message; you may want to use `summary` for a shorter version
        let commit_message = commit.summary().unwrap_or("No commit message");

        println!(
            "Rebase Operation Type: {}, Commit ID: {}, Commit Name: {}",
            op_type, id, commit_message
        );

        Ok(())
    }
}
