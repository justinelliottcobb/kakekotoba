//! Visitor pattern implementation for spatial ASTs

use super::{SpatialNode, SpatialContent, SpatialValidationError, NodeId};
use crate::vertical::{Position2D, Span2D};

/// Trait for visiting spatial AST nodes
pub trait SpatialVisitor {
    /// Visit a spatial node
    fn visit_node(&mut self, node: &SpatialNode) {
        self.enter_node(node);
        
        match &node.content {
            SpatialContent::Expression(expr) => self.visit_expression(node, expr),
            SpatialContent::Statement(stmt) => self.visit_statement(node, stmt),
            SpatialContent::Declaration(decl) => self.visit_declaration(node, decl),
            SpatialContent::Type(ty) => self.visit_type(node, ty),
            SpatialContent::Block(nodes) => self.visit_block(node, nodes),
            SpatialContent::Program => self.visit_program(node),
        }

        // Visit children
        for child in &node.children {
            self.visit_node(child);
        }

        self.exit_node(node);
    }

    /// Called when entering a node
    fn enter_node(&mut self, _node: &SpatialNode) {}

    /// Called when exiting a node
    fn exit_node(&mut self, _node: &SpatialNode) {}

    /// Visit an expression node
    fn visit_expression(&mut self, _node: &SpatialNode, _expr: &super::SpatialExpression) {}

    /// Visit a statement node
    fn visit_statement(&mut self, _node: &SpatialNode, _stmt: &super::SpatialStatement) {}

    /// Visit a declaration node
    fn visit_declaration(&mut self, _node: &SpatialNode, _decl: &super::SpatialDeclaration) {}

    /// Visit a type node
    fn visit_type(&mut self, _node: &SpatialNode, _ty: &super::SpatialType) {}

    /// Visit a block node
    fn visit_block(&mut self, _node: &SpatialNode, _nodes: &[SpatialNode]) {}

    /// Visit the program root
    fn visit_program(&mut self, _node: &SpatialNode) {}
}

/// Mutable visitor trait for transforming spatial ASTs
pub trait SpatialVisitorMut {
    /// Visit a spatial node mutably
    fn visit_node_mut(&mut self, node: &mut SpatialNode) {
        self.enter_node_mut(node);

        match &mut node.content {
            SpatialContent::Expression(expr) => self.visit_expression_mut(node, expr),
            SpatialContent::Statement(stmt) => self.visit_statement_mut(node, stmt),
            SpatialContent::Declaration(decl) => self.visit_declaration_mut(node, decl),
            SpatialContent::Type(ty) => self.visit_type_mut(node, ty),
            SpatialContent::Block(nodes) => self.visit_block_mut(node, nodes),
            SpatialContent::Program => self.visit_program_mut(node),
        }

        // Visit children
        for child in &mut node.children {
            self.visit_node_mut(child);
        }

        self.exit_node_mut(node);
    }

    /// Called when entering a node
    fn enter_node_mut(&mut self, _node: &mut SpatialNode) {}

    /// Called when exiting a node
    fn exit_node_mut(&mut self, _node: &mut SpatialNode) {}

    /// Visit an expression node mutably
    fn visit_expression_mut(&mut self, _node: &mut SpatialNode, _expr: &mut super::SpatialExpression) {}

    /// Visit a statement node mutably
    fn visit_statement_mut(&mut self, _node: &mut SpatialNode, _stmt: &mut super::SpatialStatement) {}

    /// Visit a declaration node mutably
    fn visit_declaration_mut(&mut self, _node: &mut SpatialNode, _decl: &mut super::SpatialDeclaration) {}

    /// Visit a type node mutably
    fn visit_type_mut(&mut self, _node: &mut SpatialNode, _ty: &mut super::SpatialType) {}

    /// Visit a block node mutably
    fn visit_block_mut(&mut self, _node: &mut SpatialNode, _nodes: &mut [SpatialNode]) {}

    /// Visit the program root mutably
    fn visit_program_mut(&mut self, _node: &mut SpatialNode) {}
}

/// Query visitor to find nodes at a specific position
pub struct PositionQuery {
    position: Position2D,
    results: Vec<NodeId>,
}

impl PositionQuery {
    /// Create a new position query
    pub fn new(position: Position2D) -> Self {
        Self {
            position,
            results: Vec::new(),
        }
    }

    /// Get the results of the query
    pub fn results(&self) -> Vec<&NodeId> {
        self.results.iter().collect()
    }

    /// Take the results, consuming the query
    pub fn take_results(self) -> Vec<NodeId> {
        self.results
    }
}

impl SpatialVisitor for PositionQuery {
    fn enter_node(&mut self, node: &SpatialNode) {
        if node.contains(self.position) {
            self.results.push(node.id);
        }
    }
}

/// Query visitor to find nodes in a specific range
pub struct RangeQuery {
    range: Span2D,
    results: Vec<NodeId>,
}

impl RangeQuery {
    /// Create a new range query
    pub fn new(range: Span2D) -> Self {
        Self {
            range,
            results: Vec::new(),
        }
    }

    /// Get the results of the query
    pub fn results(&self) -> Vec<&NodeId> {
        self.results.iter().collect()
    }

    /// Take the results, consuming the query
    pub fn take_results(self) -> Vec<NodeId> {
        self.results
    }
}

impl SpatialVisitor for RangeQuery {
    fn enter_node(&mut self, node: &SpatialNode) {
        if node.overlaps(&self.range) {
            self.results.push(node.id);
        }
    }
}

/// Visitor to calculate spatial metrics
pub struct MetricsCalculator {
    metrics: super::SpatialMetrics,
    current_depth: usize,
    indentation_sum: usize,
    node_count: usize,
}

impl MetricsCalculator {
    /// Create a new metrics calculator
    pub fn new() -> Self {
        Self {
            metrics: super::SpatialMetrics::default(),
            current_depth: 0,
            indentation_sum: 0,
            node_count: 0,
        }
    }

    /// Get the calculated metrics
    pub fn metrics(mut self) -> super::SpatialMetrics {
        // Finalize calculations
        self.metrics.total_nodes = self.node_count;
        if self.node_count > 0 {
            self.metrics.avg_indentation = self.indentation_sum as f64 / self.node_count as f64;
        }
        
        // Calculate density (nodes per unit coverage)
        if self.metrics.total_coverage > 0 {
            self.metrics.density = self.node_count as f64 / self.metrics.total_coverage as f64;
        }
        
        self.metrics
    }
}

impl SpatialVisitor for MetricsCalculator {
    fn enter_node(&mut self, node: &SpatialNode) {
        self.node_count += 1;
        self.current_depth += 1;
        
        // Update max depth
        if self.current_depth > self.metrics.max_depth {
            self.metrics.max_depth = self.current_depth;
        }
        
        // Add to indentation sum
        self.indentation_sum += node.metadata.indentation_level;
        
        // Add to total coverage
        self.metrics.total_coverage += node.span.byte_length();
        
        // Count blocks
        if node.metadata.starts_block {
            self.metrics.block_count += 1;
        }
    }

    fn exit_node(&mut self, _node: &SpatialNode) {
        self.current_depth -= 1;
    }
}

/// Validator for spatial AST consistency
pub struct SpatialValidator {
    errors: Vec<SpatialValidationError>,
    node_stack: Vec<NodeId>,
}

impl SpatialValidator {
    /// Create a new spatial validator
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            node_stack: Vec::new(),
        }
    }

    /// Get the validation errors
    pub fn errors(self) -> Vec<SpatialValidationError> {
        self.errors
    }
}

impl SpatialVisitor for SpatialValidator {
    fn enter_node(&mut self, node: &SpatialNode) {
        // Check span validity
        if node.span.start > node.span.end {
            self.errors.push(SpatialValidationError::InvalidSpan {
                node_id: node.id,
                span: node.span,
            });
        }

        // Check parent-child relationship
        if let Some(&parent_id) = self.node_stack.last() {
            // In a real implementation, we would check if child extends outside parent
            // For now, just store the parent ID for the check
            let _ = parent_id; // Placeholder
        }

        self.node_stack.push(node.id);
    }

    fn exit_node(&mut self, _node: &SpatialNode) {
        self.node_stack.pop();
    }
}

/// Visitor to collect all nodes of a specific type
pub struct NodeCollector<F> {
    predicate: F,
    results: Vec<NodeId>,
}

impl<F> NodeCollector<F> 
where 
    F: Fn(&SpatialNode) -> bool,
{
    /// Create a new node collector with a predicate
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            results: Vec::new(),
        }
    }

    /// Get the collected node IDs
    pub fn results(&self) -> &[NodeId] {
        &self.results
    }

    /// Take the results, consuming the collector
    pub fn take_results(self) -> Vec<NodeId> {
        self.results
    }
}

impl<F> SpatialVisitor for NodeCollector<F>
where
    F: Fn(&SpatialNode) -> bool,
{
    fn enter_node(&mut self, node: &SpatialNode) {
        if (self.predicate)(node) {
            self.results.push(node.id);
        }
    }
}

/// Visitor to update reading order
pub struct ReadingOrderUpdater {
    current_order: usize,
}

impl ReadingOrderUpdater {
    /// Create a new reading order updater
    pub fn new() -> Self {
        Self { current_order: 0 }
    }

    /// Get the final reading order count
    pub fn final_order(&self) -> usize {
        self.current_order
    }
}

impl SpatialVisitorMut for ReadingOrderUpdater {
    fn enter_node_mut(&mut self, node: &mut SpatialNode) {
        node.metadata.reading_order = Some(self.current_order);
        self.current_order += 1;
    }
}

/// Visitor to find the deepest node at a position
pub struct DeepestNodeFinder {
    position: Position2D,
    deepest_node: Option<NodeId>,
    max_depth: usize,
    current_depth: usize,
}

impl DeepestNodeFinder {
    /// Create a new deepest node finder
    pub fn new(position: Position2D) -> Self {
        Self {
            position,
            deepest_node: None,
            max_depth: 0,
            current_depth: 0,
        }
    }

    /// Get the deepest node found
    pub fn deepest_node(&self) -> Option<NodeId> {
        self.deepest_node
    }
}

impl SpatialVisitor for DeepestNodeFinder {
    fn enter_node(&mut self, node: &SpatialNode) {
        self.current_depth += 1;
        
        if node.contains(self.position) && self.current_depth > self.max_depth {
            self.deepest_node = Some(node.id);
            self.max_depth = self.current_depth;
        }
    }

    fn exit_node(&mut self, _node: &SpatialNode) {
        self.current_depth -= 1;
    }
}

/// Convenience functions for common visitor operations
pub mod queries {
    use super::*;

    /// Find all nodes at a specific position
    pub fn nodes_at_position(root: &SpatialNode, pos: Position2D) -> Vec<NodeId> {
        let mut query = PositionQuery::new(pos);
        query.visit_node(root);
        query.take_results()
    }

    /// Find nodes in a range
    pub fn nodes_in_range(root: &SpatialNode, range: Span2D) -> Vec<NodeId> {
        let mut query = RangeQuery::new(range);
        query.visit_node(root);
        query.take_results()
    }

    /// Calculate metrics for a spatial AST
    pub fn calculate_metrics(root: &SpatialNode) -> super::SpatialMetrics {
        let mut calculator = MetricsCalculator::new();
        calculator.visit_node(root);
        calculator.metrics()
    }

    /// Validate a spatial AST
    pub fn validate_ast(root: &SpatialNode) -> Vec<SpatialValidationError> {
        let mut validator = SpatialValidator::new();
        validator.visit_node(root);
        validator.errors()
    }

    /// Find the deepest node at a position
    pub fn deepest_node_at(root: &SpatialNode, pos: Position2D) -> Option<NodeId> {
        let mut finder = DeepestNodeFinder::new(pos);
        finder.visit_node(root);
        finder.deepest_node()
    }

    /// Collect nodes matching a predicate
    pub fn collect_nodes<F>(root: &SpatialNode, predicate: F) -> Vec<NodeId>
    where
        F: Fn(&SpatialNode) -> bool,
    {
        let mut collector = NodeCollector::new(predicate);
        collector.visit_node(root);
        collector.take_results()
    }

    /// Update reading order for all nodes
    pub fn update_reading_order(root: &mut SpatialNode) -> usize {
        let mut updater = ReadingOrderUpdater::new();
        updater.visit_node_mut(root);
        updater.final_order()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertical::Position2D;

    fn create_test_node(id: NodeId, start: Position2D, end: Position2D) -> SpatialNode {
        let span = Span2D::new(start, end);
        SpatialNode::new(id, span, SpatialContent::Program, Vec::new())
    }

    #[test]
    fn test_position_query() {
        let node = create_test_node(1, Position2D::new(0, 0, 0), Position2D::new(10, 10, 100));
        let mut query = PositionQuery::new(Position2D::new(5, 5, 50));
        
        query.visit_node(&node);
        let results = query.take_results();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 1);
    }

    #[test]
    fn test_range_query() {
        let node = create_test_node(1, Position2D::new(0, 0, 0), Position2D::new(10, 10, 100));
        let range = Span2D::new(Position2D::new(5, 5, 50), Position2D::new(15, 15, 150));
        let mut query = RangeQuery::new(range);
        
        query.visit_node(&node);
        let results = query.take_results();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 1);
    }

    #[test]
    fn test_metrics_calculator() {
        let node = create_test_node(1, Position2D::new(0, 0, 0), Position2D::new(10, 10, 100));
        let mut calculator = MetricsCalculator::new();
        
        calculator.visit_node(&node);
        let metrics = calculator.metrics();
        
        assert_eq!(metrics.total_nodes, 1);
        assert_eq!(metrics.max_depth, 1);
        assert_eq!(metrics.total_coverage, 100);
    }

    #[test]
    fn test_spatial_validator() {
        // Create a node with invalid span (end before start)
        let mut node = create_test_node(1, Position2D::new(10, 10, 100), Position2D::new(0, 0, 0));
        node.span = Span2D::new(Position2D::new(10, 10, 100), Position2D::new(0, 0, 0));
        
        let mut validator = SpatialValidator::new();
        validator.visit_node(&node);
        let errors = validator.errors();
        
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], SpatialValidationError::InvalidSpan { .. }));
    }

    #[test]
    fn test_node_collector() {
        let node = create_test_node(1, Position2D::new(0, 0, 0), Position2D::new(10, 10, 100));
        let collector = NodeCollector::new(|n| n.id == 1);
        
        let mut visitor = collector;
        visitor.visit_node(&node);
        let results = visitor.take_results();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 1);
    }

    #[test]
    fn test_reading_order_updater() {
        let mut node = create_test_node(1, Position2D::new(0, 0, 0), Position2D::new(10, 10, 100));
        let mut updater = ReadingOrderUpdater::new();
        
        updater.visit_node_mut(&mut node);
        
        assert_eq!(node.metadata.reading_order, Some(0));
        assert_eq!(updater.final_order(), 1);
    }

    #[test]
    fn test_convenience_functions() {
        let node = create_test_node(1, Position2D::new(0, 0, 0), Position2D::new(10, 10, 100));
        
        let pos_results = queries::nodes_at_position(&node, Position2D::new(5, 5, 50));
        assert_eq!(pos_results.len(), 1);
        
        let metrics = queries::calculate_metrics(&node);
        assert_eq!(metrics.total_nodes, 1);
        
        let errors = queries::validate_ast(&node);
        assert!(errors.is_empty());
        
        let deepest = queries::deepest_node_at(&node, Position2D::new(5, 5, 50));
        assert_eq!(deepest, Some(1));
    }
}