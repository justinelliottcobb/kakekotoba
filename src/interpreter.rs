//! Tree-walking interpreter for kakekotoba
//!
//! Evaluates the AST directly. This is the minimal execution engine
//! for Phase 2.5 — a bridge until the bytecode VM (Phase 3).

use crate::ast::*;
use crate::error::{Error, Result, Span};
use std::collections::HashMap;
use std::fmt;

/// Runtime value
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
    /// User-defined function (closure)
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: Expression,
        env: Environment,
    },
    /// Built-in function
    Builtin(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(true) => write!(f, "真"),
            Value::Bool(false) => write!(f, "偽"),
            Value::Unit => write!(f, "()"),
            Value::Function { name, params, .. } => {
                let name = name.as_deref().unwrap_or("匿名");
                write!(f, "<関数 {} ({})>", name, params.join(" "))
            }
            Value::Builtin(name) => write!(f, "<組込 {}>", name),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            _ => false,
        }
    }
}

/// Lexically-scoped environment
#[derive(Debug, Clone)]
pub struct Environment {
    bindings: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create a child environment extending this one
    pub fn extend(&self) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// The interpreter
pub struct Interpreter {
    /// Global environment persists across evaluations (for REPL)
    pub env: Environment,
    /// Captured output (for testing; None means print to stdout)
    output: Option<Vec<String>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();

        // Register built-in functions
        for name in &[
            "+",
            "-",
            "*",
            "/",
            "%",
            "==",
            "!=",
            "<",
            "<=",
            ">",
            ">=",
            "print",
            "表示",
            "not",
            "否定",
            "toString",
            "文字列化",
        ] {
            env.define(name.to_string(), Value::Builtin(name.to_string()));
        }

        Self { env, output: None }
    }

    /// Enable output capture (for testing)
    pub fn capture_output(&mut self) {
        self.output = Some(Vec::new());
    }

    /// Get captured output
    pub fn get_output(&self) -> Option<&[String]> {
        self.output.as_deref()
    }

    /// Evaluate a program (sequence of declarations)
    pub fn eval_program(&mut self, program: &Program) -> Result<Value> {
        let mut last = Value::Unit;

        for decl in &program.declarations {
            last = self.eval_declaration(decl)?;
        }

        Ok(last)
    }

    /// Evaluate a single declaration, adding it to the global environment
    pub fn eval_declaration(&mut self, decl: &Declaration) -> Result<Value> {
        match decl {
            Declaration::Function(f) => {
                if f.name.name == "_main" {
                    // Top-level expression — evaluate immediately
                    let env = self.env.clone();
                    return self.eval_expression(&f.body, &env);
                }

                let param_names = self.extract_param_names(f);
                let value = Value::Function {
                    name: Some(f.name.name.clone()),
                    params: param_names,
                    body: f.body.clone(),
                    env: self.env.clone(),
                };
                self.env.define(f.name.name.clone(), value.clone());
                Ok(value)
            }
            Declaration::Type(_) => Ok(Value::Unit),
            Declaration::Import(_) => Ok(Value::Unit),
        }
    }

    /// Evaluate an expression in the given environment
    pub fn eval_expression(&mut self, expr: &Expression, env: &Environment) -> Result<Value> {
        match expr {
            Expression::Literal(lit) => self.eval_literal(lit),
            Expression::Identifier(id) => self.eval_identifier(id, env),
            Expression::Binary(bin) => self.eval_binary(bin, env),
            Expression::Unary(un) => self.eval_unary(un, env),
            Expression::If(if_expr) => self.eval_if(if_expr, env),
            Expression::Application(app) => self.eval_application(app, env),
            Expression::Lambda(lam) => self.eval_lambda(lam, env),
            Expression::Let(let_expr) => self.eval_let(let_expr, env),
            Expression::Match(match_expr) => self.eval_match(match_expr, env),
            Expression::Block(block) => self.eval_block(block, env),
        }
    }

    fn eval_literal(&self, lit: &Literal) -> Result<Value> {
        Ok(match lit {
            Literal::Integer(n) => Value::Integer(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Unit => Value::Unit,
        })
    }

    fn eval_identifier(&self, id: &Identifier, env: &Environment) -> Result<Value> {
        // Check local env first, then global
        if let Some(val) = env.get(&id.name) {
            return Ok(val.clone());
        }
        if let Some(val) = self.env.get(&id.name) {
            return Ok(val.clone());
        }
        Err(self.runtime_error(&id.span, &format!("Undefined variable: {}", id.name)))
    }

    fn eval_binary(&mut self, bin: &BinaryExpr, env: &Environment) -> Result<Value> {
        let left = self.eval_expression(&bin.left, env)?;
        let right = self.eval_expression(&bin.right, env)?;

        match bin.operator {
            BinaryOp::Add => self.numeric_op(&left, &right, |a, b| a + b, |a, b| a + b, &bin.span),
            BinaryOp::Sub => self.numeric_op(&left, &right, |a, b| a - b, |a, b| a - b, &bin.span),
            BinaryOp::Mul => self.numeric_op(&left, &right, |a, b| a * b, |a, b| a * b, &bin.span),
            BinaryOp::Div => {
                // Check for division by zero
                match (&left, &right) {
                    (_, Value::Integer(0)) => {
                        return Err(self.runtime_error(&bin.span, "Division by zero"))
                    }
                    (_, Value::Float(f)) if *f == 0.0 => {
                        return Err(self.runtime_error(&bin.span, "Division by zero"))
                    }
                    _ => {}
                }
                self.numeric_op(&left, &right, |a, b| a / b, |a, b| a / b, &bin.span)
            }
            BinaryOp::Mod => match (&left, &right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(self.runtime_error(&bin.span, "Modulo by zero"));
                    }
                    Ok(Value::Integer(a % b))
                }
                _ => Err(self.runtime_error(&bin.span, "Modulo requires integers")),
            },
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::Ne => Ok(Value::Bool(left != right)),
            BinaryOp::Lt => self.compare_op(&left, &right, |o| o.is_lt(), &bin.span),
            BinaryOp::Le => self.compare_op(&left, &right, |o| o.is_le(), &bin.span),
            BinaryOp::Gt => self.compare_op(&left, &right, |o| o.is_gt(), &bin.span),
            BinaryOp::Ge => self.compare_op(&left, &right, |o| o.is_ge(), &bin.span),
            BinaryOp::And => match (&left, &right) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                _ => Err(self.runtime_error(&bin.span, "AND requires booleans")),
            },
            BinaryOp::Or => match (&left, &right) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                _ => Err(self.runtime_error(&bin.span, "OR requires booleans")),
            },
            BinaryOp::Compose => {
                Err(self.runtime_error(&bin.span, "Function composition not yet implemented"))
            }
        }
    }

    fn numeric_op(
        &self,
        left: &Value,
        right: &Value,
        int_op: impl Fn(i64, i64) -> i64,
        float_op: impl Fn(f64, f64) -> f64,
        span: &Span,
    ) -> Result<Value> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(int_op(*a, *b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(*a, *b))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(float_op(*a as f64, *b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(float_op(*a, *b as f64))),
            // String concatenation with +
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(self.runtime_error(span, "Arithmetic requires numbers")),
        }
    }

    fn compare_op(
        &self,
        left: &Value,
        right: &Value,
        check: impl Fn(std::cmp::Ordering) -> bool,
        span: &Span,
    ) -> Result<Value> {
        let ord = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a
                .partial_cmp(b)
                .ok_or_else(|| self.runtime_error(span, "Cannot compare NaN"))?,
            (Value::Integer(a), Value::Float(b)) => (*a as f64)
                .partial_cmp(b)
                .ok_or_else(|| self.runtime_error(span, "Cannot compare NaN"))?,
            (Value::Float(a), Value::Integer(b)) => a
                .partial_cmp(&(*b as f64))
                .ok_or_else(|| self.runtime_error(span, "Cannot compare NaN"))?,
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => return Err(self.runtime_error(span, "Cannot compare these values")),
        };
        Ok(Value::Bool(check(ord)))
    }

    fn eval_unary(&mut self, un: &UnaryExpr, env: &Environment) -> Result<Value> {
        let operand = self.eval_expression(&un.operand, env)?;
        match un.operator {
            UnaryOp::Neg => match operand {
                Value::Integer(n) => Ok(Value::Integer(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(self.runtime_error(&un.span, "Negation requires a number")),
            },
            UnaryOp::Not => match operand {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(self.runtime_error(&un.span, "NOT requires a boolean")),
            },
        }
    }

    fn eval_if(&mut self, if_expr: &IfExpr, env: &Environment) -> Result<Value> {
        let condition = self.eval_expression(&if_expr.condition, env)?;
        match condition {
            Value::Bool(true) => self.eval_expression(&if_expr.then_branch, env),
            Value::Bool(false) => {
                if let Some(ref else_branch) = if_expr.else_branch {
                    self.eval_expression(else_branch, env)
                } else {
                    Ok(Value::Unit)
                }
            }
            _ => Err(self.runtime_error(&if_expr.span, "Condition must be a boolean")),
        }
    }

    fn eval_application(&mut self, app: &Application, env: &Environment) -> Result<Value> {
        let func = self.eval_expression(&app.function, env)?;
        let args: Vec<Value> = app
            .arguments
            .iter()
            .map(|a| self.eval_expression(a, env))
            .collect::<Result<_>>()?;

        match func {
            Value::Builtin(ref name) => self.call_builtin(name, &args, &app.span),
            Value::Function {
                params,
                body,
                env: closure_env,
                name,
                ..
            } => {
                if args.len() != params.len() {
                    return Err(self.runtime_error(
                        &app.span,
                        &format!(
                            "{} expects {} arguments, got {}",
                            name.as_deref().unwrap_or("function"),
                            params.len(),
                            args.len()
                        ),
                    ));
                }

                let mut call_env = closure_env.extend();
                for (param, arg) in params.iter().zip(args) {
                    call_env.define(param.clone(), arg);
                }

                // For recursive functions, bind the function name in the call env
                if let Some(ref fn_name) = name {
                    call_env.define(
                        fn_name.clone(),
                        Value::Function {
                            name: Some(fn_name.clone()),
                            params: params.clone(),
                            body: body.clone(),
                            env: closure_env.clone(),
                        },
                    );
                }

                self.eval_expression(&body, &call_env)
            }
            _ => Err(self.runtime_error(
                &app.span,
                &format!("Cannot call non-function value: {}", func),
            )),
        }
    }

    fn eval_lambda(&self, lam: &Lambda, env: &Environment) -> Result<Value> {
        let params: Vec<String> = lam.params.iter().map(|p| p.name.name.clone()).collect();
        Ok(Value::Function {
            name: None,
            params,
            body: *lam.body.clone(),
            env: env.clone(),
        })
    }

    fn eval_let(&mut self, let_expr: &LetExpr, env: &Environment) -> Result<Value> {
        let mut new_env = env.extend();

        for binding in &let_expr.bindings {
            let value = self.eval_expression(&binding.value, &new_env)?;
            self.bind_pattern(&binding.pattern, &value, &mut new_env)?;
        }

        self.eval_expression(&let_expr.body, &new_env)
    }

    fn eval_match(&mut self, match_expr: &MatchExpr, env: &Environment) -> Result<Value> {
        let scrutinee = self.eval_expression(&match_expr.scrutinee, env)?;

        for arm in &match_expr.arms {
            let mut arm_env = env.extend();
            if self.try_match(&arm.pattern, &scrutinee, &mut arm_env) {
                // Check guard if present
                if let Some(ref guard) = arm.guard {
                    let guard_val = self.eval_expression(guard, &arm_env)?;
                    if guard_val != Value::Bool(true) {
                        continue;
                    }
                }
                return self.eval_expression(&arm.body, &arm_env);
            }
        }

        Err(self.runtime_error(
            &match_expr.span,
            &format!("Non-exhaustive match: no arm matched {}", scrutinee),
        ))
    }

    fn eval_block(&mut self, block: &Block, env: &Environment) -> Result<Value> {
        let mut block_env = env.extend();

        for stmt in &block.statements {
            match stmt {
                Statement::Expression(expr) => {
                    self.eval_expression(expr, &block_env)?;
                }
                Statement::Let(binding) => {
                    let value = self.eval_expression(&binding.value, &block_env)?;
                    self.bind_pattern(&binding.pattern, &value, &mut block_env)?;
                }
            }
        }

        if let Some(ref expr) = block.expr {
            self.eval_expression(expr, &block_env)
        } else {
            Ok(Value::Unit)
        }
    }

    // ========================================================================
    // Pattern matching
    // ========================================================================

    fn try_match(&self, pattern: &Pattern, value: &Value, env: &mut Environment) -> bool {
        match pattern {
            Pattern::Wildcard => true,
            Pattern::Identifier(id) => {
                env.define(id.name.clone(), value.clone());
                true
            }
            Pattern::Literal(lit) => {
                let lit_val = match lit {
                    Literal::Integer(n) => Value::Integer(*n),
                    Literal::Float(f) => Value::Float(*f),
                    Literal::String(s) => Value::String(s.clone()),
                    Literal::Bool(b) => Value::Bool(*b),
                    Literal::Unit => Value::Unit,
                };
                lit_val == *value
            }
            Pattern::Or(a, b) => self.try_match(a, value, env) || self.try_match(b, value, env),
            Pattern::Tuple(_) | Pattern::Constructor { .. } => {
                // Not yet implemented — constructors need ADT runtime support
                false
            }
        }
    }

    fn bind_pattern(&self, pattern: &Pattern, value: &Value, env: &mut Environment) -> Result<()> {
        match pattern {
            Pattern::Identifier(id) => {
                env.define(id.name.clone(), value.clone());
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            _ => Err(self.runtime_error(
                &Span::new(0, 0, 1, 1),
                "Complex patterns in let bindings not yet supported",
            )),
        }
    }

    // ========================================================================
    // Built-in functions
    // ========================================================================

    fn call_builtin(&mut self, name: &str, args: &[Value], span: &Span) -> Result<Value> {
        match name {
            "+" => self.builtin_arithmetic(args, |a, b| a + b, |a, b| a + b, span),
            "-" => {
                if args.len() == 1 {
                    // Unary negation
                    match &args[0] {
                        Value::Integer(n) => Ok(Value::Integer(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(self.runtime_error(span, "Negation requires a number")),
                    }
                } else {
                    self.builtin_arithmetic(args, |a, b| a - b, |a, b| a - b, span)
                }
            }
            "*" => self.builtin_arithmetic(args, |a, b| a * b, |a, b| a * b, span),
            "/" => {
                if args.len() != 2 {
                    return Err(self.runtime_error(span, "/ requires 2 arguments"));
                }
                let is_zero = matches!(&args[1], Value::Integer(0))
                    || matches!(&args[1], Value::Float(f) if *f == 0.0);
                if is_zero {
                    Err(self.runtime_error(span, "Division by zero"))
                } else {
                    self.builtin_arithmetic(args, |a, b| a / b, |a, b| a / b, span)
                }
            }
            "%" => {
                if args.len() != 2 {
                    return Err(self.runtime_error(span, "% requires 2 arguments"));
                }
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) if *b != 0 => Ok(Value::Integer(a % b)),
                    _ => Err(self.runtime_error(span, "Modulo requires non-zero integers")),
                }
            }
            "==" => {
                if args.len() != 2 {
                    return Err(self.runtime_error(span, "== requires 2 arguments"));
                }
                Ok(Value::Bool(args[0] == args[1]))
            }
            "!=" => {
                if args.len() != 2 {
                    return Err(self.runtime_error(span, "!= requires 2 arguments"));
                }
                Ok(Value::Bool(args[0] != args[1]))
            }
            "<" | "<=" | ">" | ">=" => {
                if args.len() != 2 {
                    return Err(self.runtime_error(span, &format!("{} requires 2 arguments", name)));
                }
                let check: Box<dyn Fn(std::cmp::Ordering) -> bool> = match name {
                    "<" => Box::new(|o: std::cmp::Ordering| o.is_lt()),
                    "<=" => Box::new(|o: std::cmp::Ordering| o.is_le()),
                    ">" => Box::new(|o: std::cmp::Ordering| o.is_gt()),
                    ">=" => Box::new(|o: std::cmp::Ordering| o.is_ge()),
                    _ => unreachable!(),
                };
                self.compare_op(&args[0], &args[1], check, span)
            }
            "print" | "表示" => {
                if args.is_empty() {
                    return Err(self.runtime_error(span, "print requires at least 1 argument"));
                }
                let text: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
                let output = text.join(" ");
                if let Some(ref mut captured) = self.output {
                    captured.push(output);
                } else {
                    println!("{}", output);
                }
                Ok(Value::Unit)
            }
            "not" | "否定" => {
                if args.len() != 1 {
                    return Err(self.runtime_error(span, "not requires 1 argument"));
                }
                match &args[0] {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err(self.runtime_error(span, "not requires a boolean")),
                }
            }
            "toString" | "文字列化" => {
                if args.len() != 1 {
                    return Err(self.runtime_error(span, "toString requires 1 argument"));
                }
                Ok(Value::String(format!("{}", args[0])))
            }
            _ => Err(self.runtime_error(span, &format!("Unknown builtin: {}", name))),
        }
    }

    fn builtin_arithmetic(
        &self,
        args: &[Value],
        int_op: impl Fn(i64, i64) -> i64,
        float_op: impl Fn(f64, f64) -> f64,
        span: &Span,
    ) -> Result<Value> {
        if args.len() != 2 {
            return Err(self.runtime_error(span, "Arithmetic operations require 2 arguments"));
        }
        self.numeric_op(&args[0], &args[1], int_op, float_op, span)
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn extract_param_names(&self, f: &FunctionDecl) -> Vec<String> {
        // If the function has named params, use those
        if !f.params.is_empty() {
            return f.params.iter().map(|p| p.name.name.clone()).collect();
        }

        // For S-expression definitions like (定義 二倍 (数 -> 数) (* 2 x)),
        // the function body references variables that are the actual parameters.
        // We extract free variables from the body to determine param names.
        let free = self.free_variables(&f.body);
        let fn_name = &f.name.name;

        // Filter out built-ins and the function's own name (for recursion)
        let user_vars: Vec<String> = free
            .into_iter()
            .filter(|v| !self.is_builtin(v) && v != fn_name)
            .collect();

        // If we have a type signature, verify count matches
        if let Some(crate::types::Type::Function { ref params, .. }) = f.return_type {
            if user_vars.len() == params.len() {
                return user_vars;
            }
        }

        user_vars
    }

    fn free_variables(&self, expr: &Expression) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_free_vars(expr, &mut Vec::new(), &mut vars);
        vars
    }

    fn collect_free_vars(
        &self,
        expr: &Expression,
        bound: &mut Vec<String>,
        free: &mut Vec<String>,
    ) {
        match expr {
            Expression::Identifier(id) => {
                if !bound.contains(&id.name) && !free.contains(&id.name) {
                    free.push(id.name.clone());
                }
            }
            Expression::Literal(_) => {}
            Expression::Binary(bin) => {
                self.collect_free_vars(&bin.left, bound, free);
                self.collect_free_vars(&bin.right, bound, free);
            }
            Expression::Unary(un) => {
                self.collect_free_vars(&un.operand, bound, free);
            }
            Expression::Application(app) => {
                self.collect_free_vars(&app.function, bound, free);
                for arg in &app.arguments {
                    self.collect_free_vars(arg, bound, free);
                }
            }
            Expression::Lambda(lam) => {
                let new_bound: Vec<String> =
                    lam.params.iter().map(|p| p.name.name.clone()).collect();
                let mut extended = bound.clone();
                extended.extend(new_bound);
                self.collect_free_vars(&lam.body, &mut extended, free);
            }
            Expression::Let(let_expr) => {
                let mut extended = bound.clone();
                for binding in &let_expr.bindings {
                    self.collect_free_vars(&binding.value, &mut extended, free);
                    if let Pattern::Identifier(id) = &binding.pattern {
                        extended.push(id.name.clone());
                    }
                }
                self.collect_free_vars(&let_expr.body, &mut extended, free);
            }
            Expression::If(if_expr) => {
                self.collect_free_vars(&if_expr.condition, bound, free);
                self.collect_free_vars(&if_expr.then_branch, bound, free);
                if let Some(ref else_branch) = if_expr.else_branch {
                    self.collect_free_vars(else_branch, bound, free);
                }
            }
            Expression::Match(match_expr) => {
                self.collect_free_vars(&match_expr.scrutinee, bound, free);
                for arm in &match_expr.arms {
                    let mut arm_bound = bound.clone();
                    self.collect_pattern_bindings(&arm.pattern, &mut arm_bound);
                    self.collect_free_vars(&arm.body, &mut arm_bound, free);
                }
            }
            Expression::Block(block) => {
                let mut block_bound = bound.clone();
                for stmt in &block.statements {
                    match stmt {
                        Statement::Expression(expr) => {
                            self.collect_free_vars(expr, &mut block_bound, free);
                        }
                        Statement::Let(binding) => {
                            self.collect_free_vars(&binding.value, &mut block_bound, free);
                            if let Pattern::Identifier(id) = &binding.pattern {
                                block_bound.push(id.name.clone());
                            }
                        }
                    }
                }
                if let Some(ref expr) = block.expr {
                    self.collect_free_vars(expr, &mut block_bound, free);
                }
            }
        }
    }

    fn collect_pattern_bindings(&self, pattern: &Pattern, bound: &mut Vec<String>) {
        match pattern {
            Pattern::Identifier(id) => bound.push(id.name.clone()),
            Pattern::Constructor { patterns, .. } => {
                for p in patterns {
                    self.collect_pattern_bindings(p, bound);
                }
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings(p, bound);
                }
            }
            Pattern::Or(a, b) => {
                self.collect_pattern_bindings(a, bound);
                self.collect_pattern_bindings(b, bound);
            }
            Pattern::Wildcard | Pattern::Literal(_) => {}
        }
    }

    fn is_builtin(&self, name: &str) -> bool {
        matches!(
            name,
            "+" | "-"
                | "*"
                | "/"
                | "%"
                | "=="
                | "!="
                | "<"
                | "<="
                | ">"
                | ">="
                | "print"
                | "表示"
                | "not"
                | "否定"
                | "toString"
                | "文字列化"
        )
    }

    fn runtime_error(&self, span: &Span, msg: &str) -> Error {
        Error::Runtime {
            src: String::new(),
            span: span.clone().into(),
            message: msg.to_string(),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::sexp_parser::SExpParser;

    fn eval(input: &str) -> Value {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = SExpParser::new(tokens, input.to_string());
        let program = parser.parse_program().unwrap();
        let mut interpreter = Interpreter::new();
        interpreter.eval_program(&program).unwrap()
    }

    fn eval_with_output(input: &str) -> (Value, Vec<String>) {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = SExpParser::new(tokens, input.to_string());
        let program = parser.parse_program().unwrap();
        let mut interpreter = Interpreter::new();
        interpreter.capture_output();
        let result = interpreter.eval_program(&program).unwrap();
        let output = interpreter.get_output().unwrap().to_vec();
        (result, output)
    }

    #[test]
    fn test_integer_literal() {
        assert_eq!(eval("42"), Value::Integer(42));
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(eval("(+ 1 2)"), Value::Integer(3));
        assert_eq!(eval("(- 10 3)"), Value::Integer(7));
        assert_eq!(eval("(* 4 5)"), Value::Integer(20));
        assert_eq!(eval("(/ 10 2)"), Value::Integer(5));
    }

    #[test]
    fn test_nested_arithmetic() {
        assert_eq!(eval("(* 2 (+ 1 3))"), Value::Integer(8));
        assert_eq!(eval("(+ (* 2 3) (* 4 5))"), Value::Integer(26));
    }

    #[test]
    fn test_boolean_ops() {
        assert_eq!(eval("(== 1 1)"), Value::Bool(true));
        assert_eq!(eval("(== 1 2)"), Value::Bool(false));
        assert_eq!(eval("(< 1 2)"), Value::Bool(true));
        assert_eq!(eval("(> 1 2)"), Value::Bool(false));
    }

    #[test]
    fn test_if_expression() {
        assert_eq!(eval("(もし 真 42 0)"), Value::Integer(42));
        assert_eq!(eval("(もし 偽 42 0)"), Value::Integer(0));
        assert_eq!(eval("(もし (== 1 1) 42 0)"), Value::Integer(42));
    }

    #[test]
    fn test_function_definition_and_call() {
        assert_eq!(
            eval("(定義 二倍 (数 -> 数) (* 2 x)) (二倍 21)"),
            Value::Integer(42)
        );
    }

    #[test]
    fn test_function_without_type() {
        assert_eq!(eval("(定義 inc (+ 1 x)) (inc 41)"), Value::Integer(42));
    }

    #[test]
    fn test_lambda() {
        assert_eq!(eval("((匿名 (x) (* x x)) 5)"), Value::Integer(25));
    }

    #[test]
    fn test_let_binding() {
        assert_eq!(eval("(束縛 ((x 10) (y 20)) (+ x y))"), Value::Integer(30));
    }

    #[test]
    fn test_pattern_matching() {
        assert_eq!(eval("(場合 0 (0 -> 1) (n -> (* n 2)))"), Value::Integer(1));
        assert_eq!(eval("(場合 5 (0 -> 1) (n -> (* n 2)))"), Value::Integer(10));
    }

    #[test]
    fn test_recursive_function() {
        let input = "(定義 階乗 (数 -> 数) (場合 n (0 -> 1) (n -> (* n (階乗 (- n 1)))))) (階乗 5)";
        assert_eq!(eval(input), Value::Integer(120));
    }

    #[test]
    fn test_print() {
        let (_, output) = eval_with_output("(表示 42)");
        assert_eq!(output, vec!["42"]);
    }

    #[test]
    fn test_print_hello() {
        let (_, output) = eval_with_output(r#"(表示 "こんにちは世界")"#);
        assert_eq!(output, vec!["こんにちは世界"]);
    }

    #[test]
    fn test_string_concat() {
        let result = eval(r#"(+ "hello" " world")"#);
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_division_by_zero() {
        let mut lexer = Lexer::new("(/ 1 0)".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = SExpParser::new(tokens, "(/ 1 0)".to_string());
        let program = parser.parse_program().unwrap();
        let mut interpreter = Interpreter::new();
        assert!(interpreter.eval_program(&program).is_err());
    }

    #[test]
    fn test_japanese_brackets_eval() {
        assert_eq!(eval("「+ 1 2」"), Value::Integer(3));
        assert_eq!(eval("【* 3 4】"), Value::Integer(12));
    }
}
