use crate::error::GitError;
use crate::utils;
use chrono::Utc;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::ops::Add;
use std::path::PathBuf;
use std::{env, fs};

const GIT_DIR: &str = ".git-rs";
const BLOBS_DIR: &str = "blobs";
const COMMITS_DIR: &str = "commits";
const INDEX_FILE: &str = "index";
const HEAD_FILE: &str = "HEAD";
const HEADS_DIR: &str = "refs/heads";
const MAIN_BRANCH: &str = "main";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct StagingArea {
    staged: BTreeMap<String, String>,
    deleted: BTreeMap<String, String>,
}

impl StagingArea {
    pub fn new() -> Self {
        Self {
            staged: BTreeMap::new(),
            deleted: BTreeMap::new(),
        }
    }

    /// staged file path --> file sha1 pair
    pub fn add(&mut self, path: String, hash: String) {
        self.staged.insert(path, hash);
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CommitMeta {
    message: String,
    date_time: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    meta: CommitMeta,
    blobs: BTreeMap<String, String>,
    parent: String,
}

impl Commit {
    pub fn new() -> Self {
        Self {
            meta: CommitMeta {
                message: "".to_string(),
                date_time: 0 as i64,
            },
            blobs: BTreeMap::new(),
            parent: String::new(),
        }
    }
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
    commit: Commit,
    commit_sha1: String,
    branch: String,
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
            commit: Commit::new(),
            commit_sha1: String::new(),
            branch: MAIN_BRANCH.to_string(),
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
        Self::init_repo_file(&self.heads_path.join(MAIN_BRANCH), "")?;
        Self::init_repo_file(
            &self.head_file,
            format!("{}/{}", HEADS_DIR, MAIN_BRANCH).as_str(),
        )?;
        Self::init_repo_file(&self.index_file, "")?;
        Ok(())
    }

    fn init_repo_file(path: &PathBuf, content: &str) -> Result<(), GitError> {
        if !path.exists() {
            let mut file =
                fs::File::create(path).map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
            file.write_all(content.as_bytes())
                .map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        }
        Ok(())
    }

    /// load branch name from HEAD
    fn load_branch(&mut self) -> Result<(), GitError> {
        self.branch = fs::read_to_string(&self.head_file)
            .map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        info!("branch: {}", self.branch);
        Ok(())
    }

    /// load current commit
    fn load_current_commit(&mut self) -> Result<(), GitError> {
        self.commit_sha1 = fs::read_to_string(&self.repo_path.join(&self.branch))
            .map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        info!("current commit: {}", &self.commit_sha1);
        if self.commit_sha1.is_empty() {
            self.commit = Commit::new();
        } else {
            self.commit = Self::unpersist_commit(&self.commits_path.join(&self.commit_sha1))?;
            info!("{:?}", self.commit);
        }
        Ok(())
    }

    /// load staging area from INDEX
    fn load_staging_area(&mut self) -> Result<(), GitError> {
        self.staging_area = Self::unpersist_staging_area(&self.index_file)?;
        Ok(())
    }

    /// load basic information from file.
    /// HEAD, INDEX, commit
    fn load_basic_info(&mut self) -> Result<(), GitError> {
        info!("load basic info");
        self.load_branch()?;
        self.load_current_commit()?;
        self.load_staging_area()?;
        info!("load basic info done!");
        Ok(())
    }

    /// persiste basic git infomation into file
    /// HEAD, INDEX, commit
    fn persist_basic_info(&mut self) -> Result<(), GitError> {
        info!("persist_basic_info");
        Self::persist(&self.staging_area, &self.index_file)?;
        if !&self.commit_sha1.is_empty() {
            Self::persist(&self.commit, &self.commits_path.join(&self.commit_sha1))?;
            fs::write(&self.repo_path.join(&self.branch), &self.commit_sha1)
                .map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        }
        info!("persist_basic_info done!");
        Ok(())
    }
    pub fn add(&mut self, paths: &Vec<String>) -> Result<(), GitError> {
        self.load_basic_info()?;
        for path in paths.iter() {
            self.add_file(&self.cwd.join(&path))?
        }
        self.persist_basic_info()?;
        Ok(())
    }

    pub fn remove(&mut self, paths: &Vec<String>) -> Result<(), GitError> {
        self.load_basic_info()?;
        for path in paths.iter() {
            self.remove_file(&self.cwd.join(&path))?
        }
        self.persist_basic_info()?;
        Ok(())
    }

    /// create new commit blobs with parent commit's blobs and staging area info
    fn generate_commit_blobs(
        old_blobs: &BTreeMap<String, String>,
        adding_staged: &StagingArea,
    ) -> Result<BTreeMap<String, String>, GitError> {
        let mut new_blobs = old_blobs.clone();
        for (k, v) in adding_staged.staged.iter() {
            new_blobs.insert(k.to_owned(), v.to_owned());
        }

        for (k, _) in adding_staged.deleted.iter() {
            new_blobs.remove(k);
        }
        info!("new_blobs: {:?}", &new_blobs);
        Ok(new_blobs)
    }

    /// commit
    pub fn commit(&mut self, msg: &str) -> Result<(), GitError> {
        self.load_basic_info()?;
        info!("commit start...");
        for (removed_path, _) in self.staging_area.deleted.iter() {
            fs::remove_file(&self.cwd.join(removed_path)).map_err(|_| {
                GitError::CommitError("fail to remove file from current workspace".to_string())
            })?;
        }
        let blobs = Self::generate_commit_blobs(&self.commit.blobs, &self.staging_area)
            .map_err(|e| GitError::CommitError(format!("{:?}", e)))?;
        self.staging_area = StagingArea::new();
        self.commit = Commit {
            meta: CommitMeta {
                message: msg.to_string(),
                date_time: Utc::now().timestamp(),
            },
            blobs,
            parent: self.commit_sha1.clone(),
        };
        let content = Self::persist_string(&self.commit)?;
        self.commit_sha1 = utils::crypto_string(content.as_str());
        self.persist_basic_info()?;
        Ok(())
    }

    /// Displays Untracked Files
    fn untrack_status(&self) -> Result<String, GitError> {
        Ok("=== Untracked Files ===".to_lowercase())
    }
    /// Displays what files have been modified by not Staged For Commit
    fn modified_not_staged(&self) -> Result<String, GitError> {
        Ok("=== Modifications Not Staged For Commit ===".to_lowercase())
    }
    /// Displays what files have been staged for addition
    fn staged_status(&self) -> Result<String, GitError> {
        let mut msg: Vec<String> = vec![];
        msg.push("=== Staged Files ===".to_string());
        for (k, _) in self.staging_area.staged.iter() {
            msg.push(k.clone());
        }
        Ok(msg.join("\n"))
    }
    /// Displays what files have been staged for removal.
    fn removal_status(&self) -> Result<String, GitError> {
        let mut msg: Vec<String> = vec![];
        msg.push("=== Removed Files ===".to_string());
        for (k, _) in self.staging_area.deleted.iter() {
            msg.push(k.clone());
        }
        Ok(msg.join("\n"))
    }

    /// Displays what branches currently exist, and marks the current branch with a *.
    fn branch_status(&self) -> Result<String, GitError> {
        let mut msg: Vec<String> = vec![];

        msg.push("=== Branches ===".to_string());

        let current_branch_path = self.repo_path.join(&self.branch);
        let current_branch_name = current_branch_path
            .strip_prefix(&self.heads_path)
            .map_err(|_| GitError::BranchError("invalid branch name".to_string()))?;
        msg.push(format!("*{}", current_branch_name.display()));
        for entry in
            fs::read_dir(&self.heads_path).map_err(|e| GitError::BranchError(format!("{:?}", e)))?
        {
            let path = entry
                .map_err(|_| GitError::BranchError("invalid branch name".to_lowercase()))?
                .path();
            let branch_name = path
                .strip_prefix(&self.heads_path)
                .map_err(|_| GitError::BranchError("invalid branch name".to_string()))?;

            info!("{:?}", branch_name.display());
            if current_branch_name != branch_name {
                msg.push(branch_name.display().to_string());
            }
        }
        Ok(msg.join("\n"))
    }

    /// Displays what branches currently exist, and marks the current branch with a *.
    /// Also displays what files have been staged for addition or removal. An example of the exact
    /// format it should follow is as follows.
    pub fn status(&mut self) -> Result<String, GitError> {
        info!("status >> ");
        self.load_basic_info();
        let mut msg: Vec<String> = vec![];
        msg.push(self.branch_status()?);
        msg.push(self.staged_status()?);
        msg.push(self.removal_status()?);
        msg.push(self.modified_not_staged()?);
        msg.push(self.untrack_status()?);
        info!("status << ");
        Ok(msg.join("\n\n"))
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

            Ok(())
        } else {
            Err(GitError::FileNotExistError(path.display().to_string()))
        }
    }

    /// remove file
    /// 1. Unstage the file if it is currently staged for addition.
    /// 2. If the file is tracked in the current commit, stage it for removal and remove the file from the working directory if the user has not already done so (do not remove it unless it is tracked in the current commit).
    fn remove_file(&mut self, path: &PathBuf) -> Result<(), GitError> {
        if path.exists() {
            let relative_path = path.strip_prefix(&self.cwd).map_err(|_| {
                GitError::StagedRemoveError(format!(
                    "file {} is outside repository",
                    path.display()
                ))
            })?;
            let path_name = relative_path.display().to_string();
            if self.staging_area.staged.contains_key(&path_name) {
                self.staging_area.staged.remove(&path_name);
                Ok(())
            } else if self.commit.blobs.contains_key(&path_name) {
                self.staging_area.deleted.insert(path_name, "".to_string());
                Ok(())
            } else {
                Err(GitError::StagedRemoveNoReasonError)
            }
        } else {
            Err(GitError::StagedRemoveNoReasonError)
        }
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
    /// persistence staged area
    /// 1. serialize StageArea into json string
    fn persist_string<T: Serialize>(value: &T) -> Result<String, GitError> {
        // let mut content = String::new();
        let content = serde_json::to_string(&value)
            .map_err(|e| GitError::SerdeOpError(format!("{:?}", e)))?;
        Ok(content)
    }
    fn unpersist_commit(path: &PathBuf) -> Result<Commit, GitError> {
        info!("unpersist_commit {}", path.display());
        if !path.exists() || !path.is_file() {
            info!("{}", path.display());
            Err(GitError::FileNotExistError(path.display().to_string()))
        } else {
            let mut file =
                fs::File::open(path).map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;

            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
            info!("content {}", content);
            let commit =
                serde_json::from_str(content.as_str()).expect("JSON was not well-formatted");
            Ok(commit)
        }
    }
    fn unpersist_staging_area(path: &PathBuf) -> Result<StagingArea, GitError> {
        if !path.exists() || !path.is_file() {
            Err(GitError::FileNotExistError(path.display().to_string()))
        } else {
            let mut file =
                fs::File::open(path).map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;

            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
            if content.is_empty() {
                Ok(StagingArea::new())
            } else {
                let staging_area =
                    serde_json::from_str(content.as_str()).expect("JSON was not well-formatted");
                Ok(staging_area)
            }
        }
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
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
    #[test]
    fn it_works() {
        init();

        info!("This record will be captured by `cargo test`");

        assert_eq!(2, 1 + 1);
    }
    #[test]
    fn init_repo_dir_ut() {
        init();
        let tmp_path = &env::current_dir().unwrap().join("init_repo_dir_ut");
        assert!(GitRepository::init_repo_dir(tmp_path).is_ok());
        assert!(tmp_path.exists());
        assert!(tmp_path.is_dir());
        assert!(fs::remove_dir(tmp_path).is_ok());
    }

    #[test]
    fn smoke_ut() {
        init();
        info!("This record will be captured by `cargo test`");
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
        assert!(git.head_file.is_file());
        assert!(git.index_file.exists());
        assert!(git.index_file.is_file());

        // Act git add f1
        assert_eq!(git.branch, "main");
        assert_eq!(git.commit, Commit::new());
        let res = git.add(&vec!["smoke_ut/f1".to_string()]);
        assert!(res.is_ok(), "{:?}", res.err().unwrap());
        // Verify staging add file
        let mut file = fs::File::open(&git.index_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());
        assert_eq!(
            r#"{"staged":{"smoke_ut/f1":"436e9d92cf041816563850964d9256d7b0484c46"},"deleted":{}}"#,
            content.as_str()
        );

        let res = git.add(&vec!["smoke_ut/f2".to_string(), "smoke_ut/f3".to_string()]);
        // Act git add f2
        assert!(res.is_ok(), "{:?}", res);
        // Verify staging add file
        let mut file = fs::File::open(&git.index_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());
        assert_eq!(
            r#"{"staged":{"smoke_ut/f1":"436e9d92cf041816563850964d9256d7b0484c46","smoke_ut/f2":"edf058309c9c35b69458bc469344d7e7f9906ac2","smoke_ut/f3":"de9c94ac88cae8cd61843b1ccd1339ad507e7f49"},"deleted":{}}"#,
            content.as_str()
        );

        // Act git rm f2
        let res = git.remove(&vec!["smoke_ut/f2".to_string()]);
        assert!(res.is_ok(), "{:?}", res);
        // Verify staging add file
        let mut file = fs::File::open(&git.index_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());
        assert_eq!(
            r#"{"staged":{"smoke_ut/f1":"436e9d92cf041816563850964d9256d7b0484c46","smoke_ut/f3":"de9c94ac88cae8cd61843b1ccd1339ad507e7f49"},"deleted":{}}"#,
            content.as_str()
        );
        let mut git = GitRepository::new();
        git.load_basic_info();
        let res = git.staged_status();
        assert!(res.is_ok(), "{:?}", res);
        assert_eq!(
            r#"=== Staged Files ===
smoke_ut/f1
smoke_ut/f3"#,
            res.unwrap()
        );
        // Act git commit "commit test"
        let res = git.commit("commit test");
        assert!(res.is_ok(), "{:?}", res);
        // Verify staging add file
        // let res = git.load_basic_info();
        // assert!(res.is_ok(), "{:?}", res);
        assert_eq!(
            git.commit.blobs,
            BTreeMap::from([
                (
                    "smoke_ut/f1".to_string(),
                    "436e9d92cf041816563850964d9256d7b0484c46".to_string()
                ),
                (
                    "smoke_ut/f3".to_string(),
                    "de9c94ac88cae8cd61843b1ccd1339ad507e7f49".to_string()
                ),
            ])
        );

        // Act git rm f1
        let res = git.remove(&vec!["smoke_ut/f1".to_string()]);
        assert!(res.is_ok(), "{:?}", res);
        // Verify staging add file
        let mut file = fs::File::open(&git.index_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());
        assert_eq!(
            r#"{"staged":{},"deleted":{"smoke_ut/f1":""}}"#,
            content.as_str()
        );

        let mut git = GitRepository::new();
        git.load_basic_info();
        let res = git.removal_status();
        assert!(res.is_ok(), "{:?}", res);
        assert_eq!(
            r#"=== Removed Files ===
smoke_ut/f1"#,
            res.unwrap()
        );

        // Act git commit "commit test"
        let prev_commit = git.commit_sha1.clone();
        let res = git.commit("commit 2nd");
        assert!(res.is_ok(), "{:?}", res);
        // Verify staging add file
        let mut git = GitRepository::new();
        let res = git.load_basic_info();
        assert!(res.is_ok(), "{:?}", res);
        let commit = &git.commit;
        assert_eq!(
            commit.blobs,
            BTreeMap::from([(
                "smoke_ut/f3".to_string(),
                "de9c94ac88cae8cd61843b1ccd1339ad507e7f49".to_string()
            ),])
        );
        assert_eq!(prev_commit, commit.parent);

        let mut git = GitRepository::new();
        git.load_basic_info();
        let res = git.branch_status();
        assert!(res.is_ok(), "{:?}", res);
        assert_eq!(
            r#"=== Branches ===
*main"#,
            res.unwrap()
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
            deleted: BTreeMap::new(),
        };

        let serialized = serde_json::to_string(&area).unwrap();
        assert_eq!(
            r#"{"staged":{"file1":"hash1","file2":"hash2"},"deleted":{}}"#,
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
        assert_eq!(r#"{"staged":{},"deleted":{}}"#, serialized);

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
            deleted: BTreeMap::new(),
        };
        let res = GitRepository::persist(&area, &tmp_file);
        assert!(res.is_ok(), "{:?}", res);

        let mut file = fs::File::open(&tmp_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());

        assert_eq!(
            r#"{"staged":{"file1":"hash1","file2":"hash2"},"deleted":{}}"#,
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
            parent: "mock_parent".to_string(),
        };
        let res = GitRepository::persist(&area, &tmp_file);
        assert!(res.is_ok(), "{:?}", res);

        let mut file = fs::File::open(&tmp_file).unwrap();
        let mut content = String::new();
        assert!(file.read_to_string(&mut content).is_ok());

        assert_eq!(
            r#"{"meta":{"message":"persist commit ut message","date_time":1234567890},"blobs":{"file1":"hash1","file2":"hash2"},"parent":"mock_parent"}"#,
            content.as_str()
        );
        assert!(fs::remove_file(&tmp_file).is_ok());
        assert!(fs::remove_dir(&tmp_dir).is_ok());
    }

    #[test]
    fn unpersist_staging_area_ut() {
        let tmp_dir = &env::current_dir()
            .unwrap()
            .join("unpersist_staging_area_ut");
        assert!(fs::create_dir_all(tmp_dir).is_ok());

        let tmp_file = tmp_dir.join("area");
        let mut file = fs::File::create(&tmp_file).unwrap();
        assert!(file
            .write_all(r#"{"staged":{"file1":"hash1","file2":"hash2"},"deleted":{}}"#.as_bytes())
            .is_ok());

        let res = GitRepository::unpersist_staging_area(&tmp_file);
        assert!(res.is_ok());
        assert_eq!(
            StagingArea {
                staged: BTreeMap::from([
                    ("file1".to_string(), "hash1".to_string()),
                    ("file2".to_string(), "hash2".to_string()),
                ]),
                deleted: BTreeMap::new(),
            },
            res.unwrap()
        );
        assert!(fs::remove_file(&tmp_file).is_ok());
        assert!(fs::remove_dir(&tmp_dir).is_ok());
    }

    #[test]
    fn unpersist_commit_ut() {
        let tmp_dir = &env::current_dir().unwrap().join("unpersist_commit_ut");
        assert!(fs::create_dir_all(tmp_dir).is_ok());

        let tmp_file = tmp_dir.join("commit");
        let mut file = fs::File::create(&tmp_file).unwrap();
        assert!(file.write_all(r#"{"meta":{"message":"persist commit ut message","date_time":1234567890},"blobs":{"file1":"hash1","file2":"hash2"},"parent":"mock_parent"}"#.as_bytes()).is_ok());

        let res = GitRepository::unpersist_commit(&tmp_file);
        assert!(res.is_ok());
        assert_eq!(
            Commit {
                meta: CommitMeta {
                    message: "persist commit ut message".to_string(),
                    date_time: 1234567890,
                },
                blobs: BTreeMap::from([
                    ("file1".to_string(), "hash1".to_string()),
                    ("file2".to_string(), "hash2".to_string()),
                ]),
                parent: "mock_parent".to_string(),
            },
            res.unwrap()
        );
        assert!(fs::remove_file(&tmp_file).is_ok());
        assert!(fs::remove_dir(&tmp_dir).is_ok());
    }

    #[test]
    fn generate_commit_blobs_ut1() {
        let old = BTreeMap::new();
        let staging_area = StagingArea {
            staged: BTreeMap::from([
                ("file1".to_string(), "hash1".to_string()),
                ("file2".to_string(), "hash2".to_string()),
            ]),
            deleted: BTreeMap::new(),
        };
        let new_blobs = GitRepository::generate_commit_blobs(&old, &staging_area).unwrap();
        assert_eq!(
            BTreeMap::from([
                ("file1".to_string(), "hash1".to_string()),
                ("file2".to_string(), "hash2".to_string()),
            ]),
            new_blobs
        );
    }

    #[test]
    fn generate_commit_blobs_ut2() {
        let old = BTreeMap::from([
            ("file1".to_string(), "hash1".to_string()),
            ("file2".to_string(), "hash2".to_string()),
        ]);
        let staging_area = StagingArea {
            staged: BTreeMap::from([
                ("file3".to_string(), "hash3".to_string()),
                ("file4".to_string(), "hash4".to_string()),
            ]),
            deleted: BTreeMap::new(),
        };
        let new_blobs = GitRepository::generate_commit_blobs(&old, &staging_area).unwrap();
        assert_eq!(
            BTreeMap::from([
                ("file1".to_string(), "hash1".to_string()),
                ("file2".to_string(), "hash2".to_string()),
                ("file3".to_string(), "hash3".to_string()),
                ("file4".to_string(), "hash4".to_string()),
            ]),
            new_blobs
        );
    }
}
