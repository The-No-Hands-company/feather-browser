// IndexedDB - Client-side database API
// NoSQL document database in the browser

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// IndexedDB factory
pub struct IDBFactory {
    databases: Arc<RwLock<HashMap<String, IDBDatabase>>>,
}

impl IDBFactory {
    pub fn new() -> Self {
        Self {
            databases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn open(&self, name: &str, version: u32) -> Result<IDBDatabase, IDBError> {
        let mut dbs = self.databases.write().await;
        
        if let Some(db) = dbs.get(name) {
            if db.version == version {
                return Ok(db.clone());
            }
        }

        let db = IDBDatabase::new(name, version);
        dbs.insert(name.to_string(), db.clone());
        Ok(db)
    }

    pub async fn delete_database(&self, name: &str) -> Result<(), IDBError> {
        let mut dbs = self.databases.write().await;
        dbs.remove(name);
        Ok(())
    }

    pub async fn databases(&self) -> Vec<IDBDatabaseInfo> {
        let dbs = self.databases.read().await;
        dbs.values().map(|db| IDBDatabaseInfo {
            name: db.name.clone(),
            version: db.version,
        }).collect()
    }
}

/// IndexedDB database
#[derive(Clone)]
pub struct IDBDatabase {
    pub name: String,
    pub version: u32,
    pub object_store_names: Arc<RwLock<Vec<String>>>,
    stores: Arc<RwLock<HashMap<String, IDBObjectStore>>>,
}

impl IDBDatabase {
    fn new(name: &str, version: u32) -> Self {
        Self {
            name: name.to_string(),
            version,
            object_store_names: Arc::new(RwLock::new(Vec::new())),
            stores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_object_store(&self, name: &str, options: IDBObjectStoreParameters) -> Result<IDBObjectStore, IDBError> {
        let mut stores = self.stores.write().await;
        let mut store_names = self.object_store_names.write().await;

        if stores.contains_key(name) {
            return Err(IDBError::ConstraintError("Object store already exists".to_string()));
        }

        let store = IDBObjectStore::new(name, options);
        stores.insert(name.to_string(), store.clone());
        store_names.push(name.to_string());

        Ok(store)
    }

    pub async fn delete_object_store(&self, name: &str) -> Result<(), IDBError> {
        let mut stores = self.stores.write().await;
        let mut store_names = self.object_store_names.write().await;

        stores.remove(name);
        store_names.retain(|n| n != name);

        Ok(())
    }

    pub async fn transaction(&self, store_names: &[&str], mode: IDBTransactionMode) -> IDBTransaction {
        IDBTransaction::new(self.clone(), store_names, mode)
    }

    pub async fn close(&self) {
        // Close database connection
    }
}

pub struct IDBDatabaseInfo {
    pub name: String,
    pub version: u32,
}

/// Object store (table)
#[derive(Clone)]
pub struct IDBObjectStore {
    pub name: String,
    pub key_path: Option<String>,
    pub auto_increment: bool,
    pub index_names: Arc<RwLock<Vec<String>>>,
    data: Arc<RwLock<HashMap<IDBKey, Vec<u8>>>>,
    indexes: Arc<RwLock<HashMap<String, IDBIndex>>>,
}

impl IDBObjectStore {
    fn new(name: &str, options: IDBObjectStoreParameters) -> Self {
        Self {
            name: name.to_string(),
            key_path: options.key_path,
            auto_increment: options.auto_increment,
            index_names: Arc::new(RwLock::new(Vec::new())),
            data: Arc::new(RwLock::new(HashMap::new())),
            indexes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add(&self, value: &[u8], key: Option<IDBKey>) -> Result<IDBKey, IDBError> {
        let mut data = self.data.write().await;
        
        let key = key.unwrap_or_else(|| {
            if self.auto_increment {
                IDBKey::Number((data.len() + 1) as i64)
            } else {
                IDBKey::Number(0)
            }
        });

        if data.contains_key(&key) {
            return Err(IDBError::ConstraintError("Key already exists".to_string()));
        }

        data.insert(key.clone(), value.to_vec());
        Ok(key)
    }

    pub async fn put(&self, value: &[u8], key: Option<IDBKey>) -> Result<IDBKey, IDBError> {
        let mut data = self.data.write().await;
        
        let key = key.unwrap_or_else(|| {
            if self.auto_increment {
                IDBKey::Number((data.len() + 1) as i64)
            } else {
                IDBKey::Number(0)
            }
        });

        data.insert(key.clone(), value.to_vec());
        Ok(key)
    }

    pub async fn get(&self, key: &IDBKey) -> Result<Option<Vec<u8>>, IDBError> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    pub async fn get_all(&self, query: Option<IDBKeyRange>, count: Option<u32>) -> Result<Vec<Vec<u8>>, IDBError> {
        let data = self.data.read().await;
        
        // Filter by query range if provided
        let mut results: Vec<Vec<u8>> = if let Some(range) = query {
            data.iter()
                .filter(|(key, _)| range.contains(key))
                .map(|(_, value)| value.clone())
                .collect()
        } else {
            data.values().cloned().collect()
        };
        
        // Apply count limit
        if let Some(limit) = count {
            results.truncate(limit as usize);
        }

        Ok(results)
    }

    pub async fn get_all_keys(&self, query: Option<IDBKeyRange>, count: Option<u32>) -> Result<Vec<IDBKey>, IDBError> {
        let data = self.data.read().await;
        
        // Filter by query range if provided
        let mut keys: Vec<IDBKey> = if let Some(range) = query {
            data.keys()
                .filter(|key| range.contains(key))
                .cloned()
                .collect()
        } else {
            data.keys().cloned().collect()
        };
        
        // Apply count limit
        if let Some(limit) = count {
            keys.truncate(limit as usize);
        }

        Ok(keys)
    }

    pub async fn delete(&self, key: &IDBKey) -> Result<(), IDBError> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), IDBError> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }

    pub async fn count(&self, query: Option<IDBKeyRange>) -> Result<u32, IDBError> {
        let data = self.data.read().await;
        
        // Count filtered by query range if provided
        let count = if let Some(range) = query {
            data.keys()
                .filter(|key| range.contains(key))
                .count()
        } else {
            data.len()
        };
        
        Ok(count as u32)
    }

    pub async fn create_index(&self, name: &str, key_path: &str, options: IDBIndexParameters) -> Result<IDBIndex, IDBError> {
        let mut indexes = self.indexes.write().await;
        let mut index_names = self.index_names.write().await;

        if indexes.contains_key(name) {
            return Err(IDBError::ConstraintError("Index already exists".to_string()));
        }

        let index = IDBIndex::new(name, key_path, options);
        indexes.insert(name.to_string(), index.clone());
        index_names.push(name.to_string());

        Ok(index)
    }

    pub async fn delete_index(&self, name: &str) -> Result<(), IDBError> {
        let mut indexes = self.indexes.write().await;
        let mut index_names = self.index_names.write().await;

        indexes.remove(name);
        index_names.retain(|n| n != name);

        Ok(())
    }

    pub async fn open_cursor(&self, range: Option<IDBKeyRange>, direction: IDBCursorDirection) -> Option<IDBCursor> {
        Some(IDBCursor::new(self.clone(), range, direction))
    }
}

pub struct IDBObjectStoreParameters {
    pub key_path: Option<String>,
    pub auto_increment: bool,
}

/// Index (secondary key)
#[derive(Clone)]
pub struct IDBIndex {
    pub name: String,
    pub key_path: String,
    pub unique: bool,
    pub multi_entry: bool,
    // Store reference to parent object store data
    data: Arc<RwLock<HashMap<IDBKey, Vec<u8>>>>,
}

impl IDBIndex {
    fn new(name: &str, key_path: &str, options: IDBIndexParameters) -> Self {
        Self {
            name: name.to_string(),
            key_path: key_path.to_string(),
            unique: options.unique,
            multi_entry: options.multi_entry,
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &IDBKey) -> Result<Option<Vec<u8>>, IDBError> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    pub async fn get_all(&self, query: Option<IDBKeyRange>, count: Option<u32>) -> Result<Vec<Vec<u8>>, IDBError> {
        let data = self.data.read().await;
        
        // Filter by query range if provided
        let mut results: Vec<Vec<u8>> = if let Some(range) = query {
            data.iter()
                .filter(|(key, _)| range.contains(key))
                .map(|(_, value)| value.clone())
                .collect()
        } else {
            data.values().cloned().collect()
        };
        
        // Apply count limit
        if let Some(limit) = count {
            results.truncate(limit as usize);
        }

        Ok(results)
    }

    pub async fn count(&self, query: Option<IDBKeyRange>) -> Result<u32, IDBError> {
        let data = self.data.read().await;
        
        // Count filtered by query range if provided
        let count = if let Some(range) = query {
            data.keys()
                .filter(|key| range.contains(key))
                .count()
        } else {
            data.len()
        };
        
        Ok(count as u32)
    }

    pub async fn open_cursor(&self, range: Option<IDBKeyRange>, direction: IDBCursorDirection) -> Option<IDBCursor> {
        // Create cursor with proper parameters
        let store = IDBObjectStore {
            name: self.name.clone(),
            key_path: Some(self.key_path.clone()),
            auto_increment: false,
            data: self.data.clone(),
            indexes: Arc::new(RwLock::new(HashMap::new())),
            index_names: Arc::new(RwLock::new(Vec::new())),
        };
        Some(IDBCursor::new(store, range, direction))
    }
}

pub struct IDBIndexParameters {
    pub unique: bool,
    pub multi_entry: bool,
}

/// Transaction
pub struct IDBTransaction {
    db: IDBDatabase,
    object_store_names: Vec<String>,
    mode: IDBTransactionMode,
    state: IDBTransactionState,
}

impl IDBTransaction {
    fn new(db: IDBDatabase, store_names: &[&str], mode: IDBTransactionMode) -> Self {
        Self {
            db,
            object_store_names: store_names.iter().map(|s| s.to_string()).collect(),
            mode,
            state: IDBTransactionState::Active,
        }
    }

    pub async fn object_store(&self, name: &str) -> Result<IDBObjectStore, IDBError> {
        let stores = self.db.stores.read().await;
        stores.get(name).cloned().ok_or(IDBError::NotFoundError("Object store not found".to_string()))
    }

    pub async fn commit(&mut self) -> Result<(), IDBError> {
        self.state = IDBTransactionState::Committing;
        // Commit transaction
        self.state = IDBTransactionState::Finished;
        Ok(())
    }

    pub async fn abort(&mut self) -> Result<(), IDBError> {
        self.state = IDBTransactionState::Finished;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IDBTransactionMode {
    ReadOnly,
    ReadWrite,
    VersionChange,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IDBTransactionState {
    Active,
    Inactive,
    Committing,
    Finished,
}

/// Cursor (iterator)
pub struct IDBCursor {
    source: IDBObjectStore,
    range: Option<IDBKeyRange>,
    direction: IDBCursorDirection,
    current_key: Option<IDBKey>,
    current_value: Option<Vec<u8>>,
    keys: Vec<IDBKey>, // Cached sorted keys for iteration
    current_index: usize,
}

impl IDBCursor {
    fn new(source: IDBObjectStore, range: Option<IDBKeyRange>, direction: IDBCursorDirection) -> Self {
        Self {
            source,
            range,
            direction,
            current_key: None,
            current_value: None,
            keys: Vec::new(),
            current_index: 0,
        }
    }

    /// Initialize cursor and load keys
    pub async fn init(&mut self) -> Result<bool, IDBError> {
        let data = self.source.data.read().await;
        
        // Get all keys and filter by range
        let mut keys: Vec<IDBKey> = if let Some(ref range) = self.range {
            data.keys()
                .filter(|key| range.contains(key))
                .cloned()
                .collect()
        } else {
            data.keys().cloned().collect()
        };

        // Sort keys based on direction
        match self.direction {
            IDBCursorDirection::Next | IDBCursorDirection::NextUnique => {
                keys.sort();
            }
            IDBCursorDirection::Prev | IDBCursorDirection::PrevUnique => {
                keys.sort();
                keys.reverse();
            }
        }

        self.keys = keys;
        
        // Load first item
        if !self.keys.is_empty() {
            let key = &self.keys[0];
            if let Some(value) = data.get(key) {
                self.current_key = Some(key.clone());
                self.current_value = Some(value.clone());
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn continue_(&mut self, key: Option<IDBKey>) {
        if let Some(target_key) = key {
            // Move to specific key
            if let Some(pos) = self.keys.iter().position(|k| k == &target_key) {
                self.current_index = pos;
                self.load_current().await;
            }
        } else {
            // Move to next item
            self.current_index += 1;
            self.load_current().await;
        }
    }

    pub async fn advance(&mut self, count: u32) {
        self.current_index += count as usize;
        self.load_current().await;
    }

    async fn load_current(&mut self) {
        if self.current_index < self.keys.len() {
            let key = &self.keys[self.current_index];
            let data = self.source.data.read().await;
            if let Some(value) = data.get(key) {
                self.current_key = Some(key.clone());
                self.current_value = Some(value.clone());
                return;
            }
        }
        
        // End of cursor
        self.current_key = None;
        self.current_value = None;
    }

    pub async fn update(&mut self, value: &[u8]) -> Result<IDBKey, IDBError> {
        if let Some(ref key) = self.current_key {
            let mut data = self.source.data.write().await;
            data.insert(key.clone(), value.to_vec());
            self.current_value = Some(value.to_vec());
            Ok(key.clone())
        } else {
            Err(IDBError::InvalidStateError("Cursor has no current key".to_string()))
        }
    }

    pub async fn delete(&mut self) -> Result<(), IDBError> {
        if let Some(ref key) = self.current_key {
            let mut data = self.source.data.write().await;
            data.remove(key);
            self.current_key = None;
            self.current_value = None;
            Ok(())
        } else {
            Err(IDBError::InvalidStateError("Cursor has no current key".to_string()))
        }
    }

    pub fn key(&self) -> Option<&IDBKey> {
        self.current_key.as_ref()
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.current_value.as_ref().map(|v| v.as_slice())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IDBCursorDirection {
    Next,
    NextUnique,
    Prev,
    PrevUnique,
}

/// Key
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IDBKey {
    Number(i64),  // Changed from f64 to i64 for Hash
    String(String),
    Date(i64),
    Binary(Vec<u8>),
    Array(Vec<Box<IDBKey>>),
}

/// Key range (query)
pub struct IDBKeyRange {
    pub lower: Option<IDBKey>,
    pub upper: Option<IDBKey>,
    pub lower_open: bool,
    pub upper_open: bool,
}

impl IDBKeyRange {
    pub fn only(value: IDBKey) -> Self {
        Self {
            lower: Some(value.clone()),
            upper: Some(value),
            lower_open: false,
            upper_open: false,
        }
    }

    pub fn lower_bound(lower: IDBKey, open: bool) -> Self {
        Self {
            lower: Some(lower),
            upper: None,
            lower_open: open,
            upper_open: false,
        }
    }

    pub fn upper_bound(upper: IDBKey, open: bool) -> Self {
        Self {
            lower: None,
            upper: Some(upper),
            lower_open: false,
            upper_open: open,
        }
    }

    pub fn bound(lower: IDBKey, upper: IDBKey, lower_open: bool, upper_open: bool) -> Self {
        Self {
            lower: Some(lower),
            upper: Some(upper),
            lower_open,
            upper_open,
        }
    }

    /// Check if a key is within this range
    pub fn contains(&self, key: &IDBKey) -> bool {
        // Check lower bound
        if let Some(ref lower) = self.lower {
            if self.lower_open {
                if key <= lower {
                    return false;
                }
            } else {
                if key < lower {
                    return false;
                }
            }
        }

        // Check upper bound
        if let Some(ref upper) = self.upper {
            if self.upper_open {
                if key >= upper {
                    return false;
                }
            } else {
                if key > upper {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug)]
pub enum IDBError {
    AbortError(String),
    ConstraintError(String),
    DataError(String),
    InvalidStateError(String),
    NotFoundError(String),
    QuotaExceededError(String),
    ReadOnlyError(String),
    TransactionInactiveError(String),
    VersionError(String),
}
