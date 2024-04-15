use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use walrus::{FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType};
use eyre::{ContextCompat, OptionExt, Result};

pub type MirSharedContext = Rc<RefCell<MirContext>>;
pub struct MirContext {
    pub module: Module,
    pub builder: Option<FunctionBuilder>,
    pub locals_hash: HashMap<u32, LocalId>
}
impl Debug for MirContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MirContext")
            .field("builder", &self.builder)
            .finish()
    }
}
impl MirContext {
    pub fn new(module: Module) -> MirSharedContext {
        Rc::new(RefCell::new(MirContext {
            module,
            builder: None,
            locals_hash: HashMap::new()
        }))
    }

    // Local variables
    pub fn add_local(&mut self, index: u32, val_type: ValType) -> LocalId {
        let locals = &mut self.module.locals;
        let id = locals.add(val_type);
        self.locals_hash.insert(index, id);
        id
    }
    pub fn get_local(&mut self, index: u32) -> Option<LocalId> {
        self.locals_hash.get(&index).copied()
    }
    pub fn get_or_add_local(&mut self, index: u32, val_type: ValType) -> LocalId {
        if self.locals_hash.get(&index).is_none() {
            self.add_local(index, val_type)
        } else {
            self.get_local(index).expect("Local not found")
        }
    }

    // Function builder
    pub fn set_builder(&mut self, builder: FunctionBuilder) {
        self.builder = Some(builder);
    }
    pub fn new_builder(&mut self, params: &[ValType], results: &[ValType]) -> FunctionBuilder {
        FunctionBuilder::new(&mut self.module.types, params, results)
    }
    pub fn get_builder(&mut self) -> Option<&mut FunctionBuilder> {
        self.builder.as_mut()
    }
    pub fn set_new_builder(&mut self, params: &[ValType], results: &[ValType]) {
        self.builder = Some(self.new_builder(params, results));
    }
    pub fn finish_builder(&mut self, arguments: Vec<LocalId>) -> Result<FunctionId> {
        let builder = self.builder.take().wrap_err("Builder not set")?;
        let id = builder.finish(arguments, &mut self.module.funcs);
        Ok(id)
    }

    // Sequence builder
    pub fn function_body<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(InstrSeqBuilder) -> Result<()>
    {
        let builder = self.get_builder().ok_or_eyre("Builder not set")?;
        let seq = builder.func_body();
        f(seq)?;
        Ok(())
    }

    // Export
    pub fn export_function(&mut self, name: &str, function: FunctionId) {
        self.module.exports.add(name, function);
    }

    // Emit
    pub fn emit_wasm(&mut self) -> Vec<u8> {
        self.module.emit_wasm()
    }
}