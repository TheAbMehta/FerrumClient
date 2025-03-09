use ferrum_assets::{AssetManager, AssetError};

#[tokio::test]
async fn test_asset_manager_creation() {
    let result = AssetManager::new("1.20.1").await;
    assert!(result.is_ok(), "AssetManager creation should succeed");
}

#[tokio::test]
async fn test_cache_directory_creation() {
    let manager = AssetManager::new("1.20.1").await.unwrap();
    let cache_dir = manager.cache_dir();
    
    assert!(cache_dir.exists(), "Cache directory should be created");
    assert!(cache_dir.is_dir(), "Cache path should be a directory");
    assert!(cache_dir.ends_with("1.20.1"), "Cache dir should include version");
}

#[tokio::test]
async fn test_load_texture_all_sources_fail() {
    let manager = AssetManager::new("1.20.1").await.unwrap();
    
    let result = manager.load_texture("minecraft/textures/block/nonexistent_block_xyz_12345.png").await;
    assert!(result.is_err(), "Non-existent texture should fail");
    
    if let Err(AssetError::AllSourcesFailed(msg)) = result {
        assert!(msg.contains("Mojang"), "Error should mention Mojang source");
        assert!(msg.contains("JAR"), "Error should mention JAR source");
        assert!(msg.contains("PrismarineJS"), "Error should mention PrismarineJS source");
    } else {
        panic!("Expected AllSourcesFailed error");
    }
}

#[tokio::test]
async fn test_cache_hit_after_manual_write() {
    let manager = AssetManager::new("1.20.1").await.unwrap();
    let test_path = "minecraft/textures/test_cache.png";
    let test_data = b"fake texture data";
    
    let cache_file = manager.cache_dir().join(test_path);
    tokio::fs::create_dir_all(cache_file.parent().unwrap()).await.unwrap();
    tokio::fs::write(&cache_file, test_data).await.unwrap();
    
    let result = manager.load_texture(test_path).await;
    assert!(result.is_ok(), "Cache hit should succeed");
    assert_eq!(result.unwrap(), test_data, "Cached data should match");
}

#[tokio::test]
async fn test_multiple_versions_separate_caches() {
    let manager1 = AssetManager::new("1.20.1").await.unwrap();
    let manager2 = AssetManager::new("1.19.4").await.unwrap();
    
    assert_ne!(
        manager1.cache_dir(),
        manager2.cache_dir(),
        "Different versions should have separate cache directories"
    );
}
