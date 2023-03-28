use crate::error::GitError;
use crypto;
use crypto::digest::Digest;
use log::info;
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
        let mut hasher = crypto::sha1::Sha1::new();
        hasher.input_str(s.as_str());
        Ok(hasher.result_str())
    } else {
        Err(GitError::FileNotExistError(path.display().to_string()))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::{env, fs};

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
}
