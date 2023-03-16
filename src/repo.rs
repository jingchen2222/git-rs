use crate::error::GitError;
use crate::utils;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

const GIT_DIR: &str = ".git-rs";
const BLOBS_DIR: &str = "blobs";
const COMMITS_DIR: &str = "commits";
const INDEX_FILE: &str = "index";
const HEAD_FILE: &str = "HEAD";
const HEADS_DIR: &str = "refs/heads";

#[derive(Debug, Serialize, Deserialize)]
struct StagingArea {
    staged: BTreeMap<String, String>,
    deleted: Vec<String>,
}
impl StagingArea {
    pub fn new() -> Self {
        Self {
            staged: BTreeMap::new(),
            deleted: Vec::new(),
        }
    }

    /// staged file path --> file sha1 pair
    pub fn add(&mut self, path: String, hash: String) {
        self.staged.insert(path, hash);
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct CommitMeta {
    message: String,
    date_time: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    meta: CommitMeta,
    blobs: BTreeMap<String, String>,
}
pub struct GitRepository {
    pub repo_path: PathBuf,
    cwd: PathBuf,
    blobs_path: PathBuf,
    commits_path: PathBuf,
    head_file: PathBuf,
    index_file: PathBuf,
    heads_path: PathBuf,
    staging_area: StagingArea,
}

impl GitRepository {
    pub fn new() -> Self {
        let cwd = &env::current_dir().unwrap();
        let repo_path = &cwd.join(GIT_DIR);
        Self {
            cwd: cwd.to_owned(),
            repo_path: repo_path.to_owned(),
            blobs_path: repo_path.join(BLOBS_DIR),
            commits_path: repo_path.join(COMMITS_DIR),
            head_file: repo_path.join(HEAD_FILE),
            index_file: repo_path.join(INDEX_FILE),
            heads_path: repo_path.join(HEADS_DIR),
            staging_area: StagingArea::new(),
        }
    }

    /// init repository directory including .git, commits, blobs, etc
    fn init_repo_dir(path: &PathBuf) -> Result<(), GitError> {
        if !path.exists() {
            match fs::create_dir_all(path) {
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
        Self::init_repo_dir(&self.heads_path)?;
        Self::init_repo_file(&self.head_file)?;
        Self::init_repo_file(&self.index_file)?;
        Ok(())
    }

    fn init_repo_file(path: &PathBuf) -> Result<(), GitError> {
        if !path.exists() {
            fs::File::create(path).map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        }
        Ok(())
    }

    pub fn add(&mut self, paths: &Vec<String>) -> Result<(), GitError> {
        for path in paths.iter() {
            self.add_file(&self.cwd.join(&path))?
        }
        Ok(())
    }

    /// add file under path into staging area
    /// 1. check if added file has been modified
    fn add_file(&mut self, path: &PathBuf) -> Result<(), GitError> {
        if path.exists() {
            let hash = utils::crypto_file(path)?;
            let relative_path = path.strip_prefix(&self.cwd).map_err(|_| {
                GitError::StagedAddError(format!("file {} is outside repository", path.display()))
            })?;
            // TODO: replace only when file is modified
            // move file to staging area
            utils::copy_to(&path, &self.blobs_path.join(&hash))?;
            self.staging_area
                .add(relative_path.display().to_string(), hash);
            self.persist_staging_area()?;
            Ok(())
        } else {
            Err(GitError::FileNotExistError(path.display().to_string()))
        }
    }

    /// persist staging area info into index file
    fn persist_staging_area(&self) -> Result<(), GitError> {
        Self::persist(&self.staging_area, &self.index_file)
    }

    /// persistence staged area
    /// 1. serialize StageArea into json string
    /// 2. write/update serialized string into staging area file
    fn persist<T: Serialize>(value: &T, path: &PathBuf) -> Result<(), GitError> {
        let mut file =
            fs::File::create(&path).map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        let content =
            serde_json::to_string(value).map_err(|e| GitError::SerdeOpError(format!("{:?}", e)))?;
        file.write_all(content.as_bytes())
            .map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn clean_repo() {
        let path = &env::current_dir().unwrap().join(GIT_DIR);
        if path.exists() {
            assert!(fs::remove_dir_all(path).is_ok());
        }
    }
    #[test]
    fn init_repo_dir_ut() {
        let tmp_path = &env::current_dir().unwrap().join("init_repo_dir_ut");
        assert!(GitRepository::init_repo_dir(tmp_path).is_ok());
        assert!(tmp_path.exists());
        assert!(tmp_path.is_dir());
        assert!(fs::remove_dir(tmp_path).is_ok());
    }

    #[test]
    fn smoke_ut() {
        let smoke_ut_dir = &env::current_dir().unwrap().join("smoke_ut");

        if smoke_ut_dir.exists() {
            assert!(fs::remove_dir_all(smoke_ut_dir).is_ok());
        }

        // prepare dir and files
        assert!(fs::create_dir(smoke_ut_dir).is_ok());
        assert!(fs::create_dir(smoke_ut_dir.join("d1")).is_ok());
        let paths: Vec<PathBuf> = vec!["f1", "f2", "f3", "f4", "f5", "d1/f1", "d1/f2"]
            .iter()
            .map(|f| smoke_ut_dir.join(f))
            .collect();
        for path in paths.iter() {
            let mut file = fs::File::create(path).unwrap();
            assert!(file
                .write_all(format!("this is a demo content for {}", path.display()).as_bytes())
                .is_ok());
        }

        clean_repo();
        let git = &mut GitRepository::new();
        assert!(!git.repo_path.exists());

        assert!(git.init().is_ok());

        assert!(git.repo_path.exists());
        assert!(git.repo_path.is_dir());
        assert!(git.blobs_path.exists());
        assert!(git.blobs_path.is_dir());
        assert!(git.commits_path.exists());
        assert!(git.commits_path.is_dir());
        assert!(git.heads_path.exists());
        assert!(git.heads_path.is_dir());
        assert!(git.head_file.exists());
        assert!(git.head_file.is_file());
        assert!(git.index_file.exists());
        assert!(git.index_file.is_file());

        // Act git add f1
        assert!(git.add_file(&paths[0]).is_ok());

        // Verify staging add file
        let mut file = fs::File::open(&git.index_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());
        assert_eq!(
            r#"{"staged":{"smoke_ut/f1":"436e9d92cf041816563850964d9256d7b0484c46"},"deleted":[]}"#,
            content.as_str()
        );

        // Act git add f2
        assert!(git.add(&vec!["smoke_ut/f2".to_string()]).is_ok());

        // Verify staging add file
        let mut file = fs::File::open(&git.index_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());
        assert_eq!(
            r#"{"staged":{"smoke_ut/f1":"436e9d92cf041816563850964d9256d7b0484c46","smoke_ut/f2":"edf058309c9c35b69458bc469344d7e7f9906ac2"},"deleted":[]}"#,
            content.as_str()
        );
        clean_repo();
        assert!(fs::remove_dir_all(smoke_ut_dir).is_ok());
    }

    #[test]
    fn staged_area_serialized_deserialized_ut() {
        let area = StagingArea {
            staged: BTreeMap::from([
                ("file1".to_string(), "hash1".to_string()),
                ("file2".to_string(), "hash2".to_string()),
            ]),
            deleted: Vec::new(),
        };

        let serialized = serde_json::to_string(&area).unwrap();
        assert_eq!(
            r#"{"staged":{"file1":"hash1","file2":"hash2"},"deleted":[]}"#,
            serialized
        );

        let deserialized: StagingArea = serde_json::from_str(&serialized).unwrap();
        assert_eq!(2, deserialized.staged.len());
        assert_eq!("hash1", deserialized.staged.get("file1").unwrap().as_str());
        assert_eq!("hash2", deserialized.staged.get("file2").unwrap().as_str());
    }
    #[test]
    fn staged_area_serialized_deserialized_empty_map_ut() {
        let area = StagingArea::new();

        let serialized = serde_json::to_string(&area).unwrap();
        assert_eq!(r#"{"staged":{},"deleted":[]}"#, serialized);

        let deserialized: StagingArea = serde_json::from_str(&serialized).unwrap();
        assert_eq!(0, deserialized.staged.len());
    }

    #[test]
    fn persist_staging_area_ut() {
        let tmp_dir = &env::current_dir().unwrap().join("persist_staging_area_ut");
        assert!(fs::create_dir_all(tmp_dir).is_ok());

        let tmp_file = tmp_dir.join("area");

        let area = StagingArea {
            staged: BTreeMap::from([
                ("file1".to_string(), "hash1".to_string()),
                ("file2".to_string(), "hash2".to_string()),
            ]),
            deleted: Vec::new(),
        };
        let res = GitRepository::persist(&area, &tmp_file);
        assert!(res.is_ok(), "{:?}", res);

        let mut file = fs::File::open(&tmp_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());

        assert_eq!(
            r#"{"staged":{"file1":"hash1","file2":"hash2"},"deleted":[]}"#,
            content.as_str()
        );
        assert!(fs::remove_file(&tmp_file).is_ok());
        assert!(fs::remove_dir(&tmp_dir).is_ok());
    }

    #[test]
    fn persist_commit_ut() {
        let tmp_dir = &env::current_dir().unwrap().join("persist_commit_ut");
        assert!(fs::create_dir_all(tmp_dir).is_ok());

        let tmp_file = tmp_dir.join("commit");

        let area = Commit {
            meta: CommitMeta {
                message: "persist commit ut message".to_string(),
                date_time: 1234567890,
            },
            blobs: BTreeMap::from([
                ("file1".to_string(), "hash1".to_string()),
                ("file2".to_string(), "hash2".to_string()),
            ]),
        };
        let res = GitRepository::persist(&area, &tmp_file);
        assert!(res.is_ok(), "{:?}", res);

        let mut file = fs::File::open(&tmp_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());

        assert_eq!(
            r#"{"meta":{"message":"persist commit ut message","date_time":1234567890},"blobs":{"file1":"hash1","file2":"hash2"}}"#,
            content.as_str()
        );
        assert!(fs::remove_file(&tmp_file).is_ok());
        assert!(fs::remove_dir(&tmp_dir).is_ok());
    }
}
