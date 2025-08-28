//! Spatial AST - Abstract Syntax Tree with 2D positional metadata
//!
//! This module extends the standard AST with spatial positioning information,
//! enabling vertical programming language features and preserving layout semantics.

use crate::vertical::{Position2D, Span2D};
use crate::layout::CodeLayout;
use crate::ast::{Expression, Statement, Declaration, Type};
use crate::error::Result;
use serde::{Serialize, Deserialize};

pub mod nodes;
pub mod visitor;
pub mod transformer;

pub use nodes::*;
pub use visitor::*;
pub use transformer::*;

/// Root node of a spatial AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialProgram {
    /// The original AST program
    pub program: crate::ast::Program,
    /// Root spatial node
    pub spatial_root: SpatialNode,
    /// Layout information
    pub layout: CodeLayout,
    /// Source file information
    pub source_info: SourceInfo,
}

impl SpatialProgram {
    /// Create a new spatial program
    pub fn new(
        program: crate::ast::Program,
        spatial_root: SpatialNode,
        layout: CodeLayout,
        source_info: SourceInfo,
    ) -> Self {
        Self {
            program,
            spatial_root,
            layout,
            source_info,
        }
    }

    /// Get all nodes at a specific position
    pub fn nodes_at_position(&self, pos: Position2D) -> Vec<&SpatialNode> {
        let mut visitor = PositionQuery::new(pos);
        visitor.visit_node(&self.spatial_root);
        visitor.results()
    }

    /// Find the deepest node containing a position
    pub fn deepest_node_at(&self, pos: Position2D) -> Option<&SpatialNode> {
        let nodes = self.nodes_at_position(pos);
        // Return the node with the smallest span (deepest/most specific)
        nodes.into_iter()
            .min_by_key(|node| node.span.byte_length())
    }

    /// Get all nodes in a spatial range
    pub fn nodes_in_range(&self, start: Position2D, end: Position2D) -> Vec<&SpatialNode> {
        let query_span = Span2D::new(start, end);
        let mut visitor = RangeQuery::new(query_span);
        visitor.visit_node(&self.spatial_root);
        visitor.results()
    }

    /// Calculate spatial metrics for the program
    pub fn spatial_metrics(&self) -> SpatialMetrics {
        let mut calculator = MetricsCalculator::new();
        calculator.visit_node(&self.spatial_root);
        calculator.metrics()
    }

    /// Validate spatial consistency
    pub fn validate_spatial_consistency(&self) -> Result<Vec<SpatialValidationError>> {
        let mut validator = SpatialValidator::new();
        validator.visit_node(&self.spatial_root);
        Ok(validator.errors())
    }
}

/// Information about the source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// File path or identifier
    pub file_path: Option<String>,
    /// Original source text
    pub source_text: String,
    /// File encoding information
    pub encoding: String,
    /// Writing direction used in the file
    pub writing_direction: crate::vertical::WritingDirection,
}

impl SourceInfo {
    /// Create new source info
    pub fn new(
        file_path: Option<String>,
        source_text: String,
        writing_direction: crate::vertical::WritingDirection,
    ) -> Self {
        Self {
            file_path,
            source_text,
            encoding: "UTF-8".to_string(),
            writing_direction,
        }
    }

    /// Get source text for a specific span
    pub fn text_for_span(&self, span: &Span2D) -> Option<&str> {
        let start = span.start.byte_offset;
        let end = span.end.byte_offset;
        
        if end <= self.source_text.len() {
            self.source_text.get(start..end)
        } else {
            None
        }
    }

    /// Get the line containing a position
    pub fn line_at_position(&self, pos: Position2D) -> Option<&str> {
        let lines: Vec<&str> = self.source_text.lines().collect();
        lines.get(pos.row)
    }
}

/// Core spatial node that wraps AST nodes with positional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialNode {
    /// Unique identifier for this node
    pub id: NodeId,
    /// 2D span of this node
    pub span: Span2D,
    /// The wrapped AST content
    pub content: SpatialContent,
    /// Child nodes
    pub children: Vec<SpatialNode>,
    /// Additional spatial metadata
    pub metadata: SpatialMetadata,
}

/// Unique identifier for spatial nodes
pub type NodeId = u64;

/// Content of a spatial node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpatialContent {
    /// Expression with spatial information
    Expression(SpatialExpression),
    /// Statement with spatial information
    Statement(SpatialStatement),
    /// Declaration with spatial information
    Declaration(SpatialDeclaration),
    /// Type with spatial information
    Type(SpatialType),
    /// Block of statements
    Block(Vec<SpatialNode>),
    /// Program root
    Program,
}

/// Additional metadata for spatial nodes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpatialMetadata {
    /// Indentation level at this node
    pub indentation_level: usize,
    /// Whether this node starts a new block
    pub starts_block: bool,
    /// Whether this node ends a block
    pub ends_block: bool,
    /// Reading order index (for vertical text)
    pub reading_order: Option<usize>,
    /// Visual importance weight
    pub visual_weight: f64,
    /// Comments associated with this node
    pub comments: Vec<SpatialComment>,
}

/// A comment with spatial information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialComment {
    /// Comment text
    pub text: String,
    /// Position of the comment
    pub span: Span2D,
    /// Type of comment
    pub comment_type: CommentType,
}

/// Types of comments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentType {
    /// Line comment
    Line,
    /// Block comment
    Block,
    /// Documentation comment
    Documentation,
}

impl SpatialNode {
    /// Create a new spatial node
    pub fn new(
        id: NodeId,
        span: Span2D,
        content: SpatialContent,
        children: Vec<SpatialNode>,
    ) -> Self {
        Self {
            id,
            span,
            content,
            children,
            metadata: SpatialMetadata::default(),
        }
    }

    /// Check if this node contains a position
    pub fn contains(&self, pos: Position2D) -> bool {
        self.span.contains(pos)
    }

    /// Check if this node overlaps with a span
    pub fn overlaps(&self, span: &Span2D) -> bool {
        self.span.overlaps(span)
    }

    /// Get all descendant nodes
    pub fn descendants(&self) -> Vec<&SpatialNode> {
        let mut result = Vec::new();
        for child in &self.children {
            result.push(child);
            result.extend(child.descendants());
        }
        result
    }

    /// Find child nodes at a specific indentation level
    pub fn children_at_level(&self, level: usize) -> Vec<&SpatialNode> {
        self.children.iter()
            .filter(|child| child.metadata.indentation_level == level)
            .collect()
    }

    /// Get the depth of this node in the tree
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self.children.iter()
                .map(|child| child.depth())
                .max()
                .unwrap_or(0)
        }
    }

    /// Get the reading order position of this node
    pub fn reading_order(&self) -> Option<usize> {
        self.metadata.reading_order
    }

    /// Set reading order for this node and all descendants
    pub fn set_reading_order(&mut self, start: usize) -> usize {
        let mut current = start;
        self.metadata.reading_order = Some(current);
        current += 1;

        for child in &mut self.children {
            current = child.set_reading_order(current);
        }

        current
    }
}

/// Metrics about spatial AST structure
#[derive(Debug, Clone, Default)]
pub struct SpatialMetrics {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Maximum depth of the tree
    pub max_depth: usize,
    /// Average indentation level
    pub avg_indentation: f64,
    /// Number of blocks
    pub block_count: usize,
    /// Total span coverage (in bytes)
    pub total_coverage: usize,
    /// Density (nodes per unit area)
    pub density: f64,
}

impl SpatialMetrics {
    /// Calculate complexity score based on metrics
    pub fn complexity_score(&self) -> f64 {
        let depth_factor = self.max_depth as f64 * 0.3;
        let node_factor = (self.total_nodes as f64).ln() * 0.2;
        let indentation_factor = self.avg_indentation * 0.1;
        let density_factor = self.density * 0.4;
        
        depth_factor + node_factor + indentation_factor + density_factor
    }
}

/// Spatial validation errors
#[derive(Debug, Clone)]
pub enum SpatialValidationError {
    /// Node span is invalid (end before start)
    InvalidSpan { node_id: NodeId, span: Span2D },
    /// Child node extends outside parent span
    ChildOutsideParent { parent_id: NodeId, child_id: NodeId },
    /// Overlapping sibling nodes
    OverlappingSiblings { node1_id: NodeId, node2_id: NodeId },
    /// Inconsistent indentation
    InconsistentIndentation { node_id: NodeId, expected: usize, actual: usize },
    /// Missing position information
    MissingPosition { node_id: NodeId },
}

/// Builder for creating spatial ASTs
pub struct SpatialASTBuilder {
    next_id: NodeId,
    layout: Option<CodeLayout>,
}

impl SpatialASTBuilder {
    /// Create a new spatial AST builder
    pub fn new() -> Self {
        Self {
            next_id: 1,
            layout: None,
        }
    }

    /// Set the layout information
    pub fn with_layout(mut self, layout: CodeLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Generate a new unique node ID
    fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Build a spatial program from a regular AST program
    pub fn build_program(
        &mut self,
        program: crate::ast::Program,
        source_info: SourceInfo,
    ) -> Result<SpatialProgram> {
        let layout = self.layout.take()
            .unwrap_or_else(|| CodeLayout::new(source_info.writing_direction));

        let spatial_root = self.build_node_from_program(&program)?;

        Ok(SpatialProgram::new(program, spatial_root, layout, source_info))
    }

    /// Build a spatial node from AST program (placeholder implementation)
    fn build_node_from_program(&mut self, _program: &crate::ast::Program) -> Result<SpatialNode> {
        // Placeholder implementation
        // In practice, this would traverse the AST and create spatial nodes
        let root_span = Span2D::new(Position2D::origin(), Position2D::new(0, 0, 0));
        
        Ok(SpatialNode::new(
            self.next_id(),
            root_span,
            SpatialContent::Program,
            Vec::new(),
        ))
    }
}

impl Default for SpatialASTBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertical::WritingDirection;

    #[test]
    fn test_spatial_program_creation() {
        let program = crate::ast::Program { declarations: Vec::new() };
        let root_span = Span2D::new(Position2D::origin(), Position2D::new(10, 10, 100));
        let spatial_root = SpatialNode::new(1, root_span, SpatialContent::Program, Vec::new());
        let layout = CodeLayout::new(WritingDirection::VerticalTbRl);
        let source_info = SourceInfo::new(
            Some("test.kake".to_string()),
            "関数 test() {}".to_string(),
            WritingDirection::VerticalTbRl,
        );

        let spatial_program = SpatialProgram::new(program, spatial_root, layout, source_info);
        
        assert!(spatial_program.source_info.file_path.is_some());
        assert_eq!(spatial_program.source_info.writing_direction, WritingDirection::VerticalTbRl);
    }

    #[test]
    fn test_spatial_node_operations() {
        let span = Span2D::new(Position2D::new(0, 0, 0), Position2D::new(5, 5, 25));
        let mut node = SpatialNode::new(1, span, SpatialContent::Program, Vec::new());
        
        assert!(node.contains(Position2D::new(2, 2, 10)));
        assert!(!node.contains(Position2D::new(10, 10, 100)));
        
        // Test reading order
        node.set_reading_order(0);
        assert_eq!(node.reading_order(), Some(0));
    }

    #[test]
    fn test_source_info() {
        let source_info = SourceInfo::new(
            Some("test.kake".to_string()),
            "関数 main() {\n    返す 42;\n}".to_string(),
            WritingDirection::VerticalTbRl,
        );

        let span = Span2D::new(Position2D::new(0, 0, 0), Position2D::new(2, 0, 2));
        let text = source_info.text_for_span(&span);
        assert_eq!(text, Some("関数"));
    }

    #[test]
    fn test_spatial_ast_builder() {
        let mut builder = SpatialASTBuilder::new();
        let layout = CodeLayout::new(WritingDirection::VerticalTbRl);
        builder = builder.with_layout(layout);

        let program = crate::ast::Program { declarations: Vec::new() };
        let source_info = SourceInfo::new(
            None,
            "test".to_string(),
            WritingDirection::VerticalTbRl,
        );

        let spatial_program = builder.build_program(program, source_info).unwrap();
        assert_eq!(spatial_program.spatial_root.id, 1);
    }

    #[test]
    fn test_spatial_metrics() {
        let metrics = SpatialMetrics {
            total_nodes: 100,
            max_depth: 5,
            avg_indentation: 2.5,
            block_count: 20,
            total_coverage: 1000,
            density: 0.1,
        };

        let complexity = metrics.complexity_score();
        assert!(complexity > 0.0);
    }
}