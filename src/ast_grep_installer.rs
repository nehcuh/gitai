// AST-grep executable installation and detection
// Provides automatic installation of ast-grep if not found

use crate::errors::AppError;
use std::process::Command;
use std::path::PathBuf;
use colored::Colorize;

/// AST-grep installer and detector
pub struct AstGrepInstaller {
    // Track installation status
    installation_attempted: bool,
}

impl AstGrepInstaller {
    /// Create a new installer instance
    pub fn new() -> Self {
        Self {
            installation_attempted: false,
        }
    }

    /// Check if ast-grep executable is available in the system
    /// Returns the path to the executable if found
    pub fn detect_ast_grep(&self) -> Option<PathBuf> {
        // Check for 'sg' first (the standard ast-grep executable name)
        if let Ok(sg_path) = which::which("sg") {
            tracing::info!("Found ast-grep executable 'sg' at: {}", sg_path.display());
            return Some(sg_path);
        }

        // Check for 'ast-grep' as well (alternative name)
        if let Ok(ast_grep_path) = which::which("ast-grep") {
            tracing::info!("Found ast-grep executable 'ast-grep' at: {}", ast_grep_path.display());
            return Some(ast_grep_path);
        }

        tracing::warn!("ast-grep executable not found in PATH");
        None
    }

    /// Ensure ast-grep is available, install if necessary
    pub async fn ensure_ast_grep_available(&mut self) -> Result<PathBuf, AppError> {
        // First check if it's already available
        if let Some(path) = self.detect_ast_grep() {
            return Ok(path);
        }

        // If not found and we haven't attempted installation yet, try to install
        if !self.installation_attempted {
            println!("{}", "🔍 ast-grep 可执行文件未找到，正在尝试自动安装...".yellow());
            
            match self.install_ast_grep().await {
                Ok(path) => {
                    println!("{}", "✅ ast-grep 安装成功！".green());
                    self.installation_attempted = true;
                    return Ok(path);
                }
                Err(e) => {
                    println!("{}", format!("❌ ast-grep 自动安装失败: {}", e).red());
                    self.installation_attempted = true;
                    
                    // Provide manual installation instructions
                    self.print_manual_installation_instructions();
                    
                    return Err(AppError::Generic(format!(
                        "ast-grep 可执行文件不可用，自动安装失败: {}. 请手动安装或检查系统环境。", e
                    )));
                }
            }
        }

        // If we've already attempted installation and it failed, return error
        Err(AppError::Generic(
            "ast-grep 可执行文件不可用，且自动安装已失败。请手动安装 ast-grep。".to_string()
        ))
    }

    /// Install ast-grep using cargo
    async fn install_ast_grep(&self) -> Result<PathBuf, AppError> {
        println!("{}", "📦 正在使用 cargo install ast-grep 进行安装...".blue());
        
        // Check if cargo is available
        if which::which("cargo").is_err() {
            return Err(AppError::Generic(
                "cargo 命令不可用，无法自动安装 ast-grep".to_string()
            ));
        }

        // Execute cargo install command
        let mut cmd = Command::new("cargo");
        cmd.arg("install")
           .arg("ast-grep")
           .arg("--locked"); // Use locked dependencies for reproducible builds

        println!("{}", "⏳ 正在执行安装，这可能需要几分钟时间...".cyan());
        
        let output = cmd.output()
            .map_err(|e| AppError::Generic(format!("执行 cargo install 失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            tracing::error!("cargo install ast-grep failed. stdout: {}, stderr: {}", stdout, stderr);
            
            return Err(AppError::Generic(format!(
                "cargo install ast-grep 失败: {}",
                if !stderr.is_empty() { stderr } else { stdout }
            )));
        }

        println!("{}", "🔄 验证安装结果...".cyan());
        
        // Wait a moment for the installation to complete
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Check if ast-grep is now available
        if let Some(path) = self.detect_ast_grep() {
            Ok(path)
        } else {
            Err(AppError::Generic(
                "ast-grep 安装完成但仍然无法在 PATH 中找到可执行文件".to_string()
            ))
        }
    }

    /// Print manual installation instructions
    fn print_manual_installation_instructions(&self) {
        println!("\n{}", "📋 手动安装 ast-grep 的方法：".bold().blue());
        println!("{}", "1. 使用 Cargo 安装：".bold());
        println!("   {}", "cargo install ast-grep".green());
        println!("{}", "2. 使用 Homebrew 安装 (macOS)：".bold());
        println!("   {}", "brew install ast-grep".green());
        println!("{}", "3. 使用包管理器安装 (Linux)：".bold());
        println!("   {}", "# Ubuntu/Debian: sudo apt install ast-grep".green());
        println!("   {}", "# Arch Linux: sudo pacman -S ast-grep".green());
        println!("{}", "4. 从源码编译：".bold());
        println!("   {}", "git clone https://github.com/ast-grep/ast-grep.git".green());
        println!("   {}", "cd ast-grep && cargo build --release".green());
        println!("\n{}", "安装完成后，请确保 'sg' 命令在您的 PATH 中可用。".yellow());
    }

    /// Check if cargo is available for installation
    pub fn is_cargo_available(&self) -> bool {
        which::which("cargo").is_ok()
    }

    /// Get detailed system information for troubleshooting
    pub fn get_system_info(&self) -> SystemInfo {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let cargo_available = self.is_cargo_available();
        let ast_grep_path = self.detect_ast_grep();

        SystemInfo {
            os: os.to_string(),
            arch: arch.to_string(),
            cargo_available,
            ast_grep_path,
        }
    }
}

impl Default for AstGrepInstaller {
    fn default() -> Self {
        Self::new()
    }
}

/// System information for troubleshooting
#[derive(Debug)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cargo_available: bool,
    pub ast_grep_path: Option<PathBuf>,
}

impl SystemInfo {
    /// Print system information
    pub fn print(&self) {
        println!("{}", "🔍 系统信息：".bold().blue());
        println!("   操作系统: {}", self.os);
        println!("   架构: {}", self.arch);
        println!("   Cargo 可用: {}", if self.cargo_available { "是".green() } else { "否".red() });
        println!("   ast-grep 路径: {}", 
            if let Some(path) = &self.ast_grep_path {
                path.display().to_string().green()
            } else {
                "未找到".red()
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_creation() {
        let installer = AstGrepInstaller::new();
        assert!(!installer.installation_attempted);
    }

    #[test]
    fn test_detect_ast_grep() {
        let installer = AstGrepInstaller::new();
        // This test will pass or fail depending on whether ast-grep is installed
        // Just ensure the function runs without panic
        let _result = installer.detect_ast_grep();
    }

    #[test]
    fn test_cargo_availability() {
        let installer = AstGrepInstaller::new();
        // This should normally be true in a Rust development environment
        let _available = installer.is_cargo_available();
    }

    #[test]
    fn test_system_info() {
        let installer = AstGrepInstaller::new();
        let info = installer.get_system_info();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }
}