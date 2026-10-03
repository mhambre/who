use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

pub fn hash(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("who: cannot read file `{}`: {error}", path.display()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_contents_without_text_conversion() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("schema");
        fs::write(&path, b"abc").unwrap();
        assert_eq!(
            hash(&path).unwrap(),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        fs::write(&path, [0xff, 0]).unwrap();
        assert!(hash(&path).is_ok());
    }
}
