use crate::error::GitError;
use std::path::PathBuf;
use std::{env, fs};

pub struct GitRepository {
    pub repo_dir: PathBuf,
    pub objects_dir: PathBuf,
}

impl GitRepository {
    pub fn new() -> Self {
        let repo_dir = env::current_dir().unwrap().join(".git-rs");
        Self {
            repo_dir: repo_dir.clone(),
            objects_dir: repo_dir.clone().join("objects"),
        }
    }
    fn init_repo(&self) -> Result<(), GitError> {
        if !self.repo_dir.exists() {
            match fs::create_dir(&self.repo_dir) {
                Ok(_) => Ok(()),
                Err(err) => Err(GitError::GitInitError(format!("{:?}", err))),
            }
        } else if self.repo_dir.is_file() {
            Err(GitError::GitInitError(format!(
                "invalid {} file format",
                self.repo_dir.display()
            )))
        } else {
            Ok(())
        }
    }
    fn init_objects(&self) -> Result<(), GitError> {
        if !self.objects_dir.exists() {
            match fs::create_dir(&self.objects_dir) {
                Ok(_) => Ok(()),
                Err(err) => Err(GitError::GitInitError(format!("{:?}", err))),
            }
        } else if self.objects_dir.is_file() {
            Err(GitError::GitInitError(format!(
                "invalid {} file format",
                self.objects_dir.display()
            )))
        } else {
            Ok(())
        }
    }
    pub fn init(&self) -> Result<(), GitError> {
        self.init_repo()?;
        self.init_objects()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
