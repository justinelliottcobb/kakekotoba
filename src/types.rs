use crate::error::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    // Primitive types
    Int,
    Float,
    String,
    Bool,
    Unit,

    // Type variables (for polymorphism)
    Var(TypeVar),

    // Constructed types
    Constructor {
        name: String,
        args: Vec<Type>,
    },

    // Function types
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    // Tuple types
    Tuple(Vec<Type>),

    // List types
    List(Box<Type>),

    // Higher-kinded types
    Application {
        constructor: Box<Type>,
        args: Vec<Type>,
    },

    // Group homomorphism types (for meta-programming)
    Homomorphism {
        source_group: Box<Group>,
        target_group: Box<Group>,
        properties: HomomorphismProperties,
    },

    // Kind system for higher-kinded types
    Kind(Kind),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeVar {
    pub id: usize,
    pub name: Option<String>,
    pub level: usize,
}

impl TypeVar {
    pub fn new(id: usize, level: usize) -> Self {
        Self {
            id,
            name: None,
            level,
        }
    }

    pub fn with_name(id: usize, name: String, level: usize) -> Self {
        Self {
            id,
            name: Some(name),
            level,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    Type,                        // *
    Arrow(Box<Kind>, Box<Kind>), // k1 -> k2
    Constraint,                  // For type classes/constraints
}

// Group theory structures for meta-programming
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Group {
    pub elements: Type,
    pub operation: Type,
    pub identity: Type,
    pub inverse: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HomomorphismProperties {
    pub preserves_operation: bool,
    pub preserves_identity: bool,
    pub is_isomorphism: bool,
}

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    bindings: HashMap<String, TypeScheme>,
    level: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeScheme {
    pub forall: Vec<TypeVar>,
    pub ty: Type,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            level: 0,
        }
    }

    pub fn bind(&mut self, name: String, scheme: TypeScheme) {
        self.bindings.insert(name, scheme);
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings.get(name)
    }

    pub fn enter_level(&mut self) {
        self.level += 1;
    }

    pub fn exit_level(&mut self) {
        self.level = self.level.saturating_sub(1);
    }

    pub fn current_level(&self) -> usize {
        self.level
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub expected: Type,
    pub actual: Type,
    pub span: Span,
}

impl Constraint {
    pub fn new(expected: Type, actual: Type, span: Span) -> Self {
        Self {
            expected,
            actual,
            span,
        }
    }
}

// Type checker context
#[derive(Debug, Clone)]
pub struct TypeContext {
    pub env: TypeEnvironment,
    pub constraints: Vec<Constraint>,
    pub next_type_var: usize,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::new(),
            constraints: Vec::new(),
            next_type_var: 0,
        }
    }

    pub fn fresh_type_var(&mut self) -> TypeVar {
        let id = self.next_type_var;
        self.next_type_var += 1;
        TypeVar::new(id, self.env.current_level())
    }

    pub fn fresh_type_var_with_name(&mut self, name: String) -> TypeVar {
        let id = self.next_type_var;
        self.next_type_var += 1;
        TypeVar::with_name(id, name, self.env.current_level())
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions for type construction
impl Type {
    pub fn function(params: Vec<Type>, return_type: Type) -> Self {
        Type::Function {
            params,
            return_type: Box::new(return_type),
        }
    }

    pub fn list(element_type: Type) -> Self {
        Type::List(Box::new(element_type))
    }

    pub fn tuple(types: Vec<Type>) -> Self {
        Type::Tuple(types)
    }

    pub fn var(var: TypeVar) -> Self {
        Type::Var(var)
    }
}
