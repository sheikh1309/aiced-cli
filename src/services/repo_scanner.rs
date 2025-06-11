use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use futures::{stream, StreamExt};
use crate::structs::file_info::FileInfo;

pub struct RepoScanner {
    repo_path: String,
    max_concurrent_reads: usize,
}

impl RepoScanner {
    pub fn new(repo_path: String) -> Self {
        Self::with_concurrency(repo_path, 10)
    }

    pub fn with_concurrency(repo_path: String, max_concurrent_reads: usize) -> Self {
        Self { 
            repo_path,
            max_concurrent_reads,
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
        
        
        // todo - remove
        patterns.insert(String::from("test/"));
        patterns.insert(String::from("dist/"));
        patterns.insert(String::from("messages/"));
        patterns.insert(String::from(".vscode/"));
        patterns.insert(String::from("yarn.lock"));

        Ok(patterns)
    }

    pub async fn scan_files_async(&self) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let patterns = self.load_gitignore(&self.repo_path).await?;

        // First, collect all file paths
        let file_paths = self.collect_file_paths_async(Path::new(&self.repo_path), &patterns).await?;

        println!("📁 Found {} files to analyze", file_paths.len());

        // Process files concurrently with progress tracking
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
            .buffer_unordered(self.max_concurrent_reads) // Process N files concurrently
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

    async fn collect_file_paths_async(&self, dir: &Path, patterns: &HashSet<String>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut paths = Vec::new();
        let mut dirs_to_process = vec![dir.to_path_buf()];

        while let Some(current_dir) = dirs_to_process.pop() {
            let mut entries = fs::read_dir(&current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                let relative_path = path.strip_prefix(&self.repo_path)
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