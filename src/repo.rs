use crate::error::GitError;
use std::path::PathBuf;
use std::{env, fs};

const BLOBS_DIR: &str = "blobs";
const COMMITS_DIR: &str = "commits";
const GIT_DIR: &str = ".git-rs";

pub struct GitRepository {
    pub repo_path: PathBuf,
    pub blobs_path: PathBuf,
    pub commits_path: PathBuf,
}

impl GitRepository {
    pub fn new() -> Self {
        let repo_path = &env::current_dir().unwrap().join(GIT_DIR);
        Self {
            repo_path: repo_path.to_owned(),
            blobs_path: repo_path.join(BLOBS_DIR),
            commits_path: repo_path.join(COMMITS_DIR),
        }
    }
    fn init_repo_dir(path: &PathBuf) -> Result<(), GitError> {
        if !path.exists() {
            match fs::create_dir(path) {
                Ok(_) => Ok(()),
                Err(err) => Err(GitError::GitInitError(format!("{:?}", err))),
            }
        } else if path.is_file() {
            Err(GitError::GitInitError(format!(
                "invalid {} file format",
                path.display()
            )))
        } else {
            Ok(())
        }
    }

    pub fn init(&self) -> Result<(), GitError> {
        Self::init_repo_dir(&self.repo_path)?;
        Self::init_repo_dir(&self.blobs_path)?;
        Self::init_repo_dir(&self.commits_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_repo_dir_ut() {
        let tmp_path = &env::current_dir().unwrap().join("temp");
        assert!(GitRepository::init_repo_dir(tmp_path).is_ok());
        assert!(tmp_path.exists());
        assert!(tmp_path.is_dir());
        fs::remove_dir(tmp_path);
    }
}
