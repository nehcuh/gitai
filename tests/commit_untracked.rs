use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

// Helper to run a command and return its output.
fn run_command(command: &str, args: &[&str], cwd: &Path) -> String {
    let output = Command::new(command)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect(&format!("Failed to execute command: {}", command));

    if !output.status.success() {
        panic!(
            "Command `{}` failed with exit code {:?}:\nSTDOUT: {}\nSTDERR: {}",
            command,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8(output.stdout).expect("Failed to convert stdout to string")
}

// Helper to set up a git repository in a temporary directory for testing.
fn setup_test_repo(repo_path: &Path) {
    run_command("git", &["init"], repo_path);
    run_command("git", &["config", "user.name", "Test User"], repo_path);
    run_command(
        "git",
        &["config", "user.email", "test@example.com"],
        repo_path,
    );

    // Create and commit an initial file.
    fs::write(repo_path.join("a.txt"), "initial content").unwrap();
    run_command("git", &["add", "a.txt"], repo_path);
    run_command("git", &["commit", "-m", "Initial commit"], repo_path);

    // Modify the tracked file.
    fs::write(repo_path.join("a.txt"), "modified content").unwrap();

    // Create a new untracked file.
    fs::write(repo_path.join("b.txt"), "untracked file content").unwrap();

    // Create a .gitignore file and an ignored file.
    fs::write(repo_path.join(".gitignore"), "*.log").unwrap();
    fs::write(repo_path.join("c.log"), "this is a log file").unwrap();
}

#[test]
fn test_commit_all_with_untracked_files() {
    // 1. Setup
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    setup_test_repo(&repo_path);

    let gitai_path = env!("CARGO_BIN_EXE_gitai");

    // 2. Run `gitai commit -a`
    // We need to pipe "y" to stdin twice:
    // - First for confirming to stage untracked files.
    // - Second for confirming the generated commit message.
    let mut child = Command::new(gitai_path)
        .arg("commit")
        .arg("-a") // Use the "-a" flag
        .current_dir(&repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn gitai process");

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        // Answering "yes" to "add untracked files?"
        // and "yes" to "execute this commit?"
        stdin
            .write_all(b"y\ny\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    if !output.status.success() {
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("gitai commit -a failed");
    }

    // 3. Assertions
    // Check git status, should be clean.
    let status = run_command("git", &["status", "--porcelain"], &repo_path);
    assert!(
        !status.contains("a.txt"),
        "a.txt should have been committed"
    );
    assert!(
        !status.contains("b.txt"),
        "b.txt should have been committed"
    );
    assert!(
        status.contains("c.log"),
        "c.log should remain as an untracked, ignored file"
    );

    // Check the last commit message.
    let log = run_command("git", &["log", "-1", "--pretty=%B"], &repo_path);
    // The fallback commit message is generic. We just check that a commit was made.
    assert!(!log.is_empty(), "A new commit should have been created.");
    assert!(
        log.contains("Update files"),
        "Commit message should be the fallback message."
    );

    // Check that both a.txt and b.txt are in the last commit.
    let files_in_commit = run_command(
        "git",
        &["diff", "--name-only", "HEAD~1", "HEAD"],
        &repo_path,
    );
    assert!(files_in_commit.contains("a.txt"));
    assert!(files_in_commit.contains("b.txt"));
    assert!(!files_in_commit.contains("c.log"));
}
