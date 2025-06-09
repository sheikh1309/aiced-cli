use crate::structs::file_info::FileInfo;

pub fn generate_prompt(files: Vec<FileInfo>, repo_path: &String) -> String {
    let mut prompt = String::from("");
    for file in files {
        let path = file.path.replace(repo_path, "");
        prompt.push_str(&format!("File: {} \n", &path));
        prompt.push_str(&format!("Total lines: {}\n", file.content.lines().count()));
        for (i, line) in file.content.lines().enumerate() {
            prompt.push_str(&format!("{:4}: {}\n", i + 1, line));
        }

        prompt.push_str(&format!("\n=== END OF {} (lines 1-{}) ===\n\n", &path, file.content.lines().count()));
    }

    prompt.push_str("CRITICAL: Use EXACT line numbers from above. If you reference line 562, ensure it exists in the file.\n");
    prompt
}