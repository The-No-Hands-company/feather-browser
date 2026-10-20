// JavaScript Bytecode Compiler and Virtual Machine
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Opcode {
    // Stack operations
    LoadConst(usize),
    LoadVar(String),
    StoreVar(String),
    Pop,
    Dup,
    
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    
    // Comparison
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    
    // Logical
    And,
    Or,
    Not,
    
    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Call(usize),
    Return,
    
    // Object operations
    GetProp(String),
    SetProp(String),
    NewObject,
    NewArray(usize),
    
    // Type operations
    TypeOf,
    InstanceOf,
}

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Function(Vec<Opcode>),
}

pub struct BytecodeCompiler {
    code: Vec<Opcode>,
    constants: Vec<Value>,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }
    
    pub fn compile_expression(&mut self, expr: &str) -> Result<(), String> {
        // Simple expression compiler
        if let Ok(num) = expr.parse::<f64>() {
            let idx = self.add_constant(Value::Number(num));
            self.emit(Opcode::LoadConst(idx));
        } else if expr.starts_with('"') || expr.starts_with('\'') {
            let s = expr.trim_matches(|c| c == '"' || c == '\'');
            let idx = self.add_constant(Value::String(s.to_string()));
            self.emit(Opcode::LoadConst(idx));
        } else {
            self.emit(Opcode::LoadVar(expr.to_string()));
        }
        Ok(())
    }
    
    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
    
    fn emit(&mut self, op: Opcode) {
        self.code.push(op);
    }
    
    pub fn bytecode(&self) -> &[Opcode] {
        &self.code
    }
    
    pub fn constants(&self) -> &[Value] {
        &self.constants
    }
}

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    ip: usize,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            ip: 0,
        }
    }
    
    pub fn execute(&mut self, code: &[Opcode], constants: &[Value]) -> Result<Value, String> {
        self.ip = 0;
        
        while self.ip < code.len() {
            match &code[self.ip] {
                Opcode::LoadConst(idx) => {
                    self.stack.push(constants[*idx].clone());
                }
                Opcode::LoadVar(name) => {
                    let val = self.globals.get(name)
                        .cloned()
                        .unwrap_or(Value::Undefined);
                    self.stack.push(val);
                }
                Opcode::StoreVar(name) => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    self.globals.insert(name.clone(), val);
                }
                Opcode::Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(self.add(a, b)?);
                }
                Opcode::Return => {
                    return self.stack.pop().ok_or("Stack underflow".to_string());
                }
                _ => {}
            }
            self.ip += 1;
        }
        
        Ok(Value::Undefined)
    }
    
    fn add(&self, a: Value, b: Value) -> Result<Value, String> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
            (Value::String(x), Value::String(y)) => Ok(Value::String(format!("{}{}", x, y))),
            _ => Err("Type error in addition".to_string()),
        }
    }
}
