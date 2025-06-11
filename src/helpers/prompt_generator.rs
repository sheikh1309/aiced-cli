use crate::structs::file_info::FileInfo;

pub fn generate_prompt(files: Vec<FileInfo>, repo_path: &str) -> String {
    let estimated_size = files.iter().map(|f| f.content.len() * 2).sum::<usize>();
    let mut prompt = String::with_capacity(estimated_size);
    
    for file in files {
        let path = file.path.replace(repo_path, "");
        let line_count = file.content.lines().count();

        prompt.push_str("File: ");
        prompt.push_str(&path);
        prompt.push_str(" \nTotal lines: ");
        prompt.push_str(&line_count.to_string());
        prompt.push('\n');

        for (i, line) in file.content.lines().enumerate() {
            prompt.push_str(&format!("{:4}: {}\n", i + 1, line));
        }

        prompt.push_str("\n=== END OF ");
        prompt.push_str(&path);
        prompt.push_str(" (lines 1-");
        prompt.push_str(&line_count.to_string());
        prompt.push_str(") ===\n\n");
    }

    prompt.push_str("CRITICAL: Use EXACT line numbers from above. If you reference line 562, ensure it exists in the file.\n");
    prompt
}