//! Spatial AST node definitions with 2D positioning

use crate::ast;
use crate::types::Type as KakeType;
use crate::vertical::Span2D;
use serde::{Deserialize, Serialize};

/// Expression with spatial information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialExpression {
    /// The wrapped expression
    pub expr: ast::Expression,
    /// Additional spatial properties
    pub spatial_props: ExpressionSpatialProps,
}

/// Spatial properties specific to expressions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpressionSpatialProps {
    /// Visual precedence level for layout
    pub precedence_level: usize,
    /// Whether this expression spans multiple lines
    pub multiline: bool,
    /// Operator alignment preference
    pub operator_alignment: OperatorAlignment,
    /// Parentheses positioning
    pub paren_style: ParenStyle,
}

/// Operator alignment styles for vertical text
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum OperatorAlignment {
    /// Align with the first operand
    Leading,
    /// Align with the last operand
    Trailing,
    /// Center between operands
    Center,
    /// Default/automatic alignment
    #[default]
    Auto,
}

/// Parentheses styling for vertical layout
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ParenStyle {
    /// Standard horizontal parentheses ()
    #[default]
    Horizontal,
    /// Vertical parentheses style
    Vertical,
    /// Japanese-style parentheses （）
    Japanese,
    /// Brackets []
    Brackets,
    /// Braces {}
    Braces,
}

/// Statement with spatial information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialStatement {
    /// The wrapped statement
    pub stmt: ast::Statement,
    /// Additional spatial properties
    pub spatial_props: StatementSpatialProps,
}

/// Spatial properties specific to statements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatementSpatialProps {
    /// Indentation style for this statement
    pub indentation_style: IndentationStyle,
    /// Whether this statement requires a block
    pub requires_block: bool,
    /// Statement termination style
    pub termination: StatementTermination,
    /// Flow control properties
    pub flow_control: FlowControlProps,
}

/// Indentation styles for statements
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum IndentationStyle {
    /// Block indentation (increase level)
    Block,
    /// Continuation indentation (align with previous)
    Continuation,
    /// No indentation change
    #[default]
    None,
    /// Custom indentation level
    Custom(usize),
}

/// Statement termination styles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum StatementTermination {
    /// Semicolon termination
    #[default]
    Semicolon,
    /// Newline termination
    Newline,
    /// Block termination (no explicit terminator)
    Block,
    /// Japanese punctuation (。)
    JapanesePeriod,
}

/// Flow control properties for statements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlowControlProps {
    /// Whether this statement can be the target of a jump
    pub is_jump_target: bool,
    /// Whether this statement breaks flow
    pub breaks_flow: bool,
    /// Nesting level for control structures
    pub nesting_level: usize,
}

/// Declaration with spatial information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialDeclaration {
    /// The wrapped declaration
    pub decl: ast::Declaration,
    /// Additional spatial properties
    pub spatial_props: DeclarationSpatialProps,
}

/// Spatial properties specific to declarations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeclarationSpatialProps {
    /// Visibility scope for layout purposes
    pub visibility_scope: VisibilityScope,
    /// Whether this declaration starts a new section
    pub starts_section: bool,
    /// Documentation positioning
    pub doc_position: DocumentationPosition,
    /// Export/import layout preferences
    pub module_layout: ModuleLayoutProps,
}

/// Visibility scopes affecting layout
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum VisibilityScope {
    /// Private to current module
    #[default]
    Private,
    /// Public to parent module
    Public,
    /// Exported globally
    Export,
    /// Internal/implementation detail
    Internal,
}

/// Documentation positioning preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DocumentationPosition {
    /// Above the declaration
    #[default]
    Above,
    /// To the side (right in horizontal, left in vertical)
    Side,
    /// Below the declaration
    Below,
    /// Inline with the declaration
    Inline,
}

/// Module layout properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleLayoutProps {
    /// Import grouping style
    pub import_grouping: ImportGrouping,
    /// Export listing style
    pub export_style: ExportStyle,
    /// Module boundary markers
    pub boundary_markers: bool,
}

/// Import grouping styles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ImportGrouping {
    /// Group by module
    #[default]
    ByModule,
    /// Group by functionality
    ByFunction,
    /// Single flat list
    Flat,
    /// Alphabetical ordering
    Alphabetical,
}

/// Export listing styles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ExportStyle {
    /// List exports explicitly
    #[default]
    Explicit,
    /// Use wildcard exports
    Wildcard,
    /// Selective re-exports
    Selective,
}

/// Type with spatial information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialType {
    /// The wrapped type
    pub type_info: KakeType,
    /// Additional spatial properties
    pub spatial_props: TypeSpatialProps,
}

/// Spatial properties specific to types
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeSpatialProps {
    /// Type layout complexity
    pub complexity: TypeComplexity,
    /// Constructor layout preferences
    pub constructor_layout: ConstructorLayout,
    /// Generic parameter positioning
    pub generic_positioning: GenericPositioning,
    /// Type annotation style
    pub annotation_style: AnnotationStyle,
}

/// Type layout complexity levels
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TypeComplexity {
    /// Simple types (primitives, single identifiers)
    #[default]
    Simple,
    /// Compound types (structs, tuples)
    Compound,
    /// Complex types (functions, generics)
    Complex,
    /// Higher-kinded types
    HigherKinded,
}

/// Constructor layout preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ConstructorLayout {
    /// Horizontal layout (traditional)
    #[default]
    Horizontal,
    /// Vertical layout (stacked)
    Vertical,
    /// Tabular layout (aligned columns)
    Tabular,
    /// Flow layout (wrap as needed)
    Flow,
}

/// Generic parameter positioning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum GenericPositioning {
    /// Angle brackets <T>
    #[default]
    AngleBrackets,
    /// Square brackets [T]
    SquareBrackets,
    /// Parentheses (T)
    Parentheses,
    /// Subscript style
    Subscript,
    /// Superscript style
    Superscript,
}

/// Type annotation styles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AnnotationStyle {
    /// Colon separator :
    #[default]
    Colon,
    /// Arrow separator →
    Arrow,
    /// Japanese style separator は
    JapaneseWa,
    /// No explicit separator
    Implicit,
}

/// Spatial literal with positioning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialLiteral {
    /// The literal value
    pub value: ast::Literal,
    /// Span of the literal
    pub span: Span2D,
    /// Literal-specific spatial properties
    pub spatial_props: LiteralSpatialProps,
}

/// Spatial properties for literals
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LiteralSpatialProps {
    /// Numeric base for number literals
    pub numeric_base: NumericBase,
    /// String quotation style
    pub quote_style: QuoteStyle,
    /// Whether the literal spans multiple lines
    pub multiline: bool,
}

/// Numeric base representations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum NumericBase {
    /// Decimal (base 10)
    #[default]
    Decimal,
    /// Binary (base 2)
    Binary,
    /// Octal (base 8)
    Octal,
    /// Hexadecimal (base 16)
    Hexadecimal,
    /// Japanese numerals
    Japanese,
}

/// String quotation styles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum QuoteStyle {
    /// Double quotes "
    #[default]
    Double,
    /// Single quotes '
    Single,
    /// Japanese quotes 「」
    Japanese,
    /// Corner brackets 『』
    JapaneseCorner,
    /// Backticks `
    Backtick,
}

/// Spatial identifier with full positioning context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialIdentifier {
    /// The identifier name
    pub name: String,
    /// Position span
    pub span: Span2D,
    /// Identifier properties
    pub properties: IdentifierProperties,
}

/// Properties of spatial identifiers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentifierProperties {
    /// Scope where this identifier is defined
    pub scope_level: usize,
    /// Whether this is a keyword in Japanese
    pub is_japanese_keyword: bool,
    /// Script used for this identifier
    pub script: crate::japanese::JapaneseScript,
    /// Whether this identifier has special semantic meaning
    pub is_special: bool,
}

impl SpatialExpression {
    /// Create a new spatial expression
    pub fn new(expr: ast::Expression, spatial_props: ExpressionSpatialProps) -> Self {
        Self {
            expr,
            spatial_props,
        }
    }

    /// Check if this expression is multiline
    pub fn is_multiline(&self) -> bool {
        self.spatial_props.multiline
    }

    /// Get the precedence level for layout purposes
    pub fn precedence_level(&self) -> usize {
        self.spatial_props.precedence_level
    }
}

impl SpatialStatement {
    /// Create a new spatial statement
    pub fn new(stmt: ast::Statement, spatial_props: StatementSpatialProps) -> Self {
        Self {
            stmt,
            spatial_props,
        }
    }

    /// Check if this statement requires a block structure
    pub fn requires_block(&self) -> bool {
        self.spatial_props.requires_block
    }

    /// Get the nesting level for flow control
    pub fn nesting_level(&self) -> usize {
        self.spatial_props.flow_control.nesting_level
    }
}

impl SpatialDeclaration {
    /// Create a new spatial declaration
    pub fn new(decl: ast::Declaration, spatial_props: DeclarationSpatialProps) -> Self {
        Self {
            decl,
            spatial_props,
        }
    }

    /// Check if this declaration starts a new section
    pub fn starts_section(&self) -> bool {
        self.spatial_props.starts_section
    }

    /// Get the visibility scope
    pub fn visibility_scope(&self) -> &VisibilityScope {
        &self.spatial_props.visibility_scope
    }
}

impl SpatialType {
    /// Create a new spatial type
    pub fn new(type_info: KakeType, spatial_props: TypeSpatialProps) -> Self {
        Self {
            type_info,
            spatial_props,
        }
    }

    /// Get the complexity level of this type
    pub fn complexity(&self) -> &TypeComplexity {
        &self.spatial_props.complexity
    }

    /// Check if this type has complex layout requirements
    pub fn is_complex_layout(&self) -> bool {
        matches!(
            self.spatial_props.complexity,
            TypeComplexity::Complex | TypeComplexity::HigherKinded
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_expression() {
        let expr = ast::Expression::Literal(ast::Literal::Integer(42));
        let mut props = ExpressionSpatialProps::default();
        props.multiline = true;
        props.precedence_level = 5;

        let spatial_expr = SpatialExpression::new(expr, props);

        assert!(spatial_expr.is_multiline());
        assert_eq!(spatial_expr.precedence_level(), 5);
    }

    #[test]
    fn test_spatial_statement() {
        let stmt = ast::Statement::Expression(ast::Expression::Literal(ast::Literal::Integer(42)));
        let mut props = StatementSpatialProps::default();
        props.requires_block = true;
        props.flow_control.nesting_level = 2;

        let spatial_stmt = SpatialStatement::new(stmt, props);

        assert!(spatial_stmt.requires_block());
        assert_eq!(spatial_stmt.nesting_level(), 2);
    }

    #[test]
    fn test_spatial_declaration() {
        let decl = ast::Declaration::Function(ast::FunctionDecl {
            name: ast::Identifier {
                name: "test".to_string(),
                span: crate::error::Span::new(0, 4, 1, 1),
            },
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: None,
            body: ast::Expression::Literal(ast::Literal::Integer(42)),
            span: crate::error::Span::new(0, 10, 1, 1),
        });

        let mut props = DeclarationSpatialProps::default();
        props.starts_section = true;
        props.visibility_scope = VisibilityScope::Public;

        let spatial_decl = SpatialDeclaration::new(decl, props);

        assert!(spatial_decl.starts_section());
        assert!(matches!(
            spatial_decl.visibility_scope(),
            VisibilityScope::Public
        ));
    }

    #[test]
    fn test_spatial_type() {
        let type_info = KakeType::Int;
        let mut props = TypeSpatialProps::default();
        props.complexity = TypeComplexity::Complex;

        let spatial_type = SpatialType::new(type_info, props);

        assert!(spatial_type.is_complex_layout());
        assert!(matches!(spatial_type.complexity(), TypeComplexity::Complex));
    }

    #[test]
    fn test_default_implementations() {
        let _ = OperatorAlignment::default();
        let _ = ParenStyle::default();
        let _ = IndentationStyle::default();
        let _ = StatementTermination::default();
        let _ = VisibilityScope::default();
        let _ = DocumentationPosition::default();
        let _ = ImportGrouping::default();
        let _ = ExportStyle::default();
        let _ = TypeComplexity::default();
        let _ = ConstructorLayout::default();
        let _ = GenericPositioning::default();
        let _ = AnnotationStyle::default();
        let _ = NumericBase::default();
        let _ = QuoteStyle::default();
    }
}
