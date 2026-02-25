use kakekotoba::ast::*;
use kakekotoba::codegen::CodeGenerator;

fn create_simple_program() -> Program {
    Program {
        declarations: vec![Declaration::Function(FunctionDecl {
            name: Identifier {
                name: "add_one".to_string(),
                span: kakekotoba::error::Span::new(0, 7, 1, 1),
            },
            type_params: Vec::new(),
            params: vec![Parameter {
                name: Identifier {
                    name: "x".to_string(),
                    span: kakekotoba::error::Span::new(8, 9, 1, 9),
                },
                param_type: Some(kakekotoba::types::Type::Int),
                span: kakekotoba::error::Span::new(8, 9, 1, 9),
            }],
            return_type: Some(kakekotoba::types::Type::Int),
            body: Expression::Binary(BinaryExpr {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                    span: kakekotoba::error::Span::new(10, 11, 1, 11),
                })),
                operator: BinaryOp::Add,
                right: Box::new(Expression::Literal(Literal::Integer(1))),
                span: kakekotoba::error::Span::new(10, 13, 1, 11),
            }),
            span: kakekotoba::error::Span::new(0, 13, 1, 1),
        })],
    }
}

#[test]
fn test_codegen_creation() {
    let result = CodeGenerator::new("test_module");
    assert!(result.is_ok());
}

#[test]
fn test_compile_program_is_stubbed() {
    let mut codegen = CodeGenerator::new("test_module").unwrap();
    let program = create_simple_program();

    // Codegen is currently stubbed out (see docs/ROADMAP.md Phase 3-4)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        codegen.compile_program(&program)
    }));

    // Should panic with todo!()
    assert!(result.is_err());
}

#[test]
fn test_empty_program_stubbed() {
    let mut codegen = CodeGenerator::new("test_module").unwrap();
    let empty_program = Program {
        declarations: Vec::new(),
    };

    // Codegen is currently stubbed out
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        codegen.compile_program(&empty_program)
    }));

    // Should panic with todo!()
    assert!(result.is_err());
}
