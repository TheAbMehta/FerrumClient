use crate::{AssetError, AssetResult};

pub async fn fetch_asset(
    client: &reqwest::Client,
    version: &str,
    path: &str,
) -> AssetResult<Vec<u8>> {
    let asset_path = path.strip_prefix("minecraft/").unwrap_or(path);
    
    let url = format!(
        "https://raw.githubusercontent.com/PrismarineJS/minecraft-assets/gh-pages/{}/{}",
        version,
        asset_path
    );
    
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        return Err(AssetError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Asset not found on PrismarineJS: {}", url)
        )));
    }
    
    let data = response.bytes().await?;
    Ok(data.to_vec())
}
