use serde::Deserialize;

use crate::error::{FlasherError, Result};

const RK3566_POWKIDDY_X55_MANIFEST: &str = include_str!("../targets/rk3566-powkiddy-x55.toml");

#[derive(Debug, Clone)]
pub struct Target {
    pub id: String,
    pub display_name: String,
    pub signature_offset: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct TargetManifest {
    target: TargetMetadata,
    board: BoardMetadata,
    build: BuildMetadata,
    media: MediaMetadata,
}

#[derive(Debug, Deserialize)]
struct TargetMetadata {
    id: String,
    manifest_version: u32,
}

#[derive(Debug, Deserialize)]
struct BoardMetadata {
    manufacturer: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct BuildMetadata {
    image_format: String,
}

#[derive(Debug, Deserialize)]
struct MediaMetadata {
    signature_offset: u64,
    signature: String,
    write_offset: u64,
}

pub fn resolve(id: &str) -> Result<Target> {
    let target = parse_manifest(RK3566_POWKIDDY_X55_MANIFEST)?;

    if id == target.id {
        return Ok(target);
    }

    Err(FlasherError::message(format!(
        "unsupported target: {id}; supported targets: {}",
        target.id
    )))
}

fn parse_manifest(source: &str) -> Result<Target> {
    let manifest: TargetManifest = toml::from_str(source)
        .map_err(|error| FlasherError::message(format!("invalid target manifest: {error}")))?;

    if manifest.target.manifest_version != 1 {
        return Err(FlasherError::message(format!(
            "unsupported target manifest version: {}",
            manifest.target.manifest_version
        )));
    }

    if manifest.build.image_format != "rkns" {
        return Err(FlasherError::message(format!(
            "unsupported target image format: {}",
            manifest.build.image_format
        )));
    }

    if manifest.media.signature.is_empty() || !manifest.media.signature.is_ascii() {
        return Err(FlasherError::message(
            "target media signature must be non-empty ASCII text",
        ));
    }

    if manifest.media.write_offset != 0 {
        return Err(FlasherError::message(format!(
            "unsupported target media write offset: 0x{:x}",
            manifest.media.write_offset
        )));
    }

    Ok(Target {
        id: manifest.target.id,
        display_name: format!("{} {}", manifest.board.manufacturer, manifest.board.name),
        signature_offset: manifest.media.signature_offset,
        signature: manifest.media.signature.into_bytes(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_x55_manifest() {
        let target = resolve("rk3566-powkiddy-x55").expect("target should resolve");

        assert_eq!(target.id, "rk3566-powkiddy-x55");
        assert_eq!(target.display_name, "PowKiddy X55");
        assert_eq!(target.signature_offset, 0x8000);
        assert_eq!(target.signature, b"RKNS");
    }

    #[test]
    fn rejects_an_unknown_target() {
        let error = resolve("unknown").expect_err("unknown target should fail");

        assert!(error.to_string().contains("unsupported target: unknown"));
    }
}
