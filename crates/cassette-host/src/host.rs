use std::fs;
use std::path::Path;

use wasmtime::{Engine, ExternType, Instance, Linker, Module, Store};

use crate::error::HostError;
use crate::signature::{ExportInfo, ExportKind, FuncSignature};

pub struct CassetteHost {
    engine: Engine,
    store: Store<()>,
    instance: Option<Instance>,
}

impl CassetteHost {
    pub fn new() -> Result<Self, HostError> {
        let engine = Engine::default();
        let store = Store::new(&engine, ());
        Ok(Self {
            engine,
            store,
            instance: None,
        })
    }

    pub fn load_plugin_from_bytes(&mut self, wasm_bytes: &[u8]) -> Result<(), HostError> {
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| HostError::InvalidWasm {
                message: e.to_string(),
            })?;
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut self.store, &module).map_err(|e| {
            HostError::InstantiationFailed {
                message: e.to_string(),
            }
        })?;
        self.instance = Some(instance);
        Ok(())
    }

    pub fn load_plugin_from_file(&mut self, path: &Path) -> Result<(), HostError> {
        let bytes = fs::read(path).map_err(|e| HostError::Io {
            source: e,
            path: path.to_path_buf(),
        })?;
        self.load_plugin_from_bytes(&bytes)
    }

    fn require_instance(&self) -> Result<Instance, HostError> {
        self.instance.ok_or(HostError::NoPluginLoaded)
    }

    pub fn list_exports(&mut self) -> Vec<ExportInfo> {
        let Some(instance) = self.instance else {
            return Vec::new();
        };
        let names: Vec<String> = instance
            .exports(&mut self.store)
            .map(|e| e.name().to_string())
            .collect();
        let mut result = Vec::with_capacity(names.len());
        for name in names {
            let Some(ext) = instance.get_export(&mut self.store, &name) else {
                continue;
            };
            let ty = ext.ty(&self.store);
            let kind = ExportKind::from_extern_type(&ty);
            let signature = match &ty {
                ExternType::Func(func_ty) => {
                    Some(FuncSignature::from_func_type(func_ty))
                }
                _ => None,
            };
            result.push(ExportInfo {
                name,
                kind,
                signature,
            });
        }
        result
    }

    pub fn export_names(&mut self) -> Vec<String> {
        self.list_exports().into_iter().map(|e| e.name).collect()
    }

    pub fn has_export(&mut self, name: &str) -> bool {
        let Some(instance) = self.instance else {
            return false;
        };
        instance.get_export(&mut self.store, name).is_some()
    }

    pub fn get_func_signature(&mut self, name: &str) -> Option<FuncSignature> {
        let instance = self.instance?;
        let ext = instance.get_export(&mut self.store, name)?;
        let ExternType::Func(func_ty) = ext.ty(&self.store) else {
            return None;
        };
        Some(FuncSignature::from_func_type(&func_ty))
    }

    pub fn call_void_to_i32(&mut self, name: &str) -> Result<i32, HostError> {
        let instance = self.require_instance()?;
        let func = instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| HostError::ExportNotFound {
                name: name.to_string(),
                available: self.export_names(),
            })?;
        let typed = match func.typed::<(), i32>(&mut self.store) {
            Ok(t) => t,
            Err(_) => {
                let actual = crate::signature::format_signature(&func.ty(&mut self.store));
                return Err(HostError::SignatureMismatch {
                    name: name.to_string(),
                    expected: "() -> (i32)".to_string(),
                    actual,
                });
            }
        };
        typed
            .call(&mut self.store, ())
            .map_err(|e| HostError::Trap {
                message: e.to_string(),
            })
    }

    pub fn call_i32_to_i32(&mut self, name: &str, a: i32) -> Result<i32, HostError> {
        let instance = self.require_instance()?;
        let func = instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| HostError::ExportNotFound {
                name: name.to_string(),
                available: self.export_names(),
            })?;
        let typed = match func.typed::<(i32,), i32>(&mut self.store) {
            Ok(t) => t,
            Err(_) => {
                let actual = crate::signature::format_signature(&func.ty(&mut self.store));
                return Err(HostError::SignatureMismatch {
                    name: name.to_string(),
                    expected: "(i32) -> (i32)".to_string(),
                    actual,
                });
            }
        };
        typed
            .call(&mut self.store, (a,))
            .map_err(|e| HostError::Trap {
                message: e.to_string(),
            })
    }

    pub fn call_i32_i32_to_i32(
        &mut self,
        name: &str,
        a: i32,
        b: i32,
    ) -> Result<i32, HostError> {
        let instance = self.require_instance()?;
        let func = instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| HostError::ExportNotFound {
                name: name.to_string(),
                available: self.export_names(),
            })?;
        let typed = match func.typed::<(i32, i32), i32>(&mut self.store) {
            Ok(t) => t,
            Err(_) => {
                let actual = crate::signature::format_signature(&func.ty(&mut self.store));
                return Err(HostError::SignatureMismatch {
                    name: name.to_string(),
                    expected: "(i32, i32) -> (i32)".to_string(),
                    actual,
                });
            }
        };
        typed
            .call(&mut self.store, (a, b))
            .map_err(|e| HostError::Trap {
                message: e.to_string(),
            })
    }

    pub fn call_i32_i32_i32_i32_to_i32(
        &mut self,
        name: &str,
        a: i32,
        b: i32,
        c: i32,
        d: i32,
    ) -> Result<i32, HostError> {
        let instance = self.require_instance()?;
        let func = instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| HostError::ExportNotFound {
                name: name.to_string(),
                available: self.export_names(),
            })?;
        let typed = match func.typed::<(i32, i32, i32, i32), i32>(&mut self.store) {
            Ok(t) => t,
            Err(_) => {
                let actual = crate::signature::format_signature(&func.ty(&mut self.store));
                return Err(HostError::SignatureMismatch {
                    name: name.to_string(),
                    expected: "(i32, i32, i32, i32) -> (i32)".to_string(),
                    actual,
                });
            }
        };
        typed
            .call(&mut self.store, (a, b, c, d))
            .map_err(|e| HostError::Trap {
                message: e.to_string(),
            })
    }

    pub fn write_memory(&mut self, offset: usize, data: &[u8]) -> Result<(), HostError> {
        let instance = self.require_instance()?;
        let memory = instance
            .get_memory(&mut self.store, "memory")
            .ok_or(HostError::MemoryNotFound)?;
        let mem_size = memory.data_size(&self.store);
        let end = offset.checked_add(data.len()).ok_or(HostError::MemoryOutOfBounds {
            offset,
            len: data.len(),
            memory_size: mem_size,
        })?;
        if end > mem_size {
            return Err(HostError::MemoryOutOfBounds {
                offset,
                len: data.len(),
                memory_size: mem_size,
            });
        }
        memory.data_mut(&mut self.store)[offset..end].copy_from_slice(data);
        Ok(())
    }

    pub fn read_memory(&mut self, offset: usize, len: usize) -> Result<Vec<u8>, HostError> {
        let instance = self.require_instance()?;
        let memory = instance
            .get_memory(&mut self.store, "memory")
            .ok_or(HostError::MemoryNotFound)?;
        let mem_size = memory.data_size(&self.store);
        let end = offset.checked_add(len).ok_or(HostError::MemoryOutOfBounds {
            offset,
            len,
            memory_size: mem_size,
        })?;
        if end > mem_size {
            return Err(HostError::MemoryOutOfBounds {
                offset,
                len,
                memory_size: mem_size,
            });
        }
        let data = memory.data(&self.store);
        Ok(data[offset..end].to_vec())
    }

    pub fn read_cstring(&mut self, offset: usize) -> Result<String, HostError> {
        let instance = self.require_instance()?;
        let memory = instance
            .get_memory(&mut self.store, "memory")
            .ok_or(HostError::MemoryNotFound)?;
        let data = memory.data(&self.store);
        if offset >= data.len() {
            return Err(HostError::MemoryOutOfBounds {
                offset,
                len: 1,
                memory_size: data.len(),
            });
        }
        let slice = &data[offset..];
        let null_pos = slice
            .iter()
            .position(|&b| b == 0)
            .ok_or(HostError::NoNullTerminator { offset })?;
        String::from_utf8(data[offset..offset + null_pos].to_vec()).map_err(|e| {
            HostError::Utf8Error {
                detail: e.to_string(),
            }
        })
    }

    pub fn heap_base(&mut self) -> Result<usize, HostError> {
        let instance = self.require_instance()?;
        let global = instance
            .get_global(&mut self.store, "__heap_base")
            .ok_or(HostError::HeapBaseNotFound)?;
        let val = global.get(&mut self.store);
        Ok(val.unwrap_i32() as usize)
    }

    pub fn memory_size(&mut self) -> Result<usize, HostError> {
        let instance = self.require_instance()?;
        let memory = instance
            .get_memory(&mut self.store, "memory")
            .ok_or(HostError::MemoryNotFound)?;
        Ok(memory.data_size(&self.store))
    }

    pub fn call_add(&mut self, a: i32, b: i32) -> Result<i32, HostError> {
        self.call_i32_i32_to_i32("add", a, b)
    }

    pub fn call_greet(&mut self) -> Result<String, HostError> {
        let pointer = self.call_void_to_i32("greet")?;
        self.read_cstring(pointer as usize)
    }

    pub fn call_greet_name(&mut self, name: &str) -> Result<String, HostError> {
        let heap_base = self.heap_base()?;
        let input_offset = heap_base;
        let output_offset = heap_base + 256;
        let out_capacity = 256usize;

        self.write_memory(input_offset, name.as_bytes())?;
        self.write_memory(output_offset, &[0u8; 256])?;

        let written = self.call_i32_i32_i32_i32_to_i32(
            "greet_name",
            input_offset as i32,
            name.len() as i32,
            output_offset as i32,
            out_capacity as i32,
        )?;

        let result_bytes = self.read_memory(output_offset, written as usize)?;
        String::from_utf8(result_bytes).map_err(|e| HostError::Utf8Error {
            detail: e.to_string(),
        })
    }
}

impl Default for CassetteHost {
    fn default() -> Self {
        Self::new().expect("failed to create CassetteHost")
    }
}