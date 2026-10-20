// IndexedDB - Client-side database
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum IndexedDBValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<IndexedDBValue>),
    Object(HashMap<String, IndexedDBValue>),
}

pub struct IndexedDB {
    databases: HashMap<String, Database>,
}

#[derive(Debug)]
pub struct Database {
    pub name: String,
    pub version: u32,
    pub object_stores: HashMap<String, ObjectStore>,
}

#[derive(Debug)]
pub struct ObjectStore {
    pub name: String,
    pub key_path: Option<String>,
    pub auto_increment: bool,
    pub records: HashMap<String, IndexedDBValue>,
    pub indexes: HashMap<String, Index>,
}

#[derive(Debug)]
pub struct Index {
    pub name: String,
    pub key_path: String,
    pub unique: bool,
    pub multi_entry: bool,
}

#[derive(Debug)]
pub struct Transaction {
    pub database: String,
    pub object_stores: Vec<String>,
    pub mode: TransactionMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionMode {
    Readonly,
    Readwrite,
    Versionchange,
}

impl IndexedDB {
    pub fn new() -> Self {
        IndexedDB {
            databases: HashMap::new(),
        }
    }

    pub fn open(&mut self, name: &str, version: u32) -> Result<&mut Database, String> {
        if !self.databases.contains_key(name) {
            let db = Database {
                name: name.to_string(),
                version,
                object_stores: HashMap::new(),
            };
            self.databases.insert(name.to_string(), db);
        }
        Ok(self.databases.get_mut(name).unwrap())
    }

    pub fn delete_database(&mut self, name: &str) -> bool {
        self.databases.remove(name).is_some()
    }

    pub fn databases(&self) -> Vec<String> {
        self.databases.keys().cloned().collect()
    }
}

impl Database {
    pub fn create_object_store(&mut self, name: &str, key_path: Option<String>, auto_increment: bool) -> Result<(), String> {
        if self.object_stores.contains_key(name) {
            return Err("Object store already exists".to_string());
        }

        let store = ObjectStore {
            name: name.to_string(),
            key_path,
            auto_increment,
            records: HashMap::new(),
            indexes: HashMap::new(),
        };

        self.object_stores.insert(name.to_string(), store);
        Ok(())
    }

    pub fn delete_object_store(&mut self, name: &str) -> bool {
        self.object_stores.remove(name).is_some()
    }
}

impl ObjectStore {
    pub fn put(&mut self, key: &str, value: IndexedDBValue) -> Result<(), String> {
        self.records.insert(key.to_string(), value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&IndexedDBValue> {
        self.records.get(key)
    }

    pub fn delete(&mut self, key: &str) -> bool {
        self.records.remove(key).is_some()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn create_index(&mut self, name: &str, key_path: &str, unique: bool, multi_entry: bool) -> Result<(), String> {
        if self.indexes.contains_key(name) {
            return Err("Index already exists".to_string());
        }

        let index = Index {
            name: name.to_string(),
            key_path: key_path.to_string(),
            unique,
            multi_entry,
        };

        self.indexes.insert(name.to_string(), index);
        Ok(())
    }

    pub fn delete_index(&mut self, name: &str) -> bool {
        self.indexes.remove(name).is_some()
    }
}
