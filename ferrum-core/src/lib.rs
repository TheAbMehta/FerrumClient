/// Unique identifier for a block type in the Minecraft world.
///
/// BlockId represents a specific block type (e.g., stone, dirt, air).
/// This is a core primitive used throughout the meshing and rendering pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(u16);

impl BlockId {
    /// Creates a new BlockId from a raw u16 value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrum_core::BlockId;
    ///
    /// let air = BlockId::new(0);
    /// let stone = BlockId::new(1);
    /// assert_ne!(air, stone);
    /// ```
    pub fn new(id: u16) -> Self {
        BlockId(id)
    }

    /// Returns the raw u16 value of this BlockId.
    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_id_creation() {
        let air = BlockId::new(0);
        let stone = BlockId::new(1);

        assert_eq!(air.as_u16(), 0);
        assert_eq!(stone.as_u16(), 1);
        assert_ne!(air, stone);
    }

    #[test]
    fn test_block_id_equality() {
        let stone1 = BlockId::new(1);
        let stone2 = BlockId::new(1);
        let dirt = BlockId::new(3);

        assert_eq!(stone1, stone2);
        assert_ne!(stone1, dirt);
    }
}
