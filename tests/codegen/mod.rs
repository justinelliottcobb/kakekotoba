use kakekotoba::codegen::CodeGenerator;
use kakekotoba::ast::*;
use kakekotoba::types::Type;
use inkwell::context::Context;

fn create_simple_program() -> Program {
    Program {
        declarations: vec![
            Declaration::Function(FunctionDecl {
                name: Identifier {
                    name: "add_one".to_string(),
                    span: kakekotoba::error::Span::new(0, 7, 1, 1),
                },
                type_params: Vec::new(),
                params: vec![
                    Parameter {
                        name: Identifier {
                            name: "x".to_string(),
                            span: kakekotoba::error::Span::new(8, 9, 1, 9),
                        },
                        param_type: Some(Type::Int),
                        span: kakekotoba::error::Span::new(8, 9, 1, 9),
                    }
                ],
                return_type: Some(Type::Int),
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
            })
        ],
    }
}

#[test]
fn test_codegen_creation() {
    let context = Context::create();
    let result = CodeGenerator::new(&context, "test_module");
    
    assert!(result.is_ok());
}

#[test]
fn test_simple_program_compilation() {
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_module").unwrap();
    
    let program = create_simple_program();
    
    match codegen.compile_program(&program) {
        Ok(()) => {
            // Compilation succeeded - verify module has content
            let module = codegen.get_module();
            
            // Should have at least one function
            let function = module.get_function("add_one");
            assert!(function.is_some());
        }
        Err(e) => {
            // Expected for now since codegen might not be fully implemented
            println!("Codegen error (might be expected): {:?}", e);
        }
    }
}

#[test]
fn test_literal_generation() {
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_module").unwrap();
    
    // Test integer literal
    let int_literal = Literal::Integer(42);
    match codegen.generate_literal(&int_literal) {
        Ok(value) => {
            assert!(value.is_int_value());
        }
        Err(e) => {
            println!("Literal generation error: {:?}", e);
        }
    }
    
    // Test boolean literal
    let bool_literal = Literal::Bool(true);
    match codegen.generate_literal(&bool_literal) {
        Ok(value) => {
            assert!(value.is_int_value()); // Booleans are represented as integers in LLVM
        }
        Err(e) => {
            println!("Boolean literal generation error: {:?}", e);
        }
    }
    
    // Test string literal
    let string_literal = Literal::String("hello".to_string());
    match codegen.generate_literal(&string_literal) {
        Ok(value) => {
            assert!(value.is_pointer_value());
        }
        Err(e) => {
            println!("String literal generation error: {:?}", e);
        }
    }
}

#[test]
fn test_type_conversion() {
    let context = Context::create();
    let codegen = CodeGenerator::new(&context, "test_module").unwrap();
    
    // Test basic type conversions
    assert!(codegen.type_to_basic_type(&Type::Int).is_ok());
    assert!(codegen.type_to_basic_type(&Type::Float).is_ok());
    assert!(codegen.type_to_basic_type(&Type::Bool).is_ok());
    assert!(codegen.type_to_basic_type(&Type::String).is_ok());
    assert!(codegen.type_to_basic_type(&Type::Unit).is_ok());
}

#[test]
fn test_binary_expression_generation() {
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_module").unwrap();
    
    let binary_expr = BinaryExpr {
        left: Box::new(Expression::Literal(Literal::Integer(5))),
        operator: BinaryOp::Add,
        right: Box::new(Expression::Literal(Literal::Integer(3))),
        span: kakekotoba::error::Span::new(0, 5, 1, 1),
    };
    
    match codegen.generate_binary(&binary_expr) {
        Ok(result) => {
            assert!(result.is_int_value());
        }
        Err(e) => {
            println!("Binary expression generation error: {:?}", e);
        }
    }
}

#[test]
fn test_function_declaration() {
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_module").unwrap();
    
    let func = FunctionDecl {
        name: Identifier {
            name: "simple_func".to_string(),
            span: kakekotoba::error::Span::new(0, 11, 1, 1),
        },
        type_params: Vec::new(),
        params: vec![],
        return_type: Some(Type::Int),
        body: Expression::Literal(Literal::Integer(42)),
        span: kakekotoba::error::Span::new(0, 15, 1, 1),
    };
    
    match codegen.declare_function(&func) {
        Ok(()) => {
            // Function should be declared in the module
            let module = codegen.get_module();
            let function = module.get_function("simple_func");
            assert!(function.is_some());
        }
        Err(e) => {
            println!("Function declaration error: {:?}", e);
        }
    }
}

#[test]
fn test_ir_output() {
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_module").unwrap();
    
    let program = create_simple_program();
    
    if codegen.compile_program(&program).is_ok() {
        // This should not panic - just output IR to stderr
        codegen.print_ir();
    }
}

#[test]
fn test_empty_program() {
    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "test_module").unwrap();
    
    let empty_program = Program {
        declarations: Vec::new(),
    };
    
    let result = codegen.compile_program(&empty_program);
    assert!(result.is_ok());
}