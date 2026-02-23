use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// Compute the SHA-256 hash of a file and return it as a lowercase hex string.
#[allow(dead_code)]
pub fn file_sha256(path: &Path) -> Result<String> {
    let data = std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(bytes_sha256(&data))
}

/// Compute the SHA-256 hash of a byte slice and return it as a lowercase hex string.
pub fn bytes_sha256(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    format!("{hash:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn bytes_sha256_known_value() {
        // SHA-256 of empty string
        let hash = bytes_sha256(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn bytes_sha256_hello() {
        let hash = bytes_sha256(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn bytes_sha256_length() {
        let hash = bytes_sha256(b"anything");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn file_sha256_reads_file() {
        let dir = std::env::temp_dir().join("qvox_test_hash");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("test.bin");
        let mut f = std::fs::File::create(&path).expect("create file");
        f.write_all(b"hello").expect("write");
        drop(f);

        let hash = file_sha256(&path).expect("hash");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn file_sha256_missing_file() {
        let result = file_sha256(Path::new("/nonexistent/file.bin"));
        assert!(result.is_err());
    }
}
