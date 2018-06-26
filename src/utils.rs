use std::process::Command;

pub fn get_project_files() -> Vec<String> {
    let ls_files_output = Command::new("git")
        .arg("ls-files")
        .output()
        .expect("Failed to run git ls-files. Make sure you're in a git repo.");

    String::from_utf8(ls_files_output.stdout)
        .unwrap()
        .split("\n")
        .map(str::to_string)
        .collect()
}
