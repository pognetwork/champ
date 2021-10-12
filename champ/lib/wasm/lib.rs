use anyhow::Result;
use wasmtime::{self, Caller, Engine, Func, Instance, Module, Store};

const HELLO_WORLD_WAT: &str = r#"
(module
    (import "host" "hello" (func $host_hello (param i32)))

    (func (export "hello")
        i32.const 3
        call $host_hello)
)
"#;

pub struct PogWasmRuntime {
    engine: Engine,
}

impl PogWasmRuntime {
    pub fn new() -> Self {
        let engine = Engine::default();
        PogWasmRuntime {
            engine,
        }
    }

    pub fn run(self) -> Result<()> {
        let module = Module::new(&self.engine, HELLO_WORLD_WAT)?;
        let mut store = Store::new(&self.engine, 4);
        let host_hello = Func::wrap(&mut store, |caller: Caller<'_, u32>, param: i32| {
            println!("Got {} from WebAssembly", param);
            println!("my host state is: {}", caller.data());
        });

        let instance = Instance::new(&mut store, &module, &[host_hello.into()])?;
        let hello = instance.get_typed_func::<(), (), _>(&mut store, "hello")?;

        // And finally we can call the wasm!
        hello.call(&mut store, ())?;

        Ok(())
    }
}
