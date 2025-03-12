use crate::{AssetError, AssetResult};
use serde::Deserialize;

#[derive(Deserialize)]
struct VersionManifest {
    versions: Vec<VersionEntry>,
}

#[derive(Deserialize)]
struct VersionEntry {
    id: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionInfo {
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndexInfo,
}

#[derive(Deserialize)]
struct AssetIndexInfo {
    url: String,
}

#[derive(Deserialize)]
struct AssetIndex {
    objects: std::collections::HashMap<String, AssetObject>,
}

#[derive(Deserialize)]
struct AssetObject {
    hash: String,
}

pub async fn fetch_asset(
    client: &reqwest::Client,
    version: &str,
    path: &str,
) -> AssetResult<Vec<u8>> {
    let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let manifest: VersionManifest = client.get(manifest_url).send().await?.json().await?;
    
    let version_entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version)
        .ok_or_else(|| AssetError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Version {} not found in manifest", version)
        )))?;
    
    let version_info: VersionInfo = client.get(&version_entry.url).send().await?.json().await?;
    let asset_index: AssetIndex = client.get(&version_info.asset_index.url).send().await?.json().await?;
    
    let asset_key = path
        .strip_prefix("minecraft/")
        .unwrap_or(path)
        .replace("/", "/");
    
    let asset_obj = asset_index
        .objects
        .get(&asset_key)
        .ok_or_else(|| AssetError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Asset {} not found in index (tried key: {})", path, asset_key)
        )))?;
    
    let hash = &asset_obj.hash;
    let asset_url = format!(
        "https://resources.download.minecraft.net/{}/{}",
        &hash[..2],
        hash
    );
    
    let data = client.get(&asset_url).send().await?.bytes().await?;
    Ok(data.to_vec())
}
