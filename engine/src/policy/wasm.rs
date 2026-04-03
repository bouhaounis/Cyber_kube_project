use anyhow::{anyhow, bail, Context, Result};
use std::path::Path;
use wasmtime::{Engine, Linker, Memory, Module, Store, TypedFunc};

#[derive(Clone)]
pub struct WasmRuntime {
    engine: Engine,
}

impl WasmRuntime {
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: Engine::default(),
        })
    }

    pub fn evaluate(&self, module_path: &str, input: &[u8]) -> Result<i32> {
        let module = load_module(&self.engine, module_path)?;
        let linker = Linker::new(&self.engine);
        let wasi = wasmtime_wasi::WasiCtxBuilder::new().build();
        let mut store = Store::new(&self.engine, wasi);

        let instance = linker
            .instantiate(&mut store, &module)
            .with_context(|| format!("instantiating WASM module {module_path}"))?;

        if let Some(func) = instance.get_func(&mut store, "evaluate") {
            if let Ok(typed) = func.typed::<(), i32>(&store) {
                return typed
                    .call(&mut store, ())
                    .context("calling WASM evaluate() function");
            }

            if let Ok(typed) = func.typed::<(i32, i32), i32>(&store) {
                return self.call_with_input(&mut store, &instance, typed, input);
            }
        }

        bail!(
            "WASM module {module_path} must export evaluate() -> i32 or evaluate(i32, i32) -> i32"
        )
    }

    fn call_with_input(
        &self,
        store: &mut Store<wasmtime_wasi::WasiCtx>,
        instance: &wasmtime::Instance,
        evaluate: TypedFunc<(i32, i32), i32>,
        input: &[u8],
    ) -> Result<i32> {
        let memory = instance.get_memory(&mut *store, "memory").ok_or_else(|| {
            anyhow!("WASM module exporting evaluate(ptr, len) must also export memory")
        })?;
        let alloc = instance
            .get_typed_func::<i32, i32>(&mut *store, "alloc")
            .context(
                "WASM module exporting evaluate(ptr, len) must also export alloc(i32) -> i32",
            )?;

        let len = i32::try_from(input.len()).context("WASM input too large")?;
        let ptr = alloc
            .call(&mut *store, len)
            .context("calling WASM alloc()")?;
        write_input(memory, store, ptr, input)?;

        evaluate
            .call(store, (ptr, len))
            .context("calling WASM evaluate(ptr, len)")
    }
}

fn load_module(engine: &Engine, module_path: &str) -> Result<Module> {
    let path = Path::new(module_path);
    if path.extension().and_then(|value| value.to_str()) == Some("wat") {
        let bytes = wat::parse_file(path)
            .with_context(|| format!("parsing WAT module {}", path.display()))?;
        return Module::from_binary(engine, &bytes)
            .with_context(|| format!("loading WAT-compiled module {}", path.display()));
    }

    Module::from_file(engine, path)
        .with_context(|| format!("loading WASM module {}", path.display()))
}

fn write_input(
    memory: Memory,
    store: &mut Store<wasmtime_wasi::WasiCtx>,
    ptr: i32,
    input: &[u8],
) -> Result<()> {
    if ptr < 0 {
        bail!("WASM alloc returned negative pointer");
    }

    let start = ptr as usize;
    let end = start
        .checked_add(input.len())
        .ok_or_else(|| anyhow!("WASM memory write overflow"))?;
    let data = memory.data_mut(store);

    if end > data.len() {
        bail!(
            "WASM memory range out of bounds: need {} bytes, memory has {} bytes",
            end,
            data.len()
        );
    }

    data[start..end].copy_from_slice(input);
    Ok(())
}
