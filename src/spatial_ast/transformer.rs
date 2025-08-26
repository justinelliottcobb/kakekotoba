//! AST transformation utilities for spatial ASTs

use super::{SpatialNode, SpatialContent, SpatialProgram, SpatialASTBuilder, SourceInfo};
use super::visitor::{SpatialVisitor, SpatialVisitorMut};
use crate::ast;
use crate::vertical::{Position2D, Span2D, SpatialToken, WritingDirection};
use crate::layout::CodeLayout;
use crate::error::Result;

/// Transforms regular ASTs to spatial ASTs
pub struct SpatialTransformer {
    builder: SpatialASTBuilder,
    current_position: Position2D,
    current_indentation: usize,
}

impl SpatialTransformer {
    /// Create a new spatial transformer
    pub fn new() -> Self {
        Self {
            builder: SpatialASTBuilder::new(),
            current_position: Position2D::origin(),
            current_indentation: 0,
        }
    }

    /// Transform a regular AST program to a spatial AST program
    pub fn transform_program(
        &mut self,
        program: ast::Program,
        tokens: &[SpatialToken],
        source_text: String,
        writing_direction: WritingDirection,
    ) -> Result<SpatialProgram> {
        // Analyze layout from tokens
        let layout = CodeLayout::analyze(tokens)?;
        self.builder = self.builder.with_layout(layout);

        // Create source info
        let source_info = SourceInfo::new(None, source_text, writing_direction);

        // Transform the program
        self.builder.build_program(program, source_info)
    }

    /// Transform an expression to spatial form
    pub fn transform_expression(
        &mut self,
        expr: &ast::Expression,
        span: Span2D,
    ) -> Result<SpatialNode> {
        let spatial_expr = self.create_spatial_expression(expr, span)?;
        let content = SpatialContent::Expression(spatial_expr);
        
        Ok(SpatialNode::new(
            self.next_id(),
            span,
            content,
            Vec::new(),
        ))
    }

    /// Transform a statement to spatial form
    pub fn transform_statement(
        &mut self,
        stmt: &ast::Statement,
        span: Span2D,
    ) -> Result<SpatialNode> {
        let spatial_stmt = self.create_spatial_statement(stmt, span)?;
        let content = SpatialContent::Statement(spatial_stmt);
        
        Ok(SpatialNode::new(
            self.next_id(),
            span,
            content,
            Vec::new(),
        ))
    }

    /// Transform a declaration to spatial form
    pub fn transform_declaration(
        &mut self,
        decl: &ast::Declaration,
        span: Span2D,
    ) -> Result<SpatialNode> {
        let spatial_decl = self.create_spatial_declaration(decl, span)?;
        let content = SpatialContent::Declaration(spatial_decl);
        
        Ok(SpatialNode::new(
            self.next_id(),
            span,
            content,
            Vec::new(),
        ))
    }

    /// Create spatial expression from regular expression
    fn create_spatial_expression(
        &self,
        expr: &ast::Expression,
        _span: Span2D,
    ) -> Result<super::SpatialExpression> {
        let mut props = super::nodes::ExpressionSpatialProps::default();
        
        // Set properties based on expression type
        match expr {
            ast::Expression::Binary { .. } => {
                props.multiline = false;
                props.precedence_level = 1;
                props.operator_alignment = super::nodes::OperatorAlignment::Center;
            }
            ast::Expression::Call { .. } => {
                props.multiline = false;
                props.precedence_level = 10;
                props.paren_style = super::nodes::ParenStyle::Horizontal;
            }
            _ => {
                props.precedence_level = 0;
            }
        }

        Ok(super::SpatialExpression::new(expr.clone(), props))
    }

    /// Create spatial statement from regular statement
    fn create_spatial_statement(
        &self,
        stmt: &ast::Statement,
        _span: Span2D,
    ) -> Result<super::SpatialStatement> {
        let mut props = super::nodes::StatementSpatialProps::default();
        
        // Set properties based on statement type
        match stmt {
            ast::Statement::Expression(_) => {
                props.termination = super::nodes::StatementTermination::Semicolon;
                props.indentation_style = super::nodes::IndentationStyle::None;
            }
            ast::Statement::If { .. } => {
                props.requires_block = true;
                props.indentation_style = super::nodes::IndentationStyle::Block;
                props.flow_control.nesting_level = self.current_indentation + 1;
            }
            ast::Statement::While { .. } => {
                props.requires_block = true;
                props.indentation_style = super::nodes::IndentationStyle::Block;
                props.flow_control.nesting_level = self.current_indentation + 1;
            }
            ast::Statement::Return(_) => {
                props.flow_control.breaks_flow = true;
                props.termination = super::nodes::StatementTermination::Semicolon;
            }
        }

        Ok(super::SpatialStatement::new(stmt.clone(), props))
    }

    /// Create spatial declaration from regular declaration
    fn create_spatial_declaration(
        &self,
        decl: &ast::Declaration,
        _span: Span2D,
    ) -> Result<super::SpatialDeclaration> {
        let mut props = super::nodes::DeclarationSpatialProps::default();
        
        // Set properties based on declaration type
        match decl {
            ast::Declaration::Function { .. } => {
                props.starts_section = true;
                props.visibility_scope = super::nodes::VisibilityScope::Private;
                props.doc_position = super::nodes::DocumentationPosition::Above;
            }
            ast::Declaration::Type { .. } => {
                props.starts_section = true;
                props.visibility_scope = super::nodes::VisibilityScope::Private;
                props.doc_position = super::nodes::DocumentationPosition::Above;
            }
            ast::Declaration::Variable { .. } => {
                props.starts_section = false;
                props.visibility_scope = super::nodes::VisibilityScope::Private;
            }
        }

        Ok(super::SpatialDeclaration::new(decl.clone(), props))
    }

    /// Get next unique node ID
    fn next_id(&mut self) -> super::NodeId {
        // This would be managed by the builder in practice
        1 // Placeholder
    }
}

impl Default for SpatialTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimizes spatial ASTs for better layout and performance
pub struct SpatialOptimizer {
    optimizations_applied: usize,
}

impl SpatialOptimizer {
    /// Create a new spatial optimizer
    pub fn new() -> Self {
        Self {
            optimizations_applied: 0,
        }
    }

    /// Optimize a spatial AST program
    pub fn optimize_program(&mut self, program: &mut SpatialProgram) -> Result<usize> {
        self.optimizations_applied = 0;
        
        // Apply various optimizations
        self.optimize_reading_order(&mut program.spatial_root);
        self.optimize_indentation(&mut program.spatial_root);
        self.optimize_spatial_layout(&mut program.spatial_root);
        self.compact_metadata(&mut program.spatial_root);
        
        Ok(self.optimizations_applied)
    }

    /// Optimize reading order for better vertical flow
    fn optimize_reading_order(&mut self, root: &mut SpatialNode) {
        let mut updater = super::visitor::ReadingOrderUpdater::new();
        updater.visit_node_mut(root);
        self.optimizations_applied += 1;
    }

    /// Optimize indentation levels
    fn optimize_indentation(&mut self, _root: &mut SpatialNode) {
        // Placeholder: would analyze and optimize indentation patterns
        self.optimizations_applied += 1;
    }

    /// Optimize spatial layout
    fn optimize_spatial_layout(&mut self, _root: &mut SpatialNode) {
        // Placeholder: would optimize node positioning for better layout
        self.optimizations_applied += 1;
    }

    /// Compact metadata to reduce memory usage
    fn compact_metadata(&mut self, _root: &mut SpatialNode) {
        // Placeholder: would remove unnecessary metadata
        self.optimizations_applied += 1;
    }

    /// Get the number of optimizations applied
    pub fn optimizations_applied(&self) -> usize {
        self.optimizations_applied
    }
}

impl Default for SpatialOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts spatial information from ASTs
pub struct SpatialAnalyzer {
    current_depth: usize,
    analysis_results: AnalysisResults,
}

/// Results of spatial analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisResults {
    /// Spatial complexity score
    pub complexity_score: f64,
    /// Layout efficiency rating
    pub layout_efficiency: f64,
    /// Reading flow quality
    pub reading_flow_quality: f64,
    /// Identified layout issues
    pub layout_issues: Vec<LayoutIssue>,
    /// Optimization recommendations
    pub recommendations: Vec<OptimizationRecommendation>,
}

/// Layout issues that can be detected
#[derive(Debug, Clone)]
pub enum LayoutIssue {
    /// Excessive nesting depth
    ExcessiveNesting { depth: usize, recommendation: String },
    /// Poor indentation consistency
    InconsistentIndentation { locations: Vec<Span2D> },
    /// Suboptimal reading flow
    PoorReadingFlow { affected_nodes: Vec<super::NodeId> },
    /// Inefficient space usage
    SpaceWastage { wasted_area: f64 },
}

/// Optimization recommendations
#[derive(Debug, Clone)]
pub enum OptimizationRecommendation {
    /// Reduce nesting levels
    ReduceNesting { target_depth: usize },
    /// Improve indentation consistency
    StandardizeIndentation { recommended_style: String },
    /// Reorganize for better vertical flow
    ImproveFlow { suggested_reordering: Vec<super::NodeId> },
    /// Compact layout
    CompactLayout { compactable_regions: Vec<Span2D> },
}

impl SpatialAnalyzer {
    /// Create a new spatial analyzer
    pub fn new() -> Self {
        Self {
            current_depth: 0,
            analysis_results: AnalysisResults::default(),
        }
    }

    /// Analyze a spatial AST program
    pub fn analyze_program(&mut self, program: &SpatialProgram) -> Result<AnalysisResults> {
        self.analysis_results = AnalysisResults::default();
        self.current_depth = 0;

        // Perform various analyses
        self.analyze_complexity(&program.spatial_root);
        self.analyze_layout_efficiency(&program.spatial_root);
        self.analyze_reading_flow(&program.spatial_root);
        self.generate_recommendations();

        Ok(self.analysis_results.clone())
    }

    /// Analyze spatial complexity
    fn analyze_complexity(&mut self, _root: &SpatialNode) {
        // Calculate complexity based on depth, node count, etc.
        let metrics = super::visitor::queries::calculate_metrics(_root);
        self.analysis_results.complexity_score = metrics.complexity_score();
    }

    /// Analyze layout efficiency
    fn analyze_layout_efficiency(&mut self, _root: &SpatialNode) {
        // Analyze how efficiently space is used
        self.analysis_results.layout_efficiency = 0.8; // Placeholder
    }

    /// Analyze reading flow quality
    fn analyze_reading_flow(&mut self, _root: &SpatialNode) {
        // Analyze how well the code flows for vertical reading
        self.analysis_results.reading_flow_quality = 0.7; // Placeholder
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&mut self) {
        // Generate recommendations based on analysis
        if self.analysis_results.complexity_score > 10.0 {
            self.analysis_results.recommendations.push(
                OptimizationRecommendation::ReduceNesting { target_depth: 3 }
            );
        }

        if self.analysis_results.layout_efficiency < 0.6 {
            self.analysis_results.recommendations.push(
                OptimizationRecommendation::CompactLayout { 
                    compactable_regions: Vec::new() 
                }
            );
        }
    }
}

impl Default for SpatialAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts spatial ASTs back to regular ASTs
pub struct SpatialDeTransformer;

impl SpatialDeTransformer {
    /// Create a new de-transformer
    pub fn new() -> Self {
        Self
    }

    /// Convert a spatial program back to a regular AST program
    pub fn detransform_program(&self, spatial_program: &SpatialProgram) -> Result<ast::Program> {
        Ok(spatial_program.program.clone())
    }

    /// Extract regular expression from spatial expression
    pub fn detransform_expression(&self, spatial_expr: &super::SpatialExpression) -> ast::Expression {
        spatial_expr.expr.clone()
    }

    /// Extract regular statement from spatial statement  
    pub fn detransform_statement(&self, spatial_stmt: &super::SpatialStatement) -> ast::Statement {
        spatial_stmt.stmt.clone()
    }

    /// Extract regular declaration from spatial declaration
    pub fn detransform_declaration(&self, spatial_decl: &super::SpatialDeclaration) -> ast::Declaration {
        spatial_decl.decl.clone()
    }
}

impl Default for SpatialDeTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilities for working with spatial transformations
pub mod transform_utils {
    use super::*;

    /// Transform tokens to spatial positions
    pub fn tokens_to_positions(tokens: &[SpatialToken]) -> Vec<(Position2D, String)> {
        tokens.iter()
            .map(|token| (token.span.start, token.content.clone()))
            .collect()
    }

    /// Estimate span from token sequence
    pub fn estimate_span(tokens: &[SpatialToken]) -> Option<Span2D> {
        if tokens.is_empty() {
            return None;
        }

        let start = tokens.first().unwrap().span.start;
        let end = tokens.last().unwrap().span.end;
        Some(Span2D::new(start, end))
    }

    /// Merge adjacent spans
    pub fn merge_spans(spans: &[Span2D]) -> Option<Span2D> {
        if spans.is_empty() {
            return None;
        }

        let mut min_start = spans[0].start;
        let mut max_end = spans[0].end;

        for span in spans.iter().skip(1) {
            if span.start < min_start {
                min_start = span.start;
            }
            if span.end > max_end {
                max_end = span.end;
            }
        }

        Some(Span2D::new(min_start, max_end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertical::{SpatialTokenKind, WritingDirection};

    #[test]
    fn test_spatial_transformer() {
        let mut transformer = SpatialTransformer::new();
        let expr = ast::Expression::Literal(ast::Literal::Integer(42));
        let span = Span2D::new(Position2D::new(0, 0, 0), Position2D::new(2, 0, 2));

        let spatial_node = transformer.transform_expression(&expr, span).unwrap();
        
        assert_eq!(spatial_node.span, span);
        assert!(matches!(spatial_node.content, SpatialContent::Expression(_)));
    }

    #[test]
    fn test_spatial_optimizer() {
        let program = ast::Program { declarations: Vec::new() };
        let root_span = Span2D::new(Position2D::origin(), Position2D::new(10, 10, 100));
        let spatial_root = SpatialNode::new(1, root_span, SpatialContent::Program, Vec::new());
        let layout = CodeLayout::new(WritingDirection::VerticalTbRl);
        let source_info = SourceInfo::new(None, "test".to_string(), WritingDirection::VerticalTbRl);
        let mut spatial_program = SpatialProgram::new(program, spatial_root, layout, source_info);

        let mut optimizer = SpatialOptimizer::new();
        let optimizations = optimizer.optimize_program(&mut spatial_program).unwrap();
        
        assert!(optimizations > 0);
        assert_eq!(optimizer.optimizations_applied(), optimizations);
    }

    #[test]
    fn test_spatial_analyzer() {
        let program = ast::Program { declarations: Vec::new() };
        let root_span = Span2D::new(Position2D::origin(), Position2D::new(10, 10, 100));
        let spatial_root = SpatialNode::new(1, root_span, SpatialContent::Program, Vec::new());
        let layout = CodeLayout::new(WritingDirection::VerticalTbRl);
        let source_info = SourceInfo::new(None, "test".to_string(), WritingDirection::VerticalTbRl);
        let spatial_program = SpatialProgram::new(program, spatial_root, layout, source_info);

        let mut analyzer = SpatialAnalyzer::new();
        let results = analyzer.analyze_program(&spatial_program).unwrap();
        
        assert!(results.complexity_score >= 0.0);
        assert!(results.layout_efficiency >= 0.0);
        assert!(results.reading_flow_quality >= 0.0);
    }

    #[test]
    fn test_spatial_detransformer() {
        let detransformer = SpatialDeTransformer::new();
        
        let program = ast::Program { declarations: Vec::new() };
        let root_span = Span2D::new(Position2D::origin(), Position2D::new(10, 10, 100));
        let spatial_root = SpatialNode::new(1, root_span, SpatialContent::Program, Vec::new());
        let layout = CodeLayout::new(WritingDirection::VerticalTbRl);
        let source_info = SourceInfo::new(None, "test".to_string(), WritingDirection::VerticalTbRl);
        let spatial_program = SpatialProgram::new(program.clone(), spatial_root, layout, source_info);

        let detransformed = detransformer.detransform_program(&spatial_program).unwrap();
        
        // Should be the same as the original program
        assert_eq!(detransformed.declarations.len(), program.declarations.len());
    }

    #[test]
    fn test_transform_utils() {
        let tokens = vec![
            SpatialToken::new(
                "test".to_string(),
                Span2D::new(Position2D::new(0, 0, 0), Position2D::new(4, 0, 4)),
                SpatialTokenKind::Ascii,
                WritingDirection::VerticalTbRl,
            ),
        ];

        let positions = transform_utils::tokens_to_positions(&tokens);
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].1, "test");

        let span = transform_utils::estimate_span(&tokens).unwrap();
        assert_eq!(span.start, Position2D::new(0, 0, 0));
        assert_eq!(span.end, Position2D::new(4, 0, 4));
    }
}