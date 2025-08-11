use crate::ast::*;
use crate::error::{Error, Result};
use crate::types::*;
use petgraph::{Graph, Directed};
use std::collections::{HashMap, HashSet};

pub type InferenceGraph = Graph<TypeNode, ConstraintEdge, Directed>;

#[derive(Debug, Clone)]
pub struct TypeNode {
    pub id: usize,
    pub ty: Type,
    pub span: crate::error::Span,
}

#[derive(Debug, Clone)]
pub struct ConstraintEdge {
    pub kind: ConstraintKind,
    pub strength: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintKind {
    Equality,
    Subtype,
    Instance,
    Homomorphism,
}

pub struct TypeInference {
    context: TypeContext,
    graph: InferenceGraph,
    substitution: Substitution,
}

pub type Substitution = HashMap<usize, Type>;

impl TypeInference {
    pub fn new() -> Self {
        Self {
            context: TypeContext::new(),
            graph: Graph::new(),
            substitution: HashMap::new(),
        }
    }
    
    pub fn infer_program(&mut self, program: &Program) -> Result<HashMap<String, TypeScheme>> {
        let mut bindings = HashMap::new();
        
        // First pass: collect type signatures
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(func) => {
                    let scheme = self.infer_function_signature(func)?;
                    bindings.insert(func.name.name.clone(), scheme);
                },
                Declaration::Type(type_decl) => {
                    self.process_type_declaration(type_decl)?;
                },
                Declaration::Import(_) => {
                    // Handle imports
                },
            }
        }
        
        // Second pass: type check function bodies
        for declaration in &program.declarations {
            if let Declaration::Function(func) = declaration {
                self.type_check_function(func)?;
            }
        }
        
        // Solve constraints
        self.solve_constraints()?;
        
        Ok(bindings)
    }
    
    fn infer_function_signature(&mut self, func: &FunctionDecl) -> Result<TypeScheme> {
        self.context.env.enter_level();
        
        // Create type variables for parameters
        let mut param_types = Vec::new();
        for param in &func.params {
            let param_type = match &param.param_type {
                Some(ty) => ty.clone(),
                None => Type::Var(self.context.fresh_type_var()),
            };
            param_types.push(param_type);
        }
        
        // Determine return type
        let return_type = match &func.return_type {
            Some(ty) => ty.clone(),
            None => Type::Var(self.context.fresh_type_var()),
        };
        
        let function_type = Type::function(param_types, return_type);
        
        // Create type scheme with quantified variables
        let forall = self.get_generic_variables(&function_type);
        let scheme = TypeScheme {
            forall,
            ty: function_type,
        };
        
        self.context.env.exit_level();
        Ok(scheme)
    }
    
    fn process_type_declaration(&mut self, _type_decl: &TypeDecl) -> Result<()> {
        // TODO: Process type declarations
        Ok(())
    }
    
    fn type_check_function(&mut self, func: &FunctionDecl) -> Result<()> {
        self.context.env.enter_level();
        
        // Add parameters to environment
        for (param, param_type) in func.params.iter().zip(self.get_param_types(func)) {
            let scheme = TypeScheme {
                forall: Vec::new(),
                ty: param_type,
            };
            self.context.env.bind(param.name.name.clone(), scheme);
        }
        
        // Infer body type
        let body_type = self.infer_expression(&func.body)?;
        
        // Unify with declared return type if present
        if let Some(declared_return) = &func.return_type {
            let constraint = Constraint::new(
                declared_return.clone(),
                body_type,
                func.span.clone(),
            );
            self.context.add_constraint(constraint);
        }
        
        self.context.env.exit_level();
        Ok(())
    }
    
    fn infer_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(lit) => Ok(self.infer_literal(lit)),
            Expression::Identifier(id) => self.infer_identifier(id),
            Expression::Application(app) => self.infer_application(app),
            Expression::Lambda(lambda) => self.infer_lambda(lambda),
            Expression::Let(let_expr) => self.infer_let(let_expr),
            Expression::If(if_expr) => self.infer_if(if_expr),
            Expression::Match(match_expr) => self.infer_match(match_expr),
            Expression::Block(block) => self.infer_block(block),
            Expression::Binary(binary) => self.infer_binary(binary),
            Expression::Unary(unary) => self.infer_unary(unary),
        }
    }
    
    fn infer_literal(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Integer(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
            Literal::Unit => Type::Unit,
        }
    }
    
    fn infer_identifier(&mut self, id: &Identifier) -> Result<Type> {
        if let Some(scheme) = self.context.env.lookup(&id.name) {
            Ok(self.instantiate(scheme))
        } else {
            Err(Error::Type {
                src: String::new(), // TODO: Add source
                span: id.span.clone().into(),
                expected: "bound variable".to_string(),
                found: format!("unbound variable '{}'", id.name),
            })
        }
    }
    
    fn infer_application(&mut self, app: &Application) -> Result<Type> {
        let func_type = self.infer_expression(&app.function)?;
        let mut arg_types = Vec::new();
        
        for arg in &app.arguments {
            arg_types.push(self.infer_expression(arg)?);
        }
        
        let return_type = Type::Var(self.context.fresh_type_var());
        let expected_func_type = Type::function(arg_types, return_type.clone());
        
        let constraint = Constraint::new(
            expected_func_type,
            func_type,
            app.span.clone(),
        );
        self.context.add_constraint(constraint);
        
        Ok(return_type)
    }
    
    fn infer_lambda(&mut self, lambda: &Lambda) -> Result<Type> {
        self.context.env.enter_level();
        
        let mut param_types = Vec::new();
        for param in &lambda.params {
            let param_type = match &param.param_type {
                Some(ty) => ty.clone(),
                None => Type::Var(self.context.fresh_type_var()),
            };
            
            let scheme = TypeScheme {
                forall: Vec::new(),
                ty: param_type.clone(),
            };
            self.context.env.bind(param.name.name.clone(), scheme);
            param_types.push(param_type);
        }
        
        let body_type = self.infer_expression(&lambda.body)?;
        let lambda_type = Type::function(param_types, body_type);
        
        self.context.env.exit_level();
        Ok(lambda_type)
    }
    
    fn infer_let(&mut self, let_expr: &LetExpr) -> Result<Type> {
        // Placeholder implementation
        self.infer_expression(&let_expr.body)
    }
    
    fn infer_if(&mut self, if_expr: &IfExpr) -> Result<Type> {
        let condition_type = self.infer_expression(&if_expr.condition)?;
        let then_type = self.infer_expression(&if_expr.then_branch)?;
        
        // Condition must be boolean
        let bool_constraint = Constraint::new(
            Type::Bool,
            condition_type,
            if_expr.span.clone(),
        );
        self.context.add_constraint(bool_constraint);
        
        if let Some(else_branch) = &if_expr.else_branch {
            let else_type = self.infer_expression(else_branch)?;
            // Both branches must have same type
            let branch_constraint = Constraint::new(
                then_type.clone(),
                else_type,
                if_expr.span.clone(),
            );
            self.context.add_constraint(branch_constraint);
        }
        
        Ok(then_type)
    }
    
    fn infer_match(&mut self, _match_expr: &MatchExpr) -> Result<Type> {
        // TODO: Implement pattern matching type inference
        Ok(Type::Var(self.context.fresh_type_var()))
    }
    
    fn infer_block(&mut self, block: &Block) -> Result<Type> {
        // Type check all statements
        for stmt in &block.statements {
            match stmt {
                Statement::Expression(expr) => {
                    self.infer_expression(expr)?;
                },
                Statement::Let(_binding) => {
                    // TODO: Handle let bindings
                },
            }
        }
        
        // Return type of final expression or unit
        if let Some(expr) = &block.expr {
            self.infer_expression(expr)
        } else {
            Ok(Type::Unit)
        }
    }
    
    fn infer_binary(&mut self, binary: &BinaryExpr) -> Result<Type> {
        let left_type = self.infer_expression(&binary.left)?;
        let right_type = self.infer_expression(&binary.right)?;
        
        match binary.operator {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                // Arithmetic operations require numeric types
                let constraint1 = Constraint::new(Type::Int, left_type, binary.span.clone());
                let constraint2 = Constraint::new(Type::Int, right_type, binary.span.clone());
                self.context.add_constraint(constraint1);
                self.context.add_constraint(constraint2);
                Ok(Type::Int)
            },
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality requires same types
                let constraint = Constraint::new(left_type, right_type, binary.span.clone());
                self.context.add_constraint(constraint);
                Ok(Type::Bool)
            },
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                // Comparison requires ordered types (assume Int for now)
                let constraint1 = Constraint::new(Type::Int, left_type, binary.span.clone());
                let constraint2 = Constraint::new(Type::Int, right_type, binary.span.clone());
                self.context.add_constraint(constraint1);
                self.context.add_constraint(constraint2);
                Ok(Type::Bool)
            },
            BinaryOp::And | BinaryOp::Or => {
                // Logical operations require boolean types
                let constraint1 = Constraint::new(Type::Bool, left_type, binary.span.clone());
                let constraint2 = Constraint::new(Type::Bool, right_type, binary.span.clone());
                self.context.add_constraint(constraint1);
                self.context.add_constraint(constraint2);
                Ok(Type::Bool)
            },
            BinaryOp::Compose => {
                // Function composition
                // TODO: Implement proper function composition typing
                Ok(Type::Var(self.context.fresh_type_var()))
            },
            _ => Ok(Type::Var(self.context.fresh_type_var())),
        }
    }
    
    fn infer_unary(&mut self, unary: &UnaryExpr) -> Result<Type> {
        let operand_type = self.infer_expression(&unary.operand)?;
        
        match unary.operator {
            UnaryOp::Not => {
                let constraint = Constraint::new(Type::Bool, operand_type, unary.span.clone());
                self.context.add_constraint(constraint);
                Ok(Type::Bool)
            },
            UnaryOp::Neg => {
                let constraint = Constraint::new(Type::Int, operand_type, unary.span.clone());
                self.context.add_constraint(constraint);
                Ok(Type::Int)
            },
        }
    }
    
    fn solve_constraints(&mut self) -> Result<()> {
        for constraint in &self.context.constraints.clone() {
            self.unify(&constraint.expected, &constraint.actual, constraint.span.clone())?;
        }
        Ok(())
    }
    
    fn unify(&mut self, t1: &Type, t2: &Type, span: crate::error::Span) -> Result<()> {
        match (t1, t2) {
            (Type::Var(var), ty) | (ty, Type::Var(var)) => {
                if let Type::Var(other_var) = ty {
                    if var.id == other_var.id {
                        return Ok(());
                    }
                }
                
                if self.occurs_check(var.id, ty) {
                    return Err(Error::Type {
                        src: String::new(),
                        span: span.into(),
                        expected: "finite type".to_string(),
                        found: "infinite type".to_string(),
                    });
                }
                
                self.substitution.insert(var.id, ty.clone());
                Ok(())
            },
            (Type::Int, Type::Int) |
            (Type::Float, Type::Float) |
            (Type::String, Type::String) |
            (Type::Bool, Type::Bool) |
            (Type::Unit, Type::Unit) => Ok(()),
            
            (Type::Function { params: p1, return_type: r1 },
             Type::Function { params: p2, return_type: r2 }) => {
                if p1.len() != p2.len() {
                    return Err(Error::Type {
                        src: String::new(),
                        span: span.into(),
                        expected: format!("function with {} parameters", p1.len()),
                        found: format!("function with {} parameters", p2.len()),
                    });
                }
                
                for (param1, param2) in p1.iter().zip(p2.iter()) {
                    self.unify(param1, param2, span.clone())?;
                }
                
                self.unify(r1, r2, span)
            },
            
            _ => Err(Error::Type {
                src: String::new(),
                span: span.into(),
                expected: format!("{:?}", t1),
                found: format!("{:?}", t2),
            }),
        }
    }
    
    fn occurs_check(&self, var_id: usize, ty: &Type) -> bool {
        match ty {
            Type::Var(var) => var.id == var_id,
            Type::Function { params, return_type } => {
                params.iter().any(|p| self.occurs_check(var_id, p)) ||
                self.occurs_check(var_id, return_type)
            },
            Type::List(elem_type) => self.occurs_check(var_id, elem_type),
            Type::Tuple(types) => types.iter().any(|t| self.occurs_check(var_id, t)),
            _ => false,
        }
    }
    
    fn instantiate(&mut self, scheme: &TypeScheme) -> Type {
        let mut substitution = HashMap::new();
        
        for type_var in &scheme.forall {
            let fresh_var = self.context.fresh_type_var();
            substitution.insert(type_var.id, Type::Var(fresh_var));
        }
        
        self.substitute_type(&scheme.ty, &substitution)
    }
    
    fn substitute_type(&self, ty: &Type, substitution: &HashMap<usize, Type>) -> Type {
        match ty {
            Type::Var(var) => {
                substitution.get(&var.id).cloned().unwrap_or(ty.clone())
            },
            Type::Function { params, return_type } => {
                let new_params = params.iter()
                    .map(|p| self.substitute_type(p, substitution))
                    .collect();
                let new_return = Box::new(self.substitute_type(return_type, substitution));
                Type::Function { params: new_params, return_type: new_return }
            },
            _ => ty.clone(),
        }
    }
    
    fn get_generic_variables(&self, ty: &Type) -> Vec<TypeVar> {
        let mut vars = HashSet::new();
        self.collect_type_vars(ty, &mut vars);
        vars.into_iter().collect()
    }
    
    fn collect_type_vars(&self, ty: &Type, vars: &mut HashSet<TypeVar>) {
        match ty {
            Type::Var(var) => {
                vars.insert(var.clone());
            },
            Type::Function { params, return_type } => {
                for param in params {
                    self.collect_type_vars(param, vars);
                }
                self.collect_type_vars(return_type, vars);
            },
            _ => {},
        }
    }
    
    fn get_param_types(&self, func: &FunctionDecl) -> Vec<Type> {
        // TODO: Extract actual parameter types from function signature
        func.params.iter().map(|_| Type::Var(TypeVar::new(0, 0))).collect()
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}