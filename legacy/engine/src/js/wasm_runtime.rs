// WebAssembly Runtime - WASM execution environment
use std::collections::HashMap;

#[derive(Debug)]
pub struct WasmRuntime {
    modules: HashMap<String, WasmModule>,
    memory: WasmMemory,
    table: WasmTable,
    globals: HashMap<String, WasmValue>,
}

#[derive(Debug, Clone)]
pub struct WasmModule {
    pub name: String,
    pub functions: Vec<WasmFunction>,
    pub exports: HashMap<String, Export>,
    pub imports: Vec<Import>,
    pub memory_pages: usize,
}

#[derive(Debug, Clone)]
pub struct WasmFunction {
    pub name: String,
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
    pub locals: Vec<ValueType>,
    pub body: Vec<WasmInstruction>,
}

#[derive(Debug, Clone)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
    V128,      // SIMD
    FuncRef,   // Reference types
    ExternRef,
}

#[derive(Debug, Clone)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    V128([u8; 16]),
    FuncRef(usize),
    ExternRef(usize),
    Null,
}

#[derive(Debug, Clone)]
pub enum WasmInstruction {
    // Control flow
    Unreachable,
    Nop,
    Block(Vec<WasmInstruction>),
    Loop(Vec<WasmInstruction>),
    If(Vec<WasmInstruction>, Vec<WasmInstruction>),
    Br(u32),
    BrIf(u32),
    BrTable(Vec<u32>, u32),
    Return,
    Call(u32),
    CallIndirect(u32),

    // Parametric
    Drop,
    Select,

    // Variable access
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    // Memory
    I32Load(u32, u32),
    I64Load(u32, u32),
    F32Load(u32, u32),
    F64Load(u32, u32),
    I32Store(u32, u32),
    I64Store(u32, u32),
    F32Store(u32, u32),
    F64Store(u32, u32),
    MemorySize,
    MemoryGrow,

    // Constants
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    // Numeric operations
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,

    // F32 operations
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Abs,
    F32Neg,
    F32Sqrt,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,

    // SIMD operations
    V128Load(u32, u32),
    V128Store(u32, u32),
    V128Const([u8; 16]),
    I8x16Add,
    I16x8Mul,
    F32x4Add,
    F64x2Sub,
}

#[derive(Debug)]
pub struct WasmMemory {
    pages: Vec<[u8; 65536]>, // 64KB pages
    max_pages: Option<usize>,
}

#[derive(Debug)]
pub struct WasmTable {
    elements: Vec<Option<usize>>, // Function references
    max_size: Option<usize>,
}

impl WasmTable {
    pub fn get_element(&self, idx: usize) -> Option<usize> {
        self.elements.get(idx).and_then(|e| *e)
    }

    pub fn set_element(&mut self, idx: usize, func_ref: Option<usize>) -> Result<(), String> {
        if idx >= self.elements.len() {
            if let Some(max) = self.max_size {
                if idx >= max {
                    return Err("Table index out of bounds".to_string());
                }
            }
            self.elements.resize(idx + 1, None);
        }
        self.elements[idx] = func_ref;
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.elements.len()
    }

    pub fn grow(&mut self, delta: usize) -> Result<usize, String> {
        let old_size = self.elements.len();
        let new_size = old_size + delta;
        if let Some(max) = self.max_size {
            if new_size > max {
                return Err("Table grow exceeds maximum".to_string());
            }
        }
        self.elements.resize(new_size, None);
        Ok(old_size)
    }
}

#[derive(Debug, Clone)]
pub enum Export {
    Function(usize),
    Memory,
    Table,
    Global(usize),
}

#[derive(Debug, Clone)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub kind: ImportKind,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    Function(Vec<ValueType>, Vec<ValueType>),
    Memory(usize, Option<usize>),
    Table(usize, Option<usize>),
    Global(ValueType, bool),
}

impl WasmRuntime {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            memory: WasmMemory::new(1, Some(256)),
            table: WasmTable::new(0, None),
            globals: HashMap::new(),
        }
    }

    pub fn instantiate(&mut self, module: WasmModule) -> Result<String, String> {
        let name = module.name.clone();
        
        // Allocate memory
        if module.memory_pages > 0 {
            self.memory.grow(module.memory_pages)?;
        }

        self.modules.insert(name.clone(), module);
        Ok(name)
    }

    pub fn call_export(&mut self, module_name: &str, export_name: &str, args: Vec<WasmValue>) 
        -> Result<Vec<WasmValue>, String> {
        let func_idx = {
            let module = self.modules.get(module_name)
                .ok_or("Module not found")?;
            
            match module.exports.get(export_name) {
                Some(Export::Function(idx)) => *idx,
                _ => return Err("Export is not a function".to_string()),
            }
        };

        let func = {
            let module = self.modules.get(module_name)
                .ok_or("Module not found")?;
            module.functions[func_idx].clone()
        };

        self.execute_function(&func, args)
    }

    fn execute_function(&mut self, func: &WasmFunction, args: Vec<WasmValue>) 
        -> Result<Vec<WasmValue>, String> {
        let mut stack: Vec<WasmValue> = Vec::new();
        let mut locals: Vec<WasmValue> = args;
        
        // Initialize locals
        for local_type in &func.locals {
            locals.push(match local_type {
                ValueType::I32 => WasmValue::I32(0),
                ValueType::I64 => WasmValue::I64(0),
                ValueType::F32 => WasmValue::F32(0.0),
                ValueType::F64 => WasmValue::F64(0.0),
                _ => WasmValue::Null,
            });
        }

        // Execute instructions (simplified)
        for instruction in &func.body {
            match instruction {
                WasmInstruction::I32Const(val) => {
                    stack.push(WasmValue::I32(*val));
                }
                WasmInstruction::I32Add => {
                    if let (Some(WasmValue::I32(b)), Some(WasmValue::I32(a))) = 
                        (stack.pop(), stack.pop()) {
                        stack.push(WasmValue::I32(a.wrapping_add(b)));
                    }
                }
                WasmInstruction::LocalGet(idx) => {
                    if let Some(val) = locals.get(*idx as usize) {
                        stack.push(val.clone());
                    }
                }
                WasmInstruction::Return => {
                    break;
                }
                _ => {} // Simplified
            }
        }

        Ok(stack)
    }

    pub fn memory(&self) -> &WasmMemory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut WasmMemory {
        &mut self.memory
    }
}

impl WasmMemory {
    pub fn new(initial_pages: usize, max_pages: Option<usize>) -> Self {
        let mut pages = Vec::new();
        for _ in 0..initial_pages {
            pages.push([0u8; 65536]);
        }
        Self { pages, max_pages }
    }

    pub fn grow(&mut self, delta: usize) -> Result<usize, String> {
        let old_size = self.pages.len();
        let new_size = old_size + delta;

        if let Some(max) = self.max_pages {
            if new_size > max {
                return Err("Memory growth exceeds maximum".to_string());
            }
        }

        for _ in 0..delta {
            self.pages.push([0u8; 65536]);
        }

        Ok(old_size)
    }

    pub fn read(&self, addr: usize, len: usize) -> Option<Vec<u8>> {
        let mut result = Vec::new();
        for i in 0..len {
            let page = (addr + i) / 65536;
            let offset = (addr + i) % 65536;
            result.push(*self.pages.get(page)?.get(offset)?);
        }
        Some(result)
    }

    pub fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), String> {
        for (i, &byte) in data.iter().enumerate() {
            let page = (addr + i) / 65536;
            let offset = (addr + i) % 65536;
            if let Some(p) = self.pages.get_mut(page) {
                p[offset] = byte;
            } else {
                return Err("Memory access out of bounds".to_string());
            }
        }
        Ok(())
    }
}

impl WasmTable {
    pub fn new(initial_size: usize, max_size: Option<usize>) -> Self {
        Self {
            elements: vec![None; initial_size],
            max_size,
        }
    }

    pub fn get(&self, index: usize) -> Option<usize> {
        self.elements.get(index).and_then(|e| *e)
    }

    pub fn set(&mut self, index: usize, func_ref: Option<usize>) -> Result<(), String> {
        if index < self.elements.len() {
            self.elements[index] = func_ref;
            Ok(())
        } else {
            Err("Table index out of bounds".to_string())
        }
    }
}
