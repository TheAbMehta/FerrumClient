use crate::{AssetError, AssetResult};
use std::path::PathBuf;
use zip::ZipArchive;

pub async fn extract_asset(version: &str, path: &str) -> AssetResult<Vec<u8>> {
    let jar_path = find_minecraft_jar(version)?;
    
    let jar_file = std::fs::File::open(&jar_path)?;
    let mut archive = ZipArchive::new(jar_file)?;
    
    let asset_path = format!("assets/{}", path.strip_prefix("minecraft/").unwrap_or(path));
    
    let mut file = archive.by_name(&asset_path).map_err(|_| {
        AssetError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Asset {} not found in JAR", asset_path)
        ))
    })?;
    
    let mut data = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut data)?;
    Ok(data)
}

fn find_minecraft_jar(version: &str) -> AssetResult<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| AssetError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory"
        )))?;
    
    let minecraft_dir = PathBuf::from(home).join(".minecraft");
    
    let jar_path = minecraft_dir
        .join("versions")
        .join(version)
        .join(format!("{}.jar", version));
    
    if jar_path.exists() {
        Ok(jar_path)
    } else {
        Err(AssetError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Minecraft JAR not found at {:?}", jar_path)
        )))
    }
}
