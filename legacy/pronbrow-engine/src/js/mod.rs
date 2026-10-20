// JavaScript Engine Module
pub mod engine;
pub mod runtime;
pub mod wasm;
pub mod promise;
pub mod gc;
pub mod jit;
pub mod wasm_runtime;
pub mod workers;
pub mod parser;
pub mod scope;
pub mod es6;
pub mod proxy;
pub mod regexp;
pub mod bytecode;
pub mod dom_bridge;
pub mod interpreter;
pub mod dom_executor;
pub mod console_api;
pub mod fetch_api;
pub mod url_api;
pub mod timer_api;

pub use engine::{JSEngine, JSValue};
pub use promise::{Promise, AsyncFunction, Generator};
pub use gc::GarbageCollector;
pub use jit::JITCompiler;
pub use wasm_runtime::WasmRuntime;
pub use workers::{WorkerPool, SharedArrayBuffer, Atomics};
pub use parser::{JSParser, ASTNode};
pub use scope::{Scope, ScopeManager, Closure};
pub use es6::{Symbol, Generator as ES6Generator, ArrayIterator};
pub use proxy::{ProxyObject, Reflect};
pub use regexp::RegExp;
pub use dom_bridge::{DOMBridge, DocumentAPI, ElementAPI, WindowAPI};

// Legacy compatibility
pub use engine::JSEngine as JsEngine;
