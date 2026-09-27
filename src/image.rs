use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    error::{FlasherError, Result},
    target::Target,
};

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub struct Image {
    path: PathBuf,
    size: u64,
    sha256: String,
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
impl Image {
    pub fn inspect(path: &Path, target: &Target) -> Result<Self> {
        let canonical_path = path.canonicalize().map_err(|error| {
            FlasherError::message(format!("cannot resolve image {}: {error}", path.display()))
        })?;

        let metadata = canonical_path.metadata()?;
        if !metadata.is_file() {
            return Err(FlasherError::message(format!(
                "image is not a regular file: {}",
                canonical_path.display()
            )));
        }

        if metadata.len() == 0 {
            return Err(FlasherError::message(format!(
                "image is empty: {}",
                canonical_path.display()
            )));
        }

        validate_target_signature(&canonical_path, target)?;
        let sha256 = calculate_sha256(&canonical_path)?;

        Ok(Self {
            path: canonical_path,
            size: metadata.len(),
            sha256,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

fn validate_target_signature(path: &Path, target: &Target) -> Result<()> {
    let mut file = File::open(path)?;
    let signature_end = target
        .signature_offset
        .checked_add(target.signature.len() as u64)
        .ok_or_else(|| FlasherError::message("target signature offset overflow"))?;

    if file.metadata()?.len() < signature_end {
        return Err(FlasherError::message(format!(
            "image is too small to contain the {} signature",
            target.display_name
        )));
    }

    file.seek(SeekFrom::Start(target.signature_offset))?;
    let mut actual = vec![0_u8; target.signature.len()];
    file.read_exact(&mut actual)?;

    if actual != target.signature {
        return Err(FlasherError::message(format!(
            "image does not contain the expected {} signature at offset 0x{:x}",
            target.id, target.signature_offset
        )));
    }

    Ok(())
}

fn calculate_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
