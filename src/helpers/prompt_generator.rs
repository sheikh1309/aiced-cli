use crate::structs::file_info::FileInfo;

pub fn generate_prompt(files: Vec<FileInfo>, repo_path: &String) -> String {
    let mut prompt = String::from("");
    for file in files {
        prompt.push_str(&format!(
            "File: {} \n{}\n",
            file.path.replace(repo_path, ""), file.content
        ));
    }

    prompt
}