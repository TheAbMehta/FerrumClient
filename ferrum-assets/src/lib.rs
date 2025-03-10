mod mojang;
mod jar;
mod prismarine;

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("All asset sources failed: {0}")]
    AllSourcesFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub type AssetResult<T> = Result<T, AssetError>;

pub struct AssetManager {
    version: String,
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl AssetManager {
    pub async fn new(version: &str) -> AssetResult<Self> {
        let cache_dir = Self::get_cache_dir(version)?;
        tokio::fs::create_dir_all(&cache_dir).await?;
        
        Ok(Self {
            version: version.to_string(),
            cache_dir,
            client: reqwest::Client::new(),
        })
    }
    
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
    
    pub async fn load_texture(&self, path: &str) -> AssetResult<Vec<u8>> {
        let cache_path = self.cache_dir.join(path);
        
        if cache_path.exists() {
            return Ok(tokio::fs::read(&cache_path).await?);
        }
        
        let mut errors = Vec::new();
        
        match mojang::fetch_asset(&self.client, &self.version, path).await {
            Ok(data) => {
                self.cache_asset(path, &data).await?;
                return Ok(data);
            }
            Err(e) => errors.push(format!("Mojang: {}", e)),
        }
        
        match jar::extract_asset(&self.version, path).await {
            Ok(data) => {
                self.cache_asset(path, &data).await?;
                return Ok(data);
            }
            Err(e) => errors.push(format!("JAR: {}", e)),
        }
        
        match prismarine::fetch_asset(&self.client, &self.version, path).await {
            Ok(data) => {
                self.cache_asset(path, &data).await?;
                return Ok(data);
            }
            Err(e) => errors.push(format!("PrismarineJS: {}", e)),
        }
        
        Err(AssetError::AllSourcesFailed(errors.join(", ")))
    }
    
    async fn cache_asset(&self, path: &str, data: &[u8]) -> AssetResult<()> {
        let cache_path = self.cache_dir.join(path);
        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&cache_path, data).await?;
        Ok(())
    }
    
    fn get_cache_dir(version: &str) -> AssetResult<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| AssetError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find home directory"
            )))?;
        
        Ok(PathBuf::from(home)
            .join(".ferrum")
            .join("cache")
            .join("assets")
            .join(version))
    }
}
