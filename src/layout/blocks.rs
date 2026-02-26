//! Code block detection for vertical layouts

use crate::error::Result;
use crate::vertical::{Position2D, Span2D, SpatialToken, WritingDirection};
use std::collections::HashMap;

/// Represents a detected code block
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeBlock {
    /// Span covering the entire block
    pub span: Span2D,
    /// Indentation level of this block
    pub indentation_level: usize,
    /// Type of block detected
    pub block_type: BlockType,
    /// Child blocks nested within this block
    pub children: Vec<CodeBlock>,
    /// Parent block (if nested)
    pub parent_id: Option<usize>,
}

/// Types of code blocks that can be detected
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BlockType {
    /// Function definition block
    Function,
    /// Type definition block
    Type,
    /// Conditional block (if/else)
    Conditional,
    /// Loop block
    Loop,
    /// Generic code block (based on indentation only)
    Generic,
    /// Expression block
    Expression,
    /// Comment block
    Comment,
}

impl CodeBlock {
    /// Create a new code block
    pub fn new(span: Span2D, indentation_level: usize, block_type: BlockType) -> Self {
        Self {
            span,
            indentation_level,
            block_type,
            children: Vec::new(),
            parent_id: None,
        }
    }

    /// Check if this block contains a given position
    pub fn contains(&self, pos: Position2D) -> bool {
        self.span.contains(pos)
    }

    /// Add a child block
    pub fn add_child(&mut self, mut child: CodeBlock, child_id: usize) {
        child.parent_id = Some(child_id);
        self.children.push(child);
    }

    /// Get the depth of nesting for this block
    pub fn nesting_depth(&self) -> usize {
        self.indentation_level / 4 // Assuming 4-space indentation
    }

    /// Check if this block is a leaf (has no children)
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Get all descendant blocks (children, grandchildren, etc.)
    pub fn descendants(&self) -> Vec<&CodeBlock> {
        let mut result = Vec::new();
        for child in &self.children {
            result.push(child);
            result.extend(child.descendants());
        }
        result
    }
}

/// Detects code blocks from spatial tokens and indentation information
pub struct BlockDetector {
    _direction: WritingDirection,
}

impl BlockDetector {
    /// Create a new block detector
    pub fn new(direction: WritingDirection) -> Self {
        Self {
            _direction: direction,
        }
    }

    /// Detect all code blocks in the given tokens
    pub fn detect_blocks(
        &self,
        tokens: &[SpatialToken],
        indentation_map: &HashMap<Position2D, usize>,
    ) -> Result<Vec<CodeBlock>> {
        let mut blocks = Vec::new();
        let _block_stack: Vec<usize> = Vec::new(); // Stack for tracking nested blocks
        let mut current_block: Option<BlockBuilder> = None;

        for token in tokens.iter() {
            // Skip whitespace tokens
            if token.is_whitespace() {
                continue;
            }

            let token_indent = indentation_map.get(&token.span.start).copied().unwrap_or(0);

            // Check for block-starting keywords
            if let Some(block_type) = self.detect_block_start(&token.content) {
                // If we have a current block, finish it
                if let Some(builder) = current_block.take() {
                    blocks.push(builder.build());
                }

                // Start a new block
                current_block = Some(BlockBuilder::new(
                    token.span.start,
                    token_indent,
                    block_type,
                ));
            }

            // Handle indentation changes
            if let Some(ref mut builder) = current_block {
                builder.add_token(token.clone());

                // Check if we've reached the end of the block
                let should_end = self.is_block_end(token, token_indent, builder.indentation_level);
                if should_end {
                    let block = current_block.take().unwrap().build();
                    blocks.push(block);
                }
            } else {
                // Not in a specific block, create a generic block if indented
                if token_indent > 0 {
                    let mut builder =
                        BlockBuilder::new(token.span.start, token_indent, BlockType::Generic);
                    builder.add_token(token.clone());
                    current_block = Some(builder);
                }
            }
        }

        // Finish any remaining block
        if let Some(builder) = current_block {
            blocks.push(builder.build());
        }

        // Build hierarchical structure
        self.build_hierarchy(blocks)
    }

    /// Detect if a token starts a new block
    fn detect_block_start(&self, content: &str) -> Option<BlockType> {
        match content {
            "関数" => Some(BlockType::Function),
            "型" => Some(BlockType::Type),
            "もし" => Some(BlockType::Conditional),
            "繰り返し" => Some(BlockType::Loop),
            _ => None,
        }
    }

    /// Check if we've reached the end of a block
    fn is_block_end(&self, token: &SpatialToken, token_indent: usize, block_indent: usize) -> bool {
        // Block ends when indentation decreases or equals the block's level
        token_indent <= block_indent && !token.is_whitespace()
    }

    /// Build hierarchical structure of blocks based on indentation
    fn build_hierarchy(&self, mut blocks: Vec<CodeBlock>) -> Result<Vec<CodeBlock>> {
        // Sort blocks by position to maintain order
        blocks.sort_by_key(|block| (block.span.start.row, block.span.start.column));

        let mut root_blocks = Vec::new();
        let mut stack: Vec<usize> = Vec::new(); // Indices into root_blocks

        for block in blocks {
            // Find the appropriate parent for this block
            while let Some(&parent_idx) = stack.last() {
                let parent: &CodeBlock = &root_blocks[parent_idx];
                if block.indentation_level > parent.indentation_level {
                    // This block is a child of the current parent
                    break;
                } else {
                    // Pop parents until we find the right level
                    stack.pop();
                }
            }

            let block_idx = root_blocks.len();

            if let Some(&parent_idx) = stack.last() {
                // Add as child to parent
                let parent = &mut root_blocks[parent_idx];
                parent.children.push(block);
            } else {
                // This is a root-level block
                root_blocks.push(block);
                stack.push(block_idx);
            }
        }

        Ok(root_blocks)
    }
}

/// Helper for building code blocks incrementally
struct BlockBuilder {
    start_position: Position2D,
    end_position: Option<Position2D>,
    indentation_level: usize,
    block_type: BlockType,
    tokens: Vec<SpatialToken>,
}

impl BlockBuilder {
    /// Create a new block builder
    fn new(start_position: Position2D, indentation_level: usize, block_type: BlockType) -> Self {
        Self {
            start_position,
            end_position: None,
            indentation_level,
            block_type,
            tokens: Vec::new(),
        }
    }

    /// Add a token to this block
    fn add_token(&mut self, token: SpatialToken) {
        self.end_position = Some(token.span.end);
        self.tokens.push(token);
    }

    /// Build the final code block
    fn build(self) -> CodeBlock {
        let end_pos = self.end_position.unwrap_or(self.start_position);
        let span = Span2D::new(self.start_position, end_pos);

        CodeBlock::new(span, self.indentation_level, self.block_type)
    }
}

/// Statistics about detected blocks
#[derive(Debug, Default)]
pub struct BlockStatistics {
    pub total_blocks: usize,
    pub function_blocks: usize,
    pub type_blocks: usize,
    pub conditional_blocks: usize,
    pub loop_blocks: usize,
    pub generic_blocks: usize,
    pub max_nesting_depth: usize,
    pub average_block_size: f64,
}

impl BlockStatistics {
    /// Calculate statistics from a set of blocks
    pub fn from_blocks(blocks: &[CodeBlock]) -> Self {
        let mut stats = Self {
            total_blocks: blocks.len(),
            ..Default::default()
        };

        for block in blocks {
            match block.block_type {
                BlockType::Function => stats.function_blocks += 1,
                BlockType::Type => stats.type_blocks += 1,
                BlockType::Conditional => stats.conditional_blocks += 1,
                BlockType::Loop => stats.loop_blocks += 1,
                BlockType::Generic => stats.generic_blocks += 1,
                _ => {}
            }

            let depth = block.nesting_depth();
            if depth > stats.max_nesting_depth {
                stats.max_nesting_depth = depth;
            }
        }

        if !blocks.is_empty() {
            let total_size: usize = blocks.iter().map(|b| b.span.byte_length()).sum();
            stats.average_block_size = total_size as f64 / blocks.len() as f64;
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertical::{SpatialToken, SpatialTokenKind};

    #[test]
    fn test_code_block_creation() {
        let start = Position2D::new(0, 0, 0);
        let end = Position2D::new(10, 5, 50);
        let span = Span2D::new(start, end);

        let block = CodeBlock::new(span, 4, BlockType::Function);
        assert_eq!(block.indentation_level, 4);
        assert_eq!(block.block_type, BlockType::Function);
        assert_eq!(block.nesting_depth(), 1);
        assert!(block.is_leaf());
    }

    #[test]
    fn test_block_detection() {
        let detector = BlockDetector::new(WritingDirection::VerticalTbRl);

        // Test block start detection
        assert_eq!(
            detector.detect_block_start("関数"),
            Some(BlockType::Function)
        );
        assert_eq!(detector.detect_block_start("型"), Some(BlockType::Type));
        assert_eq!(
            detector.detect_block_start("もし"),
            Some(BlockType::Conditional)
        );
        assert_eq!(detector.detect_block_start("other"), None);
    }

    #[test]
    fn test_block_statistics() {
        let blocks = vec![
            CodeBlock::new(
                Span2D::new(Position2D::new(0, 0, 0), Position2D::new(10, 0, 10)),
                0,
                BlockType::Function,
            ),
            CodeBlock::new(
                Span2D::new(Position2D::new(0, 1, 11), Position2D::new(5, 1, 16)),
                4,
                BlockType::Generic,
            ),
        ];

        let stats = BlockStatistics::from_blocks(&blocks);
        assert_eq!(stats.total_blocks, 2);
        assert_eq!(stats.function_blocks, 1);
        assert_eq!(stats.generic_blocks, 1);
        assert_eq!(stats.max_nesting_depth, 1);
    }

    #[test]
    fn test_block_hierarchy() {
        let mut parent = CodeBlock::new(
            Span2D::new(Position2D::new(0, 0, 0), Position2D::new(20, 5, 100)),
            0,
            BlockType::Function,
        );

        let child = CodeBlock::new(
            Span2D::new(Position2D::new(4, 1, 10), Position2D::new(10, 3, 80)),
            4,
            BlockType::Generic,
        );

        parent.add_child(child, 1);

        assert_eq!(parent.children.len(), 1);
        assert!(!parent.is_leaf());
        assert_eq!(parent.descendants().len(), 1);
    }
}
