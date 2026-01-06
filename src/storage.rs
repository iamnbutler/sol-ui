//! Local data persistence system
//!
//! This module provides persistence for app data, preferences, and state.
//!
//! ## Storage Locations
//!
//! On macOS, data is stored in standard locations:
//! - App data: `~/Library/Application Support/<app_name>/`
//! - Preferences: Same location, `preferences.json`
//!
//! ## Usage
//!
//! ```ignore
//! use sol_ui::storage::{Storage, StorageConfig};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct AppState {
//!     counter: i32,
//!     items: Vec<String>,
//! }
//!
//! // Create storage for your app
//! let storage = Storage::new(StorageConfig {
//!     app_name: "MyApp".to_string(),
//!     ..Default::default()
//! });
//!
//! // Save state
//! let state = AppState { counter: 42, items: vec!["a".into()] };
//! storage.save("state", &state)?;
//!
//! // Load state
//! let loaded: Option<AppState> = storage.load("state")?;
//! ```
//!
//! ## Auto-save
//!
//! For automatic saving on changes, use `AutoSaver`:
//!
//! ```ignore
//! let auto_saver = AutoSaver::new(storage, Duration::from_secs(1));
//! auto_saver.mark_dirty(); // Triggers save after debounce delay
//! ```

use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Configuration for storage
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Application name (used for directory naming)
    pub app_name: String,
    /// Organization name (optional, for directory hierarchy)
    pub org_name: Option<String>,
    /// Whether to create directories if they don't exist
    pub create_dirs: bool,
    /// Whether to pretty-print JSON
    pub pretty_json: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            app_name: "SolApp".to_string(),
            org_name: None,
            create_dirs: true,
            pretty_json: true,
        }
    }
}

/// Errors that can occur during storage operations
#[derive(Debug)]
pub enum StorageError {
    /// Failed to create storage directory
    DirectoryCreation(std::io::Error),
    /// Failed to read file
    Read(std::io::Error),
    /// Failed to write file
    Write(std::io::Error),
    /// Failed to serialize data
    Serialize(serde_json::Error),
    /// Failed to deserialize data
    Deserialize(serde_json::Error),
    /// Storage path not available
    PathNotAvailable,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DirectoryCreation(e) => write!(f, "Failed to create directory: {}", e),
            StorageError::Read(e) => write!(f, "Failed to read file: {}", e),
            StorageError::Write(e) => write!(f, "Failed to write file: {}", e),
            StorageError::Serialize(e) => write!(f, "Failed to serialize: {}", e),
            StorageError::Deserialize(e) => write!(f, "Failed to deserialize: {}", e),
            StorageError::PathNotAvailable => write!(f, "Storage path not available"),
        }
    }
}

impl std::error::Error for StorageError {}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Local data storage manager
///
/// Handles saving and loading app data to/from the filesystem.
pub struct Storage {
    config: StorageConfig,
    base_path: Option<PathBuf>,
}

impl Storage {
    /// Create a new storage manager with the given configuration
    pub fn new(config: StorageConfig) -> Self {
        let base_path = Self::get_app_support_dir(&config);

        let mut storage = Self { config, base_path };

        // Create directories if needed
        if storage.config.create_dirs {
            if let Err(e) = storage.ensure_directories() {
                eprintln!("Warning: Failed to create storage directories: {}", e);
            }
        }

        storage
    }

    /// Get the Application Support directory for this app
    fn get_app_support_dir(config: &StorageConfig) -> Option<PathBuf> {
        // On macOS: ~/Library/Application Support/<org>/<app>/
        // On other platforms, use a fallback
        #[cfg(target_os = "macos")]
        {
            dirs::data_dir().map(|mut path| {
                if let Some(ref org) = config.org_name {
                    path.push(org);
                }
                path.push(&config.app_name);
                path
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Fallback for other platforms
            dirs::config_dir().map(|mut path| {
                if let Some(ref org) = config.org_name {
                    path.push(org);
                }
                path.push(&config.app_name);
                path
            })
        }
    }

    /// Ensure storage directories exist
    fn ensure_directories(&mut self) -> StorageResult<()> {
        if let Some(ref path) = self.base_path {
            fs::create_dir_all(path).map_err(StorageError::DirectoryCreation)?;
        }
        Ok(())
    }

    /// Get the base storage path
    pub fn base_path(&self) -> Option<&PathBuf> {
        self.base_path.as_ref()
    }

    /// Get the full path for a data file
    pub fn path_for(&self, name: &str) -> StorageResult<PathBuf> {
        self.base_path
            .as_ref()
            .map(|p| p.join(format!("{}.json", name)))
            .ok_or(StorageError::PathNotAvailable)
    }

    /// Save data to storage
    ///
    /// The data is serialized to JSON and saved to `<base_path>/<name>.json`.
    pub fn save<T: Serialize>(&self, name: &str, data: &T) -> StorageResult<()> {
        let path = self.path_for(name)?;

        let file = File::create(&path).map_err(StorageError::Write)?;
        let writer = BufWriter::new(file);

        if self.config.pretty_json {
            serde_json::to_writer_pretty(writer, data).map_err(StorageError::Serialize)?;
        } else {
            serde_json::to_writer(writer, data).map_err(StorageError::Serialize)?;
        }

        Ok(())
    }

    /// Load data from storage
    ///
    /// Returns `Ok(None)` if the file doesn't exist.
    /// Returns `Err` if the file exists but can't be read or parsed.
    pub fn load<T: DeserializeOwned>(&self, name: &str) -> StorageResult<Option<T>> {
        let path = self.path_for(name)?;

        if !path.exists() {
            return Ok(None);
        }

        let file = File::open(&path).map_err(StorageError::Read)?;
        let reader = BufReader::new(file);

        let data = serde_json::from_reader(reader).map_err(StorageError::Deserialize)?;
        Ok(Some(data))
    }

    /// Delete data from storage
    pub fn delete(&self, name: &str) -> StorageResult<()> {
        let path = self.path_for(name)?;

        if path.exists() {
            fs::remove_file(&path).map_err(StorageError::Write)?;
        }

        Ok(())
    }

    /// Check if data exists in storage
    pub fn exists(&self, name: &str) -> bool {
        self.path_for(name)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// List all stored data files
    pub fn list(&self) -> StorageResult<Vec<String>> {
        let base = self.base_path.as_ref().ok_or(StorageError::PathNotAvailable)?;

        let mut names = Vec::new();

        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem() {
                    if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                        names.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(names)
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new(StorageConfig::default())
    }
}

/// Preferences storage with typed access
///
/// A convenience wrapper around Storage for app preferences.
pub struct Preferences {
    storage: Storage,
    /// Name of the preferences file (without extension)
    file_name: String,
}

impl Preferences {
    /// Create preferences storage with the given base storage
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            file_name: "preferences".to_string(),
        }
    }

    /// Create preferences with a custom file name
    pub fn with_file_name(storage: Storage, file_name: impl Into<String>) -> Self {
        Self {
            storage,
            file_name: file_name.into(),
        }
    }

    /// Save all preferences
    pub fn save<T: Serialize>(&self, prefs: &T) -> StorageResult<()> {
        self.storage.save(&self.file_name, prefs)
    }

    /// Load all preferences
    pub fn load<T: DeserializeOwned>(&self) -> StorageResult<Option<T>> {
        self.storage.load(&self.file_name)
    }

    /// Load preferences or return default
    pub fn load_or_default<T: DeserializeOwned + Default>(&self) -> T {
        self.load().ok().flatten().unwrap_or_default()
    }
}

/// Auto-saver for debounced automatic saving
///
/// Tracks when data becomes "dirty" and schedules saves after a debounce delay.
pub struct AutoSaver {
    /// Whether data needs saving
    dirty: bool,
    /// When the data was last marked dirty
    dirty_since: Option<Instant>,
    /// Debounce delay
    debounce: Duration,
    /// Last save time
    last_save: Option<Instant>,
}

impl AutoSaver {
    /// Create a new auto-saver with the given debounce delay
    pub fn new(debounce: Duration) -> Self {
        Self {
            dirty: false,
            dirty_since: None,
            debounce,
            last_save: None,
        }
    }

    /// Mark data as dirty (needs saving)
    pub fn mark_dirty(&mut self) {
        if !self.dirty {
            self.dirty = true;
            self.dirty_since = Some(Instant::now());
        }
    }

    /// Mark data as clean (just saved)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.dirty_since = None;
        self.last_save = Some(Instant::now());
    }

    /// Check if data is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if we should save now (debounce has elapsed)
    pub fn should_save(&self) -> bool {
        if let Some(dirty_since) = self.dirty_since {
            dirty_since.elapsed() >= self.debounce
        } else {
            false
        }
    }

    /// Attempt to save if the debounce delay has passed
    ///
    /// Returns true if save was attempted, false if still debouncing.
    pub fn try_save<F, E>(&mut self, save_fn: F) -> Result<bool, E>
    where
        F: FnOnce() -> Result<(), E>,
    {
        if self.should_save() {
            save_fn()?;
            self.mark_clean();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the time until the next save should occur
    pub fn time_until_save(&self) -> Option<Duration> {
        self.dirty_since.map(|since| {
            let elapsed = since.elapsed();
            if elapsed >= self.debounce {
                Duration::ZERO
            } else {
                self.debounce - elapsed
            }
        })
    }
}

impl Default for AutoSaver {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.app_name, "SolApp");
        assert!(config.create_dirs);
        assert!(config.pretty_json);
    }

    #[test]
    fn test_auto_saver_dirty_tracking() {
        let mut saver = AutoSaver::new(Duration::from_millis(100));

        assert!(!saver.is_dirty());
        assert!(!saver.should_save());

        saver.mark_dirty();
        assert!(saver.is_dirty());
        assert!(!saver.should_save()); // Not yet, debounce not elapsed

        // Wait for debounce
        std::thread::sleep(Duration::from_millis(150));
        assert!(saver.should_save());

        saver.mark_clean();
        assert!(!saver.is_dirty());
        assert!(!saver.should_save());
    }

    #[test]
    fn test_auto_saver_try_save() {
        let mut saver = AutoSaver::new(Duration::from_millis(10));
        let mut save_count = 0;

        saver.mark_dirty();

        // Shouldn't save yet
        let result: Result<bool, ()> = saver.try_save(|| {
            save_count += 1;
            Ok(())
        });
        assert!(!result.unwrap());
        assert_eq!(save_count, 0);

        // Wait for debounce
        std::thread::sleep(Duration::from_millis(20));

        // Should save now
        let result: Result<bool, ()> = saver.try_save(|| {
            save_count += 1;
            Ok(())
        });
        assert!(result.unwrap());
        assert_eq!(save_count, 1);
        assert!(!saver.is_dirty());
    }

    #[test]
    fn test_storage_path_generation() {
        let config = StorageConfig {
            app_name: "TestApp".to_string(),
            org_name: Some("TestOrg".to_string()),
            create_dirs: false,
            pretty_json: true,
        };
        let storage = Storage::new(config);

        // Just test that path_for works
        if let Ok(path) = storage.path_for("test") {
            assert!(path.to_string_lossy().contains("test.json"));
        }
    }
}
