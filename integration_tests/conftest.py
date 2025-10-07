"""
GitAI 集成测试配置和共享fixtures
"""

import pytest
import subprocess
import json
import time
import os
import tempfile
import shutil
from pathlib import Path
from typing import Dict, Any, Optional


class GitAIHelper:
    """GitAI命令行工具的辅助类"""

    def __init__(self, binary_path: str = None):
        if binary_path is None:
            # 默认使用项目根目录下的Release版本
            self.binary_path = Path(__file__).parent.parent / "target" / "release" / "gitai"
        else:
            self.binary_path = Path(binary_path)

        if not self.binary_path.exists():
            raise FileNotFoundError(f"GitAI binary not found at {self.binary_path}")

    def run_command(self, args: list, timeout: int = 30, capture_output: bool = True) -> subprocess.CompletedProcess:
        """运行GitAI命令"""
        cmd = [str(self.binary_path)] + args
        try:
            return subprocess.run(
                cmd,
                timeout=timeout,
                capture_output=capture_output,
                text=True,
                cwd=self.binary_path.parent.parent
            )
        except subprocess.TimeoutExpired:
            raise TimeoutError(f"Command timed out after {timeout}s: {' '.join(cmd)}")

    def get_features(self) -> Dict[str, Any]:
        """获取功能特性"""
        result = self.run_command(["features"])
        if result.returncode == 0:
            return self._parse_features_output(result.stdout)
        return {}

    def _parse_features_output(self, output: str) -> Dict[str, Any]:
        """解析features命令的输出"""
        features = {}
        for line in output.split('\n'):
            if '🔒' in line and '安全扫描' in line:
                features['security'] = '已启用' in line
            elif '📊' in line and '完整分析' in line:
                features['full_analysis'] = '已启用' in line
            elif '⚡' in line and '最小配置' in line:
                features['minimal'] = '已启用' in line
        return features

    def get_config(self) -> Dict[str, str]:
        """获取当前配置"""
        result = self.run_command(["config", "show"])
        config = {}
        if result.returncode == 0:
            for line in result.stdout.split('\n'):
                if ': ' in line:
                    key, value = line.split(': ', 1)
                    config[key.strip()] = value.strip()
        return config


class MCPServerManager:
    """MCP服务器管理器"""

    def __init__(self, gitai_helper: GitAIHelper):
        self.gitai = gitai_helper
        self.process = None

    def start_server(self, transport: str = "stdio", port: int = 8711) -> bool:
        """启动MCP服务器"""
        try:
            if transport == "http":
                # 启动HTTP模式的MCP服务器
                self.process = subprocess.Popen([
                    str(self.gitai.binary_path),
                    "mcp",
                    "--transport", "http",
                    "--port", str(port)
                ], stdout=subprocess.PIPE, stderr=subprocess.PIPE)

                # 等待服务器启动
                time.sleep(2)

                # 检查服务器是否正在运行
                if self.process.poll() is None:
                    return True
                else:
                    stdout, stderr = self.process.communicate()
                    print(f"MCP Server failed to start: {stderr}")
                    return False
            else:
                # stdio模式不在这里测试，需要特殊的交互处理
                return True

        except Exception as e:
            print(f"Failed to start MCP server: {e}")
            return False

    def stop_server(self):
        """停止MCP服务器"""
        if self.process:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
            self.process = None

    def is_running(self) -> bool:
        """检查服务器是否正在运行"""
        return self.process is not None and self.process.poll() is None


@pytest.fixture(scope="session")
def gitai_helper():
    """提供GitAI命令行工具的辅助实例"""
    return GitAIHelper()


@pytest.fixture(scope="session")
def mcp_server(gitai_helper):
    """提供MCP服务器实例"""
    server = MCPServerManager(gitai_helper)
    yield server
    server.stop_server()


@pytest.fixture
def temp_git_repo():
    """提供临时的Git仓库"""
    with tempfile.TemporaryDirectory() as temp_dir:
        repo_path = Path(temp_dir)

        # 初始化Git仓库
        subprocess.run(["git", "init"], cwd=repo_path, check=True)
        subprocess.run(["git", "config", "user.name", "Test User"], cwd=repo_path, check=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=repo_path, check=True)

        yield repo_path


@pytest.fixture
def sample_rust_file(temp_git_repo):
    """创建一个示例Rust文件"""
    file_path = temp_git_repo / "src" / "main.rs"
    file_path.parent.mkdir(parents=True, exist_ok=True)

    rust_code = '''
fn main() {
    println!("Hello, GitAI!");

    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();

    println!("Sum: {}", sum);

    if sum > 10 {
        println!("Large sum!");
    }
}
'''

    file_path.write_text(rust_code)

    # 添加到Git
    subprocess.run(["git", "add", "."], cwd=temp_git_repo, check=True)
    subprocess.run(["git", "commit", "-m", "Initial commit"], cwd=temp_git_repo, check=True)

    return file_path


@pytest.fixture
def sample_python_file(temp_git_repo):
    """创建一个示例Python文件"""
    file_path = temp_git_repo / "app.py"

    python_code = '''
import os
import sys

def calculate_fibonacci(n):
    """Calculate the nth Fibonacci number."""
    if n <= 1:
        return n
    return calculate_fibonacci(n-1) + calculate_fibonacci(n-2)

def main():
    n = 10
    result = calculate_fibonacci(n)
    print(f"Fibonacci({n}) = {result}")

if __name__ == "__main__":
    main()
'''

    file_path.write_text(python_code)
    return file_path


# 测试配置
TEST_CONFIG = {
    "timeout": {
        "short": 10,
        "medium": 30,
        "long": 60
    },
    "mcp_port": 8711,
    "test_file_size": {
        "small": 100,      # 100行
        "medium": 1000,    # 1K行
        "large": 10000     # 10K行
    }
}


def pytest_configure(config):
    """pytest配置"""
    # 添加自定义标记
    config.addinivalue_line(
        "markers", "slow: marks tests as slow (deselect with '-m \"not slow\"')"
    )
    config.addinivalue_line(
        "markers", "mcp: marks tests that require MCP server"
    )
    config.addinivalue_line(
        "markers", "integration: marks tests as integration tests"
    )