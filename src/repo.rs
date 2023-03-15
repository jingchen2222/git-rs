use crate::error::GitError;
use std::path::PathBuf;
use std::{env, fs};
use std::collections::BTreeMap;
use std::io::{Write};
use crate::utils;
use serde::{Serialize, Deserialize};

const GIT_DIR: &str = ".git-rs";
const BLOBS_DIR: &str = "blobs";
const COMMITS_DIR: &str = "commits";
const STAGED_DIR: &str = "staged";
const STAGED_FOR_ADD: &str = "STAGED_ADD";
const HEAD_FILE: &str = "HEAD";

#[derive(Debug, Serialize, Deserialize)]
struct StagedArea {
    blobs: BTreeMap<String, String>
}
impl StagedArea {
    pub fn new() -> Self {
        Self {
            blobs: BTreeMap::new()
        }
    }

    pub fn add(&mut self, path: String, hash: String) {
        self.blobs.insert(path, hash);
    }

    /// persistence staged area
    /// 1. serialize StageArea into json string
    /// 2. write/update serialized string into staging area file
    pub fn persist(&self, path: &PathBuf) -> Result<(), GitError> {
        let mut file = fs::File::create(&path)
            .map_err(|e| GitError::FileOpError(format!("{:?}",e)))?;
        let content = serde_json::to_string(self)
            .map_err(|e| GitError::SerdeOpError(format!("{:?}",e)))?;
        file.write_all(content.as_bytes())
            .map_err(|e| GitError::FileOpError(format!("{:?}",e)))?;
        Ok(())
    }
}
pub struct GitRepository {
    pub repo_path: PathBuf,
    cwd: PathBuf,
    blobs_path: PathBuf,
    commits_path: PathBuf,
    staged_path: PathBuf,
    head_file: PathBuf,
    staged_for_add_file: PathBuf,
    staged_for_add: StagedArea,
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
            staged_path: repo_path.join(STAGED_DIR),
            head_file: repo_path.join(HEAD_FILE),
            staged_for_add_file: repo_path.join(STAGED_FOR_ADD),
            staged_for_add: StagedArea::new(),
        }
    }

    /// init repository directory including .git, commits, blobs, etc
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
        Self::init_repo_dir(&self.staged_path)?;
        Self::init_repo_file(&self.head_file)?;
        Self::init_repo_file(&self.staged_for_add_file)?;
        Ok(())
    }

    fn init_repo_file(path: &PathBuf) -> Result<(), GitError> {
        if !path.exists() {
            fs::File::create(path)
                .map_err(|e| GitError::FileOpError(format!("{:?}",e)))?;
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
            let relative_path = path.strip_prefix(&self.cwd)
                .map_err(|_| GitError::StagedAddError(format!("file {} is outside repository", path.display())))?;
            // TODO: replace only when file is modified
            // move file to staging area
            utils::copy_to(&path, &self.staged_path.join(&hash))?;
            self.staged_for_add.add(relative_path.display().to_string(), hash);
            self.persist_adding_staged_area()?;
            Ok(())
        } else {
            Err(GitError::FileNotExistError(path.display().to_string()))
        }
    }
    fn persist_adding_staged_area(&self) -> Result<(), GitError> {
        self.staged_for_add.persist(&self.staged_for_add_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_repo() {
        fs::remove_dir_all(&env::current_dir().unwrap().join(GIT_DIR));
    }
    #[test]
    fn init_repo_dir_ut() {
        let tmp_path = &env::current_dir().unwrap().join("init_repo_dir_ut");
        assert!(GitRepository::init_repo_dir(tmp_path).is_ok());
        assert!(tmp_path.exists());
        assert!(tmp_path.is_dir());
        fs::remove_dir(tmp_path);
    }

    #[test]
    fn smoke_ut() {
        let cwd = &env::current_dir().unwrap();
        let smoke_ut_dir= &env::current_dir().unwrap().join("smoke_ut");

        if smoke_ut_dir.exists() {
            fs::remove_dir_all(smoke_ut_dir);
        }

        // prepare dir and files
        fs::create_dir(smoke_ut_dir);
        fs::create_dir(smoke_ut_dir.join("d1"));
        let paths: Vec<PathBuf> = vec!["f1", "f2", "f3", "f4", "f5", "d1/f1", "d1/f2"]
            .iter().map(|f| smoke_ut_dir.join(f)).collect();
        for path in paths.iter() {
            let mut file = fs::File::create(path).unwrap();
            file.write_all(format!("this is a demo content for {}", path.display()).as_bytes());
        }


        clean_repo();
        let git = &mut GitRepository::new();
        assert!(!git.repo_path.exists());
        git.init();
        assert!(git.blobs_path.exists());
        assert!(git.blobs_path.is_dir());
        assert!(git.staged_path.exists());
        assert!(git.staged_path.is_dir());
        assert!(git.commits_path.exists());
        assert!(git.commits_path.is_dir());
        assert!(git.head_file.exists());
        assert!(git.head_file.is_file());
        assert!(git.staged_for_add_file.exists());
        assert!(git.staged_for_add_file.is_file());

        // Act git add f1
        git.add_file(&paths[0]);

        // Verify staging add file
        let mut file = fs::File::open(&git.staged_for_add_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content);
        assert_eq!(r#"{"blobs":{"smoke_ut/f1":"436e9d92cf041816563850964d9256d7b0484c46"}}"#, content.as_str());

        // Act git add f2
        git.add(&vec!["smoke_ut/f2".to_string()]);

        // Verify staging add file
        let mut file = fs::File::open(&git.staged_for_add_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content);
        assert_eq!(r#"{"blobs":{"smoke_ut/f1":"436e9d92cf041816563850964d9256d7b0484c46","smoke_ut/f2":"edf058309c9c35b69458bc469344d7e7f9906ac2"}}"#, content.as_str());
        clean_repo();
        fs::remove_dir_all(smoke_ut_dir);
    }

    #[test]
    fn staged_area_serialized_deserialized_ut() {
        let area = StagedArea { blobs : BTreeMap::from(
            [("file1".to_string(), "hash1".to_string()),
            ("file2".to_string(), "hash2".to_string())]) };

        let serialized = serde_json::to_string(&area).unwrap();
        assert_eq!(r#"{"blobs":{"file1":"hash1","file2":"hash2"}}"#, serialized);

        let deserialized: StagedArea = serde_json::from_str(&serialized).unwrap();
        assert_eq!(2, deserialized.blobs.len());
        assert_eq!("hash1", deserialized.blobs.get("file1").unwrap().as_str());
        assert_eq!("hash2", deserialized.blobs.get("file2").unwrap().as_str());
    }
    #[test]
    fn staged_area_serialized_deserialized_empty_map_ut() {
        let area = StagedArea { blobs : BTreeMap::new()};

        let serialized = serde_json::to_string(&area).unwrap();
        assert_eq!(r#"{"blobs":{}}"#, serialized);

        let deserialized: StagedArea = serde_json::from_str(&serialized).unwrap();
        assert_eq!(0, deserialized.blobs.len());
    }

    #[test]
    fn staged_area_persist_ut() {
        let tmp_dir = &env::current_dir().unwrap().join("staged_area_persist_ut");
        fs::create_dir_all(tmp_dir);

        let tmp_file = tmp_dir.join("area");

        let area = StagedArea { blobs : BTreeMap::from(
            [("file1".to_string(), "hash1".to_string()),
                ("file2".to_string(), "hash2".to_string())]) };
        let res = area.persist(&tmp_file);
        assert!(res.is_ok(), "{:?}", res);

        let mut file = fs::File::open(&tmp_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content);

        assert_eq!(r#"{"blobs":{"file1":"hash1","file2":"hash2"}}"#, content.as_str());
        fs::remove_file(&tmp_file);
        fs::remove_dir(&tmp_dir);
    }

}
