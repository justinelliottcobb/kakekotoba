use crate::ast::*;
use crate::error::{Error, Result};
use crate::types::Type;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::{AnyType, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{
    AnyValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue,
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};
use std::collections::HashMap;

pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,

    // Symbol tables
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Result<Self> {
        let module = context.create_module(module_name);
        let execution_engine = module
            .create_jit_execution_engine(inkwell::OptimizationLevel::None)
            .map_err(|e| Error::Codegen {
                message: format!("Failed to create execution engine: {}", e),
            })?;

        let builder = context.create_builder();

        Ok(Self {
            context,
            module,
            builder,
            execution_engine,
            variables: HashMap::new(),
            functions: HashMap::new(),
        })
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<()> {
        // First pass: declare all functions
        for declaration in &program.declarations {
            if let Declaration::Function(func) = declaration {
                self.declare_function(func)?;
            }
        }

        // Second pass: generate function bodies
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(func) => {
                    self.generate_function(func)?;
                }
                Declaration::Type(_) => {
                    // Type declarations don't generate code directly
                }
                Declaration::Import(_) => {
                    // Handle imports
                }
            }
        }

        Ok(())
    }

    fn declare_function(&mut self, func: &FunctionDecl) -> Result<()> {
        let param_types: Result<Vec<BasicMetadataTypeEnum>> = func
            .params
            .iter()
            .map(|p| self.type_to_basic_metadata_type(&p.param_type.as_ref().unwrap_or(&Type::Int)))
            .collect();

        let param_types = param_types?;

        let return_type = func.return_type.as_ref().unwrap_or(&Type::Unit);
        let return_llvm_type = self.type_to_basic_type(return_type)?;

        let function_type = return_llvm_type.fn_type(&param_types, false);
        let function = self
            .module
            .add_function(&func.name.name, function_type, None);

        self.functions.insert(func.name.name.clone(), function);

        Ok(())
    }

    fn generate_function(&mut self, func: &FunctionDecl) -> Result<()> {
        let function = self.functions[&func.name.name];
        let entry_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(entry_block);

        // Clear local variables for new function scope
        self.variables.clear();

        // Create allocas for parameters
        for (i, param) in func.params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            let param_type = param.param_type.as_ref().unwrap_or(&Type::Int);
            let llvm_type = self.type_to_basic_type(param_type)?;

            let alloca = self
                .builder
                .build_alloca(llvm_type, &param.name.name)
                .map_err(|e| Error::Codegen {
                    message: e.to_string(),
                })?;

            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| Error::Codegen {
                    message: e.to_string(),
                })?;

            self.variables.insert(param.name.name.clone(), alloca);
        }

        // Generate function body
        let body_value = self.generate_expression(&func.body)?;

        // Return the result
        if let Some(value) = body_value {
            self.builder
                .build_return(Some(&value))
                .map_err(|e| Error::Codegen {
                    message: e.to_string(),
                })?;
        } else {
            self.builder
                .build_return(None)
                .map_err(|e| Error::Codegen {
                    message: e.to_string(),
                })?;
        }

        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<Option<BasicValueEnum<'ctx>>> {
        match expr {
            Expression::Literal(lit) => Ok(Some(self.generate_literal(lit)?)),
            Expression::Identifier(id) => self.generate_identifier(id),
            Expression::Application(app) => self.generate_application(app),
            Expression::Lambda(_) => {
                // TODO: Implement lambda generation (requires closure conversion)
                Err(Error::Codegen {
                    message: "Lambda expressions not yet supported".to_string(),
                })
            }
            Expression::Let(_) => {
                // TODO: Implement let expressions
                Err(Error::Codegen {
                    message: "Let expressions not yet supported".to_string(),
                })
            }
            Expression::If(if_expr) => self.generate_if(if_expr),
            Expression::Match(_) => {
                // TODO: Implement pattern matching
                Err(Error::Codegen {
                    message: "Match expressions not yet supported".to_string(),
                })
            }
            Expression::Block(block) => self.generate_block(block),
            Expression::Binary(binary) => Ok(Some(self.generate_binary(binary)?)),
            Expression::Unary(unary) => Ok(Some(self.generate_unary(unary)?)),
        }
    }

    fn generate_literal(&self, lit: &Literal) -> Result<BasicValueEnum<'ctx>> {
        match lit {
            Literal::Integer(n) => {
                let int_type = self.context.i64_type();
                Ok(int_type.const_int(*n as u64, true).into())
            }
            Literal::Float(f) => {
                let float_type = self.context.f64_type();
                Ok(float_type.const_float(*f).into())
            }
            Literal::String(s) => {
                let string_value = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;
                Ok(string_value.as_pointer_value().into())
            }
            Literal::Bool(b) => {
                let bool_type = self.context.bool_type();
                Ok(bool_type.const_int(if *b { 1 } else { 0 }, false).into())
            }
            Literal::Unit => {
                // Unit type can be represented as void, but we need a value
                // Use a zero-sized struct or null pointer
                let void_ptr = self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .const_null();
                Ok(void_ptr.into())
            }
        }
    }

    fn generate_identifier(&mut self, id: &Identifier) -> Result<Option<BasicValueEnum<'ctx>>> {
        if let Some(ptr) = self.variables.get(&id.name) {
            let value = self
                .builder
                .build_load(ptr.get_type().get_element_type(), *ptr, &id.name)
                .map_err(|e| Error::Codegen {
                    message: e.to_string(),
                })?;
            Ok(Some(value))
        } else {
            Err(Error::Codegen {
                message: format!("Undefined variable: {}", id.name),
            })
        }
    }

    fn generate_application(&mut self, app: &Application) -> Result<Option<BasicValueEnum<'ctx>>> {
        if let Expression::Identifier(func_name) = &*app.function {
            if let Some(function) = self.functions.get(&func_name.name) {
                let mut args = Vec::new();

                for arg in &app.arguments {
                    if let Some(value) = self.generate_expression(arg)? {
                        args.push(BasicMetadataValueEnum::from(value));
                    }
                }

                let result = self
                    .builder
                    .build_call(*function, &args, "call")
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;

                Ok(result.try_as_basic_value().left())
            } else {
                Err(Error::Codegen {
                    message: format!("Undefined function: {}", func_name.name),
                })
            }
        } else {
            Err(Error::Codegen {
                message: "Complex function expressions not yet supported".to_string(),
            })
        }
    }

    fn generate_if(&mut self, if_expr: &IfExpr) -> Result<Option<BasicValueEnum<'ctx>>> {
        let condition = self
            .generate_expression(&if_expr.condition)?
            .ok_or_else(|| Error::Codegen {
                message: "If condition must produce a value".to_string(),
            })?;

        let current_function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let then_block = self.context.append_basic_block(current_function, "then");
        let else_block = self.context.append_basic_block(current_function, "else");
        let merge_block = self.context.append_basic_block(current_function, "merge");

        // Build conditional branch
        self.builder
            .build_conditional_branch(condition.into_int_value(), then_block, else_block)
            .map_err(|e| Error::Codegen {
                message: e.to_string(),
            })?;

        // Generate then branch
        self.builder.position_at_end(then_block);
        let then_value = self.generate_expression(&if_expr.then_branch)?;
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| Error::Codegen {
                message: e.to_string(),
            })?;
        let then_block_end = self.builder.get_insert_block().unwrap();

        // Generate else branch
        self.builder.position_at_end(else_block);
        let else_value = if let Some(else_branch) = &if_expr.else_branch {
            self.generate_expression(else_branch)?
        } else {
            // Unit value for missing else branch
            Some(
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into(),
            )
        };
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| Error::Codegen {
                message: e.to_string(),
            })?;
        let else_block_end = self.builder.get_insert_block().unwrap();

        // Merge block with phi
        self.builder.position_at_end(merge_block);

        if then_value.is_some() || else_value.is_some() {
            let then_val = then_value.unwrap_or_else(|| {
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into()
            });
            let else_val = else_value.unwrap_or_else(|| {
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into()
            });

            let phi = self
                .builder
                .build_phi(then_val.get_type(), "if_result")
                .map_err(|e| Error::Codegen {
                    message: e.to_string(),
                })?;
            phi.add_incoming(&[(&then_val, then_block_end), (&else_val, else_block_end)]);

            Ok(Some(phi.as_basic_value()))
        } else {
            Ok(None)
        }
    }

    fn generate_block(&mut self, block: &Block) -> Result<Option<BasicValueEnum<'ctx>>> {
        let mut last_value = None;

        for statement in &block.statements {
            match statement {
                Statement::Expression(expr) => {
                    last_value = self.generate_expression(expr)?;
                }
                Statement::Let(_) => {
                    // TODO: Handle let bindings
                }
            }
        }

        if let Some(expr) = &block.expr {
            last_value = self.generate_expression(expr)?;
        }

        Ok(last_value)
    }

    fn generate_binary(&mut self, binary: &BinaryExpr) -> Result<BasicValueEnum<'ctx>> {
        let left = self
            .generate_expression(&binary.left)?
            .ok_or_else(|| Error::Codegen {
                message: "Left operand must produce a value".to_string(),
            })?;

        let right = self
            .generate_expression(&binary.right)?
            .ok_or_else(|| Error::Codegen {
                message: "Right operand must produce a value".to_string(),
            })?;

        match binary.operator {
            BinaryOp::Add => {
                if left.is_int_value() && right.is_int_value() {
                    let result = self
                        .builder
                        .build_int_add(left.into_int_value(), right.into_int_value(), "add")
                        .map_err(|e| Error::Codegen {
                            message: e.to_string(),
                        })?;
                    Ok(result.into())
                } else {
                    Err(Error::Codegen {
                        message: "Addition requires integer operands".to_string(),
                    })
                }
            }
            BinaryOp::Sub => {
                let result = self
                    .builder
                    .build_int_sub(left.into_int_value(), right.into_int_value(), "sub")
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;
                Ok(result.into())
            }
            BinaryOp::Mul => {
                let result = self
                    .builder
                    .build_int_mul(left.into_int_value(), right.into_int_value(), "mul")
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;
                Ok(result.into())
            }
            BinaryOp::Div => {
                let result = self
                    .builder
                    .build_int_signed_div(left.into_int_value(), right.into_int_value(), "div")
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;
                Ok(result.into())
            }
            BinaryOp::Eq => {
                let result = self
                    .builder
                    .build_int_compare(
                        IntPredicate::EQ,
                        left.into_int_value(),
                        right.into_int_value(),
                        "eq",
                    )
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;
                Ok(result.into())
            }
            BinaryOp::Lt => {
                let result = self
                    .builder
                    .build_int_compare(
                        IntPredicate::SLT,
                        left.into_int_value(),
                        right.into_int_value(),
                        "lt",
                    )
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;
                Ok(result.into())
            }
            _ => Err(Error::Codegen {
                message: format!("Binary operator {:?} not yet implemented", binary.operator),
            }),
        }
    }

    fn generate_unary(&mut self, unary: &UnaryExpr) -> Result<BasicValueEnum<'ctx>> {
        let operand = self
            .generate_expression(&unary.operand)?
            .ok_or_else(|| Error::Codegen {
                message: "Unary operand must produce a value".to_string(),
            })?;

        match unary.operator {
            UnaryOp::Neg => {
                let result = self
                    .builder
                    .build_int_neg(operand.into_int_value(), "neg")
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;
                Ok(result.into())
            }
            UnaryOp::Not => {
                let result = self
                    .builder
                    .build_not(operand.into_int_value(), "not")
                    .map_err(|e| Error::Codegen {
                        message: e.to_string(),
                    })?;
                Ok(result.into())
            }
        }
    }

    fn type_to_basic_type(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>> {
        match ty {
            Type::Int => Ok(self.context.i64_type().into()),
            Type::Float => Ok(self.context.f64_type().into()),
            Type::Bool => Ok(self.context.bool_type().into()),
            Type::String => Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into()),
            Type::Unit => Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into()), // Void pointer for unit
            _ => Err(Error::Codegen {
                message: format!("Type {:?} not yet supported in codegen", ty),
            }),
        }
    }

    fn type_to_basic_metadata_type(&self, ty: &Type) -> Result<BasicMetadataTypeEnum<'ctx>> {
        Ok(self.type_to_basic_type(ty)?.into())
    }

    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn print_ir(&self) {
        self.module.print_to_stderr();
    }
}
