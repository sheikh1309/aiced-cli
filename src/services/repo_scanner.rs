use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use futures::{stream, StreamExt};
use crate::adapters::ailyzer_adapter::AiLyzerAdapter;
use crate::errors::AilyzerResult;
use crate::helpers::prompt_generator;
use crate::logger::animated_logger::AnimatedLogger;
use crate::structs::analyze_request::AnalyzeRequest;
use crate::structs::analyze_response::AnalyzeResponse;
use crate::structs::api_response::ApiResponse;
use crate::structs::config::repository_config::RepositoryConfig;
use crate::structs::file_info::FileInfo;
use crate::structs::files_cache::FilesCache;

pub struct RepoScanner {
    repository_config: Arc<RepositoryConfig>,
    max_concurrent_reads: usize,
    adapter: Arc<AiLyzerAdapter>
}

impl RepoScanner {
    pub fn new(repository_config: Arc<RepositoryConfig>, adapter: Arc<AiLyzerAdapter>) -> Self {
        Self { repository_config, max_concurrent_reads: 10, adapter  }
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

    pub async fn scan_files(&self) -> AilyzerResult<Vec<FileInfo>> {
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

    async fn get_filtered_files(&self, repo_files_paths: Vec<PathBuf>, cache_path: &Path) -> AilyzerResult<Vec<PathBuf>> {
        if let Some(cache) = FilesCache::load_from_file(cache_path)? {
            if cache.is_valid_for(&repo_files_paths) {
                log::info!("📋 Using cached AI filter results ({} files)", cache.files.len());
                return Ok(cache.to_path_bufs());
            }
        }

        self.run_ai_filtering_and_cache(repo_files_paths, cache_path).await
    }

    async fn run_ai_filtering_and_cache(&self, repo_files_paths: Vec<PathBuf>, cache_path: &Path) -> AilyzerResult<Vec<PathBuf>> {
        log::info!("🤖 Running AI filtering on {} files...", repo_files_paths.len());

        let filtered_paths = self.filter_files(repo_files_paths.clone()).await?;

        // Create and save cache
        let cache = FilesCache::from_data(&filtered_paths, &repo_files_paths);
        cache.save_to_file(cache_path)?;

        Ok(filtered_paths)
    }

    async fn process_files(&self, file_paths: Vec<PathBuf>) -> AilyzerResult<Vec<FileInfo>> {
        log::info!("📁 Found {} files to analyze", file_paths.len());

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
                        log::error!("⚠️ Error reading {}: {}", path.display(), e);
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
                            log::info!("📊 Progress: {}/{} files processed", processed, total_files);
                        }
                        Some(file_info)
                    }
                    Err(_) => None,
                }
            })
            .collect()
            .await;

        log::info!("✅ Processed {} files successfully", files.len());
        Ok(files)
    }

    async fn filter_files(&self, repo_files_paths: Vec<PathBuf>) -> AilyzerResult<Vec<PathBuf>> {
        let user_prompt = prompt_generator::generate_file_filter_user_prompt(&repo_files_paths, &self.repository_config.path);

        let mut logger = AnimatedLogger::new("File Filtering".to_string());
        logger.start();

        let request_body = AnalyzeRequest { prompt: user_prompt };
        let filter_data: ApiResponse<AnalyzeResponse> = self.adapter.post_json_extract_data("api/files_filter", &request_body, &mut logger, "File filtering").await?;

        let content = &filter_data.data.unwrap().content
            .replace("```json", "")
            .replace("```", "")
            .trim()
            .to_string();

        let filtered_files_paths: Vec<PathBuf> = serde_json::from_str::<Vec<String>>(content)
            .unwrap_or_else(|e| {
                log::error!("Failed to parse filtered files JSON: {}", e);
                Vec::new()
            })
            .into_iter()
            .map(|file_path| {
                let str_path = format!("{}/{}", &self.repository_config.path, file_path).replace("//", "/");
                PathBuf::from(str_path)
            })
            .collect();

        logger.stop("File filtering complete").await;
        Ok(filtered_files_paths)
    }

    async fn load_gitignore(&self, repo_path: &str) -> AilyzerResult<HashSet<String>> {
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

    async fn collect_file_paths(&self, dir: &Path, patterns: &HashSet<String>) -> AilyzerResult<Vec<PathBuf>> {
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