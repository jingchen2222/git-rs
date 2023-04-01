use crate::error::GitError;
use crypto;
use crypto::digest::Digest;
use log::info;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
/// crypto file to sha1
/// support text file currently, binary file will be supported in the future
pub fn crypto_file(path: &PathBuf) -> Result<String, GitError> {
    if path.exists() {
        let mut file =
            fs::File::open(&path).map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        let mut s = String::new();
        file.read_to_string(&mut s)
            .map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        Ok(crypto_string(s.as_str()))
    } else {
        Err(GitError::FileNotExistError(path.display().to_string()))
    }
}

/// persistence Serialize object to string
/// e.g serialize StageArea into json string
pub fn sha1<T: Serialize>(value: &T) -> Result<String, GitError> {
    // let mut content = String::new();
    let content =
        serde_json::to_string(&value).map_err(|e| GitError::SerdeOpError(format!("{:?}", e)))?;
    Ok(crypto_string(&content))
}

/// crypto string to sha1
pub fn crypto_string(content: &str) -> String {
    let mut hasher = crypto::sha1::Sha1::new();
    hasher.input_str(content);
    hasher.result_str()
}

/// copy file to repo
/// e.g src/d1/f1 to .git-repo-dir/src/d1/f1
pub fn copy_to(path: &PathBuf, dist: &PathBuf) -> Result<(), GitError> {
    if path.exists() && path.is_file() {
        info!("copy {} to {}", path.display(), dist.display());
        fs::copy(path, dist).map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
        Ok(())
    } else {
        Err(GitError::FileNotExistError(path.display().to_string()))
    }
}

/// visit all files under given directory ans sub directory and return file path vector
fn visit_dirs(
    dir: &PathBuf,
    paths: &mut Vec<PathBuf>,
    ignore: &HashSet<PathBuf>,
) -> Result<(), GitError> {
    if dir.exists() && dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| GitError::FileOpError(format!("{:?}", e)))? {
            let entry = entry.map_err(|e| GitError::FileOpError(format!("{:?}", e)))?;
            let path = entry.path();
            if ignore.contains(&path) {
                continue;
            }
            if path.is_dir() {
                visit_dirs(&path, paths, ignore)?;
            } else {
                paths.push(path);
            }
        }
    }
    Ok(())
}

/// generate file to sha1 map under given directory
pub fn generate_file_sha1_map(
    dir: &PathBuf,
    ignore: &HashSet<PathBuf>,
) -> Result<HashMap<String, String>, GitError> {
    let mut file_sha1_map = HashMap::new();
    if dir.exists() && dir.is_dir() {
        let mut paths = Vec::new();
        visit_dirs(dir, &mut paths, ignore)?;
        for path in paths.iter() {
            let relative_path = path.strip_prefix(dir).unwrap().to_path_buf();
            let sha1 = crypto_file(&path)?;
            file_sha1_map.insert(relative_path.display().to_string(), sha1);
        }
    }
    Ok(file_sha1_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::Commit;
    use std::io::Write;
    use std::{env, fs};

    /// unit test for sha1 Commit object
    #[test]
    fn sha1_commit_ut() {
        let commit = Commit::new();
        let sha1 = sha1(&commit).unwrap();
        assert_eq!("a4afecc02e1a215819ddec84b69e1b51b7b27821", sha1);
    }

    #[test]
    fn crypto_file_ut() {
        let tmp_dir_path = &env::current_dir().unwrap().join("crypto_file_ut");
        if !tmp_dir_path.exists() {
            assert!(fs::create_dir(&tmp_dir_path).is_ok());
        }
        let file_path = tmp_dir_path.join("crypto_file_ut");
        let mut file = fs::File::create(&file_path).unwrap();
        assert!(file
            .write("This is a demo content for crypto_file_ut".as_bytes())
            .is_ok());
        let hash = crypto_file(&file_path).unwrap();
        assert_eq!("2564cf76bd5b1cf65f7b9f52546f1ba7c8accee8", hash);

        if tmp_dir_path.exists() {
            assert!(fs::remove_dir_all(tmp_dir_path).is_ok());
        }
    }

    #[test]
    fn copy_to_ut() {
        let tmp_dir_path = &env::current_dir().unwrap().join("copy_to_ut");
        if !tmp_dir_path.exists() {
            assert!(fs::create_dir(&tmp_dir_path).is_ok());
        }
        let file_path = tmp_dir_path.join("copy_to_ut");
        let mut file = fs::File::create(&file_path).unwrap();
        assert!(file
            .write("This is a demo content for copy_to_ut".as_bytes())
            .is_ok());
        let dist_path = tmp_dir_path.join("copy_to_ut_dist");
        assert!(copy_to(&file_path, &dist_path).is_ok());
        assert!(dist_path.exists());
        assert!(dist_path.is_file());
        if tmp_dir_path.exists() {
            assert!(fs::remove_dir_all(tmp_dir_path).is_ok());
        }
    }

    #[test]
    fn crypto_string_ut() {
        let hash = crypto_string("This is a demo content for crypto_string_ut");
        assert_eq!("cc9eef9cdbe8b198eddf07651446ad9cdf1446f3", hash);
    }

    #[test]
    fn generate_file_sha1_map_ut() {
        let tmp_dir_path = &env::current_dir()
            .unwrap()
            .join("generate_file_sha1_map_ut");
        if tmp_dir_path.exists() {
            assert!(fs::remove_dir_all(tmp_dir_path).is_ok());
        }
        assert!(fs::create_dir(&tmp_dir_path).is_ok());

        for dir in vec!["d1", "d2"] {
            let dir_path = tmp_dir_path.join(dir);
            assert!(fs::create_dir(&dir_path).is_ok());
        }
        for file_name in vec!["f1", "f2", "f3", "d1/f1", "d1/f2", "d2/f1", "d2/f2"] {
            let file_path = tmp_dir_path.join(file_name);
            let mut file = fs::File::create(&file_path).unwrap();
            assert!(file
                .write(format!("This is a demo content for {}", file_path.display()).as_bytes())
                .is_ok());
        }

        let file_sha1_map = generate_file_sha1_map(&tmp_dir_path, &HashSet::new()).unwrap();
        assert_eq!(
            "c5f2b24026cd1db66d21fed90afd80b258438306",
            file_sha1_map.get("f1").unwrap()
        );
        assert_eq!(
            "2d4cc9562e0bbb7d41a885715f88110da36a853a",
            file_sha1_map.get("f2").unwrap()
        );
        assert_eq!(
            "210c5f425360d053925af2fd91539c7b72839d37",
            file_sha1_map.get("f3").unwrap()
        );
        assert_eq!(
            "4a0c297958c72ab87e189c9f0e038d6e5a402b8d",
            file_sha1_map.get("d1/f1").unwrap()
        );
        assert_eq!(
            "5944e3502db357c0fcec506f8ebf1fcf74dd10b3",
            file_sha1_map.get("d1/f2").unwrap()
        );
        assert_eq!(
            "c9dd9a1bfd686e3827d31aca8d283b75035a3cb7",
            file_sha1_map.get("d2/f1").unwrap()
        );
        assert_eq!(
            "04d7bf8ca3f8679f0bd9a80a6dc3b14f55637063",
            file_sha1_map.get("d2/f2").unwrap()
        );

        let file_sha1_map = generate_file_sha1_map(
            &tmp_dir_path,
            &HashSet::from([tmp_dir_path.join("d1"), tmp_dir_path.join("f1")]),
        )
        .unwrap();
        assert!(!file_sha1_map.contains_key("d1/f1"));
        assert!(!file_sha1_map.contains_key("d1/f2"));
        assert_eq!(
            "2d4cc9562e0bbb7d41a885715f88110da36a853a",
            file_sha1_map.get("f2").unwrap()
        );
        assert_eq!(
            "210c5f425360d053925af2fd91539c7b72839d37",
            file_sha1_map.get("f3").unwrap()
        );
        // assert_eq!("4a0c297958c72ab87e189c9f0e038d6e5a402b8d", file_sha1_map.get("d1/f1").unwrap());
        // assert_eq!("5944e3502db357c0fcec506f8ebf1fcf74dd10b3", file_sha1_map.get("d1/f2").unwrap());
        assert_eq!(
            "c9dd9a1bfd686e3827d31aca8d283b75035a3cb7",
            file_sha1_map.get("d2/f1").unwrap()
        );
        assert_eq!(
            "04d7bf8ca3f8679f0bd9a80a6dc3b14f55637063",
            file_sha1_map.get("d2/f2").unwrap()
        );

        if tmp_dir_path.exists() {
            assert!(fs::remove_dir_all(tmp_dir_path).is_ok());
        }
    }
}
