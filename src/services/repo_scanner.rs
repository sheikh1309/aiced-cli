use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use futures::{stream, StreamExt};
use crate::prompts::file_filter_system_prompt::FILE_FILTER_SYSTEM_PROMPT;
use crate::helpers::prompt_generator;
use crate::logger::animated_logger::AnimatedLogger;
use crate::services::anthropic::AnthropicProvider;
use crate::structs::config::repository_config::RepositoryConfig;
use crate::structs::file_info::FileInfo;
use crate::structs::files_cache::FilesCache;

pub struct RepoScanner {
    anthropic_provider: Arc<AnthropicProvider>,
    repository_config: Arc<RepositoryConfig>,
    max_concurrent_reads: usize,
}

impl RepoScanner {
    pub fn new(anthropic_provider: Arc<AnthropicProvider>, repository_config: Arc<RepositoryConfig>) -> Self {
        Self {
            anthropic_provider,
            repository_config,
            max_concurrent_reads: 10
        }
    }

    fn get_default_image_patterns(&self) -> HashSet<String> {
        let image_extensions = vec![
            "*.jpg", "*.jpeg", "*.png", "*.gif", "*.bmp", "*.tiff", "*.tif",
            "*.webp", "*.ico", "*.cur",
            "*.raw", "*.cr2", "*.nef", "*.arw", "*.dng", "*.orf", "*.rw2",
            "*.svg", "*.eps", "*.ai", "*.pdf",
            "*.psd", "*.xcf", "*.sketch", "*.fig",
            "Thumbs.db", ".DS_Store", "*.tmp",
            "images/", "img/", "assets/images/", "static/images/",
            "public/images/", "src/assets/", "assets/img/",
        ];

        image_extensions.into_iter().map(String::from).collect()
    }

    pub async fn scan_files(&self) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let patterns = self.load_gitignore(&self.repository_config.path).await?;
        let repo_files_paths = self.collect_file_paths(Path::new(&self.repository_config.path), &patterns).await?;

        let cache_path = self.get_cache_file_path();
        let files_to_analyze = self.get_filtered_files(repo_files_paths, &cache_path).await?;
        let files = self.process_files(files_to_analyze).await?;

        Ok(files)
    }

    fn get_cache_file_path(&self) -> PathBuf {
        let cache_name = format!("ailyzer/{}.toml", self.repository_config.name);
        dirs::home_dir()
            .map(|d| d.join(cache_name))
            .unwrap_or_default()
    }

    async fn get_filtered_files(&self, repo_files_paths: Vec<PathBuf>, cache_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        // Try to load from cache first
        if let Some(cache) = FilesCache::load_from_file(cache_path)? {
            if cache.is_valid_for(&repo_files_paths) {
                println!("📋 Using cached AI filter results ({} files)", cache.files.len());
                return Ok(cache.to_path_bufs());
            }
        }

        // Cache miss - run AI filtering and update cache
        self.run_ai_filtering_and_cache(repo_files_paths, cache_path).await
    }

    async fn run_ai_filtering_and_cache(&self, repo_files_paths: Vec<PathBuf>, cache_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        println!("🤖 Running AI filtering on {} files...", repo_files_paths.len());

        let filtered_paths = self.filter_files(repo_files_paths.clone()).await?;

        // Create and save cache
        let cache = FilesCache::from_data(&filtered_paths, &repo_files_paths);
        cache.save_to_file(cache_path)?;

        Ok(filtered_paths)
    }

    async fn process_files(&self, file_paths: Vec<PathBuf>) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        println!("📁 Found {} files to analyze", file_paths.len());

        let total_files = file_paths.len();
        let mut processed = 0;

        let files: Vec<FileInfo> = stream::iter(file_paths)
            .map(|path| async move {
                match fs::read_to_string(&path).await {
                    Ok(content) => Ok(FileInfo {
                        path: path.to_string_lossy().to_string(),
                        content,
                    }),
                    Err(e) => {
                        eprintln!("⚠️ Error reading {}: {}", path.display(), e);
                        Err(e)
                    }
                }
            })
            .buffer_unordered(self.max_concurrent_reads)
            .filter_map(|result| async move {
                match result {
                    Ok(file_info) => {
                        processed += 1;
                        if processed % 100 == 0 {
                            println!("📊 Progress: {}/{} files processed", processed, total_files);
                        }
                        Some(file_info)
                    }
                    Err(_) => None,
                }
            })
            .collect()
            .await;

        println!("✅ Processed {} files successfully", files.len());
        Ok(files)
    }

    async fn filter_files(&self, repo_files_paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let user_prompt = prompt_generator::generate_file_filter_user_prompt(&repo_files_paths, &self.repository_config.path);

        let mut logger = AnimatedLogger::new("File Filtering".to_string());
        logger.start();

        let mut response_text = String::new();
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(150);
        let mut stream = self.anthropic_provider.trigger_stream_request(FILE_FILTER_SYSTEM_PROMPT.to_string(), vec![user_prompt]).await?;

        while let Some(result) = stream.next().await {
            if last_update.elapsed() >= update_interval {
                last_update = Instant::now();
            }

            match result {
                Ok(item) => {
                    response_text.push_str(&item.content);
                },
                Err(_e) => {},
            }
        }
        
        logger.stop("File Filtering complete").await;

        let clean_response = response_text
            .replace("```json", "")
            .replace("```", "")
            .trim()
            .to_string();

        let filtered_files_paths: Vec<PathBuf> = serde_json::from_str::<Vec<String>>(&clean_response)
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse JSON response: {}", e);
                eprintln!("Response was: {}", clean_response);
                Vec::new()
            })
            .into_iter()
            .map(|file_path| {
                let str_path = format!("{}/{}", &self.repository_config.path, file_path).replace("//", "/");
                PathBuf::from(str_path)
            })
            .collect();

        Ok(filtered_files_paths)
    }

    async fn load_gitignore(&self, repo_path: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
        let gitignore_path = format!("{}/.gitignore", repo_path);
        let mut patterns = self.get_default_image_patterns();
        patterns.insert(String::from(".git/"));

        if let Ok(content) = fs::read_to_string(gitignore_path).await {
            let gitignore_patterns: HashSet<String> = content
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(|line| line.to_string())
                .collect();

            patterns.extend(gitignore_patterns);
        }
        Ok(patterns)
    }

    async fn collect_file_paths(&self, dir: &Path, patterns: &HashSet<String>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut paths = Vec::new();
        let mut dirs_to_process = vec![dir.to_path_buf()];

        while let Some(current_dir) = dirs_to_process.pop() {
            let mut entries = fs::read_dir(&current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                let relative_path = path.strip_prefix(&self.repository_config.path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                if self.should_ignore_path(&relative_path, &path, patterns) {
                    continue;
                }

                if metadata.is_file() {
                    paths.push(path);
                } else if metadata.is_dir() {
                    dirs_to_process.push(path);
                }
            }
        }

        Ok(paths)
    }
    
    fn should_ignore_path(&self, relative_path: &str, full_path: &Path, patterns: &HashSet<String>) -> bool {
        let file_name = full_path.file_name().unwrap_or_default().to_string_lossy();

        if patterns.contains(relative_path) || patterns.contains(&*file_name) {
            return true;
        }

        for pattern in patterns {
            if self.matches_gitignore_pattern(&relative_path, &file_name, full_path, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_gitignore_pattern(&self, relative_path: &str, file_name: &str, full_path: &Path, pattern: &str) -> bool {
        // Handle root-relative patterns starting with /
        if pattern.starts_with('/') {
            let root_pattern = &pattern[1..]; // Remove leading /

            // For root-relative patterns, only match at the root level
            if root_pattern.ends_with('/') {
                // Directory pattern like "/target/"
                let dir_pattern = &root_pattern[..root_pattern.len()-1];
                if full_path.is_dir() {
                    // Check if this is a top-level directory
                    let path_components: Vec<&str> = relative_path.split('/').collect();
                    return path_components.len() == 1 && self.matches_glob(&path_components[0], dir_pattern);
                }
                return false;
            } else {
                // File or directory pattern like "/target" or "/Cargo.lock"
                let path_components: Vec<&str> = relative_path.split('/').collect();
                return path_components.len() == 1 && self.matches_glob(&path_components[0], root_pattern);
            }
        }

        // Directory patterns ending with /
        if pattern.ends_with('/') {
            let dir_pattern = &pattern[..pattern.len()-1];
            if full_path.is_dir() {
                return self.matches_glob(relative_path, dir_pattern) || self.matches_glob(file_name, dir_pattern);
            }
            return false;
        }

        // Extension patterns like "*.rs"
        if pattern.starts_with("*.") {
            let ext = &pattern[2..];
            if let Some(file_ext) = full_path.extension() {
                return file_ext == ext;
            }
            return false;
        }

        // Hidden files/directories starting with .
        if pattern.starts_with('.') && !pattern.contains('*') {
            return file_name.starts_with('.') && file_name == pattern;
        }

        // Glob patterns with wildcards
        if pattern.contains('*') {
            return self.matches_glob(relative_path, pattern) || self.matches_glob(file_name, pattern);
        }

        // Exact match
        relative_path == pattern || file_name == pattern
    }

    fn matches_glob(&self, text: &str, pattern: &str) -> bool {
        // Handle simple cases
        if pattern == "*" {
            return true;
        }

        if pattern == text {
            return true;
        }

        // Handle patterns like "*.ext"
        if pattern.starts_with("*.") {
            let ext = &pattern[2..];
            return text.ends_with(&format!(".{}", ext));
        }

        // Handle patterns like "prefix*"
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len()-1];
            return text.starts_with(prefix);
        }

        // Handle patterns like "*suffix"
        if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            return text.ends_with(suffix);
        }

        // Handle patterns like "prefix*suffix"
        if let Some(star_pos) = pattern.find('*') {
            let (prefix, suffix_with_star) = pattern.split_at(star_pos);
            let suffix = &suffix_with_star[1..]; // Remove the '*'
            return text.starts_with(prefix) && text.ends_with(suffix) && text.len() >= prefix.len() + suffix.len();
        }

        // No wildcard, check for substring match (common in gitignore)
        text.contains(pattern)
    }
}