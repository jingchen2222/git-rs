use std::path::PathBuf;
use std::fs;
use crate::error::GitError;
use std::io::Read;
use crypto;
use crypto::digest::Digest;

/// crypto file to sha1
/// support text file currently, binary file will be supported in the future
pub fn crypto_file(path: &PathBuf) -> Result<String, GitError>{
    if path.exists() {
        let mut file = fs::File::open(&path)
            .map_err(|e| GitError::FileOpError(format!("{:?}",e)))?;
        let mut s = String::new();
        file.read_to_string(&mut s)
            .map_err(|e| GitError::FileOpError(format!("{:?}",e)))?;
        let mut hasher = crypto::sha1::Sha1::new();
        hasher.input_str(s.as_str());
        Ok(hasher.result_str())
    } else {
        Err(GitError::FileNotExistError(path.display().to_string()))
    }
}

/// copy file to repo
/// e.g src/d1/f1 to .git-repo-dir/src/d1/f1
pub fn copy_to(path: &PathBuf, dist: &PathBuf) -> Result<(), GitError> {
    if path.exists() {
        fs::copy(path, dist)
            .map_err(|e| GitError::FileOpError(format!("{:?}",e)))?;
        Ok(())
    } else {
        Err(GitError::FileNotExistError(path.display().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};
    use std::io::Write;

    #[test]
    fn crypto_file_ut() {
        let tmp_dir_path = &env::current_dir().unwrap().join("temp_utils");
        if !tmp_dir_path.exists() {
            fs::create_dir(&tmp_dir_path);
        }
        let file_path = tmp_dir_path.join("crypto_file_ut");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write("This is a demo content for crypto_file_ut".as_bytes());
        let hash = crypto_file(&file_path).unwrap();
        assert_eq!("2564cf76bd5b1cf65f7b9f52546f1ba7c8accee8", hash);

        if tmp_dir_path.exists() {
            fs::remove_dir_all(tmp_dir_path);
        }
    }
}