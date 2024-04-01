use wasmtime::{Engine, Instance, Module, Store, WasmResults};
use anyhow::{Context, Result};

pub struct Runtime {
    pub engine: Engine,
    pub module: Module,
    pub store: Store<()>,
    pub instance: Instance,
}

impl Runtime {
    pub fn new(bytes: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes)?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;

        Ok(Self {
            engine,
            module,
            store,
            instance,
        })
    }

    pub fn run<T: WasmResults>(&mut self) -> Result<T> {
        let main = self.instance.get_func(&mut self.store, "main").context("No main function")?;
        let answer = main.typed::<(), T>(&self.store)?;
        answer.call(&mut self.store, ())
    }
}