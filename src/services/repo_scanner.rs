use std::collections::HashSet;
use std::fs;
use std::path::Path;
use crate::structs::file_info::FileInfo;

pub struct RepoScanner {
    repo_path: String
}

impl RepoScanner {
    pub fn new(repo_path: String) -> Self {
        Self { repo_path }
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

    fn load_gitignore(&self, repo_path: &str) -> Result<HashSet<String>, std::io::Error> {
        let gitignore_path = format!("{}/.gitignore", repo_path);

        let mut patterns: HashSet<String> = self.get_default_image_patterns();
        patterns.extend(HashSet::from([String::from(".git/")]));

        match fs::read_to_string(gitignore_path) {
            Ok(content) => {
                let gitignore_patterns: HashSet<String> = content
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(|line| line.to_string())
                    .collect();

                patterns.extend(gitignore_patterns);
            },
            Err(_) => {
                eprintln!("Warning: .gitignore file not found, using default image patterns only");
            }
        }

        Ok(patterns)
    }

    pub fn scan_files(&self) -> Vec<FileInfo> {
        let mut patterns = self.load_gitignore(self.repo_path.as_str()).unwrap_or_else(|_e| HashSet::new());
        patterns.insert(String::from("test/"));
        patterns.insert(String::from("dist/"));
        patterns.insert(String::from("messages/"));
        patterns.insert(String::from(".vscode/"));
        patterns.insert(String::from("yarn.lock"));
        let mut files: Vec<FileInfo> = Vec::new();
        self.collect_files(Path::new(&self.repo_path), &patterns, &mut files, &self.repo_path);
        files
    }

    fn collect_files(&self, dir: &Path, patterns: &HashSet<String>, files: &mut Vec<FileInfo>, repo_root: &str) {
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();

                    let relative_path = if let Ok(rel) = path.strip_prefix(repo_root) {
                        rel.to_string_lossy().to_string()
                    } else {
                        path.to_string_lossy().to_string()
                    };

                    if self.should_ignore_path(&relative_path, &path, patterns) {
                        continue;
                    }

                    if path.is_file() {
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                let file = FileInfo {
                                    path: path.to_string_lossy().to_string(),
                                    content
                                };
                                files.push(file)
                            },
                            Err(e) => {
                                eprintln!("Error reading file {:?}: {}", path, e);
                            }
                        }
                    } else if path.is_dir() && !self.should_ignore_path(&relative_path, &path, patterns) {
                        self.collect_files(&path, patterns, files, repo_root);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error reading directory {:?}: {}", dir, e);
            }
        }
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