use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::errors::AicedResult;

#[derive(Serialize, Deserialize, Clone)]
pub struct FilesCache {
    pub files: Vec<String>,
    pub last_modified: u64,
    pub total_files_count: usize,
}

impl FilesCache {
   
    pub fn from_data(filtered_files: &[PathBuf], all_files: &[PathBuf]) -> Self {
        Self {
            files: filtered_files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            last_modified: Self::current_timestamp(),
            total_files_count: all_files.len(),
        }
    }

    pub fn load_from_file(cache_path: &Path) -> AicedResult<Option<Self>> {
        if !cache_path.exists() {
            log::info!("📋 No cache file found, running AI filtering for the first time");
            return Ok(None);
        }

        let content = std::fs::read_to_string(cache_path)?;

        match toml::from_str::<Self>(&content) {
            Ok(cache) => Ok(Some(cache)),
            Err(_) => {
                log::error!("⚠️ Invalid cache file format, recreating");
                Ok(None)
            }
        }
    }

    pub fn save_to_file(&self, cache_path: &Path) -> AicedResult<()> {
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let cache_content = toml::to_string_pretty(self).unwrap();
        std::fs::write(cache_path, cache_content)?;

        log::info!("💾 Cache updated with {} filtered files", self.files.len());
        Ok(())
    }

    pub fn is_valid_for(&self, current_files: &[PathBuf]) -> bool {
        if self.total_files_count != current_files.len() {
            log::info!("🔄 File count changed ({} -> {}), need to re-run AI filtering", self.total_files_count, current_files.len());
            return false;
        }

        // Compare actual file sets
        let cached_files: HashSet<String> = self.files.iter().cloned().collect();
        let current_files_set: HashSet<String> = current_files.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let diff: HashSet<String> = cached_files.difference(&current_files_set).cloned().collect();

        if diff.len() > 0 {
            log::info!("🔄 File list changed, need to re-run AI filtering");
            return false;
        }

        true
    }

    pub fn to_path_bufs(&self) -> Vec<PathBuf> {
        self.files.iter().map(|s| PathBuf::from(s)).collect()
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}