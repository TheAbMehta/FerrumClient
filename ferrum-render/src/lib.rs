mod block_renderer;
mod texture_atlas;

pub use block_renderer::BlockRenderer;
pub use texture_atlas::TextureAtlas;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to load texture: {0}")]
    TextureLoadFailed(String),

    #[error("Invalid mesh data: {0}")]
    InvalidMeshData(String),
}

pub type RenderResult<T> = Result<T, RenderError>;
