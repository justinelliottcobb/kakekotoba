//! Code generation module (stubbed)
//!
//! The LLVM-based codegen has been removed pending a decision on the compilation
//! backend (custom bytecode VM vs WASM extension vs LLVM). See docs/ROADMAP.md
//! Phase 3 (Bytecode VM) and Phase 4 (Native Compilation) for the plan.

use crate::ast::Program;
use crate::error::Result;

pub struct CodeGenerator {
    _module_name: String,
}

impl CodeGenerator {
    pub fn new(module_name: &str) -> Result<Self> {
        Ok(Self {
            _module_name: module_name.to_string(),
        })
    }

    pub fn compile_program(&mut self, _program: &Program) -> Result<()> {
        todo!("Code generation backend not yet implemented. See docs/ROADMAP.md Phase 3-4.")
    }
}
