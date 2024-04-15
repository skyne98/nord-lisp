use wasmtime::{Engine, Instance, Linker, Module, Store, WasmResults};
use eyre::{ContextCompat, Result, WrapErr};

pub struct Runtime {
    pub engine: Engine,
    pub module: Module,
    pub linker: Linker<()>,
    pub store: Store<()>,
    pub instance: Instance,
}

impl Runtime {
    pub fn new(bytes: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes).map_err(|err| eyre::eyre!("Failed to create module: {:#?}", err))?;
        let mut linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).map_err(|err| eyre::eyre!("Failed to instantiate module: {:#?}", err))?;

        Ok(Self {
            engine,
            module,
            linker,
            store,
            instance,
        })
    }

    pub fn run<T: WasmResults>(&mut self) -> Result<T> {
        let main = self.instance.get_func(&mut self.store, "main").wrap_err("Failed to get function")?;
        let answer = main.typed::<(), T>(&self.store).map_err(|err| eyre::eyre!("Failed to get typed function: {:#?}", err))?;
        answer.call(&mut self.store, ()).map_err(|err| eyre::eyre!("Failed to call function: {:#?}", err))
    }
}