// Garbage Collector - Mark and Sweep algorithm
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GarbageCollector {
    heap: Vec<HeapObject>,
    free_list: Vec<usize>,
    marked: HashSet<usize>,
    allocation_threshold: usize,
    allocated: usize,
}

#[derive(Debug, Clone)]
pub struct HeapObject {
    pub data: ObjectData,
    pub marked: bool,
    pub references: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum ObjectData {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Function(FunctionData),
    Closure(ClosureData),
}

#[derive(Debug, Clone)]
pub struct FunctionData {
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ClosureData {
    pub function: FunctionData,
    pub captured: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Reference(usize),
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            free_list: Vec::new(),
            marked: HashSet::new(),
            allocation_threshold: 1000,
            allocated: 0,
        }
    }

    pub fn allocate(&mut self, data: ObjectData) -> usize {
        self.allocated += 1;
        
        if self.allocated >= self.allocation_threshold {
            self.collect(&[]);
        }

        if let Some(index) = self.free_list.pop() {
            self.heap[index] = HeapObject {
                data,
                marked: false,
                references: Vec::new(),
            };
            index
        } else {
            self.heap.push(HeapObject {
                data,
                marked: false,
                references: Vec::new(),
            });
            self.heap.len() - 1
        }
    }

    pub fn collect(&mut self, roots: &[usize]) {
        // Mark phase
        self.marked.clear();
        for &root in roots {
            self.mark(root);
        }

        // Sweep phase
        for (i, obj) in self.heap.iter_mut().enumerate() {
            if !self.marked.contains(&i) {
                // Deallocate
                obj.data = ObjectData::Object(HashMap::new());
                obj.references.clear();
                self.free_list.push(i);
                self.allocated = self.allocated.saturating_sub(1);
            } else {
                obj.marked = false;
            }
        }
    }

    fn mark(&mut self, index: usize) {
        if self.marked.contains(&index) {
            return;
        }
        
        if index >= self.heap.len() {
            return;
        }

        self.marked.insert(index);
        self.heap[index].marked = true;

        // Mark referenced objects
        let refs = self.heap[index].references.clone();
        for &ref_idx in &refs {
            self.mark(ref_idx);
        }
    }

    pub fn get(&self, index: usize) -> Option<&HeapObject> {
        self.heap.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut HeapObject> {
        self.heap.get_mut(index)
    }

    pub fn stats(&self) -> GCStats {
        GCStats {
            total_objects: self.heap.len(),
            allocated: self.allocated,
            free: self.free_list.len(),
            marked: self.marked.len(),
        }
    }
}

pub struct GCStats {
    pub total_objects: usize,
    pub allocated: usize,
    pub free: usize,
    pub marked: usize,
}

// Reference counting for immediate reclamation
pub struct RefCounted<T> {
    value: T,
    ref_count: usize,
}

impl<T> RefCounted<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            ref_count: 1,
        }
    }

    pub fn increment(&mut self) {
        self.ref_count += 1;
    }

    pub fn decrement(&mut self) -> bool {
        self.ref_count = self.ref_count.saturating_sub(1);
        self.ref_count == 0
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}
