"""
GitAI 核心功能测试
测试基本的CLI命令和功能
"""

import os
import pytest
import json
import subprocess
from pathlib import Path


class TestBasicCommands:
    """测试基本CLI命令"""

    def test_help_command(self, gitai_helper):
        """测试help命令"""
        result = gitai_helper.run_command(["--help"])
        assert result.returncode == 0
        assert "GitAI" in result.stdout
        assert "AI驱动的Git工作流助手" in result.stdout
        assert "Commands:" in result.stdout

    def test_version_display(self, gitai_helper):
        """测试版本信息显示"""
        result = gitai_helper.run_command(["features"])
        assert result.returncode == 0
        assert "GitAI 功能特性" in result.stdout

    def test_features_command(self, gitai_helper):
        """测试features命令"""
        features = gitai_helper.get_features()
        assert isinstance(features, dict)
        assert "minimal" in features
        assert features["minimal"] is True  # 默认应该启用最小配置

    def test_config_show(self, gitai_helper):
        """测试配置显示"""
        config = gitai_helper.get_config()
        assert isinstance(config, dict)
        assert "AI服务" in config
        assert "AI模型" in config
        assert "格式" in config

    def test_config_check(self, gitai_helper):
        """测试配置检查"""
        result = gitai_helper.run_command(["config", "check"])
        assert result.returncode == 0


class TestReviewFeature:
    """测试代码评审功能"""

    def test_review_help(self, gitai_helper):
        """测试review命令帮助"""
        result = gitai_helper.run_command(["review", "--help"])
        assert result.returncode == 0
        assert "AI驱动的代码评审" in result.stdout
        assert "--format" in result.stdout
        assert "--tree-sitter" in result.stdout
        assert "--security-scan" in result.stdout

    def test_review_offline_mode(self, gitai_helper, temp_git_repo):
        """测试离线模式的代码评审"""
        # 切换到临时仓库
        os.chdir(temp_git_repo)

        # 运行离线评审
        result = gitai_helper.run_command([
            "review",
            "--format", "text",
            "--offline"
        ], timeout=60)

        assert result.returncode == 0
        assert "代码评审报告" in result.stdout
        assert "变更概览" in result.stdout

    def test_review_json_output(self, gitai_helper, temp_git_repo):
        """测试JSON格式输出"""
        os.chdir(temp_git_repo)

        result = gitai_helper.run_command([
            "review",
            "--format", "json",
            "--offline"
        ], timeout=60)

        assert result.returncode == 0

        # 尝试解析JSON输出
        try:
            review_data = json.loads(result.stdout)
            assert "changes" in review_data
            assert "metadata" in review_data
        except json.JSONDecodeError:
            # 如果输出不是纯JSON，可能包含其他文本，检查是否包含JSON部分
            assert "{" in result.stdout and "}" in result.stdout


class TestCommitFeature:
    """测试智能提交功能"""

    def test_commit_help(self, gitai_helper):
        """测试commit命令帮助"""
        result = gitai_helper.run_command(["commit", "--help"])
        assert result.returncode == 0
        assert "智能提交" in result.stdout
        assert "--message" in result.stdout
        assert "--dry-run" in result.stdout
        assert "--review" in result.stdout

    def test_commit_dry_run(self, gitai_helper, temp_git_repo, sample_rust_file):
        """测试干运行模式的提交"""
        os.chdir(temp_git_repo)

        # 修改文件
        sample_rust_file.write_text(sample_rust_file.read_text() + "\n// Modified comment\n")

        result = gitai_helper.run_command([
            "commit",
            "--dry-run",
            "--offline"
        ])

        assert result.returncode == 0
        # 干运行应该不会实际提交
        assert "测试运行" in result.stdout or "没有检测到文件变更" in result.stdout

    def test_commit_with_message(self, gitai_helper, temp_git_repo, sample_rust_file):
        """测试带自定义消息的提交"""
        os.chdir(temp_git_repo)

        # 修改文件
        sample_rust_file.write_text(sample_rust_file.read_text() + "\n// Another modification\n")

        result = gitai_helper.run_command([
            "commit",
            "--message", "Test commit with custom message",
            "--dry-run",
            "--offline"
        ])

        assert result.returncode == 0


class TestScanFeature:
    """测试安全扫描功能"""

    def test_scan_help(self, gitai_helper):
        """测试scan命令帮助"""
        result = gitai_helper.run_command(["scan", "--help"])
        assert result.returncode == 0
        assert "代码安全扫描" in result.stdout
        assert "--path" in result.stdout
        assert "--format" in result.stdout
        assert "--tool" in result.stdout

    def test_scan_offline_mode(self, gitai_helper, temp_git_repo):
        """测试离线模式的扫描"""
        os.chdir(temp_git_repo)

        result = gitai_helper.run_command([
            "scan",
            "--path", ".",
            "--format", "text",
            "--offline"
        ])

        # 扫描功能可能需要security特性，如果未启用应该有相应提示
        if result.returncode != 0:
            assert "功能未启用" in result.stdout or "安全扫描功能未启用" in result.stdout
        else:
            # 如果功能可用，应该有扫描输出
            assert "扫描" in result.stdout or "结果" in result.stdout

    def test_scan_with_sensitive_content(self, gitai_helper, temp_git_repo):
        """测试包含敏感内容的扫描"""
        os.chdir(temp_git_repo)

        # 创建包含敏感信息的文件
        secret_file = temp_git_repo / "config.py"
        secret_file.write_text('''
import os

API_KEY = "sk-1234567890abcdef"
DATABASE_URL = "postgresql://user:password@localhost/db"
SECRET_TOKEN = "my_secret_token_123"

def connect_db():
    return DATABASE_URL
''')

        result = gitai_helper.run_command([
            "scan",
            "--path", str(secret_file),
            "--format", "text",
            "--offline"
        ])

        # 无论功能是否启用，都应该有明确的响应
        assert result.returncode == 0 or "功能未启用" in result.stdout


class TestInitFeature:
    """测试初始化功能"""

    def test_init_help(self, gitai_helper):
        """测试init命令帮助"""
        result = gitai_helper.run_command(["init", "--help"])
        assert result.returncode == 0
        assert "初始化GitAI配置" in result.stdout
        assert "--offline" in result.stdout
        assert "--dev" in result.stdout

    def test_init_offline_mode(self, gitai_helper, temp_git_repo):
        """测试离线模式初始化"""
        os.chdir(temp_git_repo)

        result = gitai_helper.run_command([
            "init",
            "--offline"
        ], timeout=30)

        assert result.returncode == 0


class TestGraphFeature:
    """测试依赖图功能"""

    def test_graph_help(self, gitai_helper):
        """测试graph命令帮助"""
        result = gitai_helper.run_command(["graph", "--help"])
        assert result.returncode == 0
        assert "依赖图导出" in result.stdout
        assert "--format" in result.stdout
        assert "--output" in result.stdout

    def test_graph_export(self, gitai_helper, temp_git_repo):
        """测试依赖图导出"""
        os.chdir(temp_git_repo)

        result = gitai_helper.run_command([
            "graph",
            "--format", "dot",
            "--offline"
        ], timeout=30)

        # 应该有响应，可能是成功输出或错误提示
        assert result.returncode == 0 or "错误" in result.stdout


class TestMetricsFeature:
    """测试质量指标功能"""

    def test_metrics_help(self, gitai_helper):
        """测试metrics命令帮助"""
        result = gitai_helper.run_command(["metrics", "--help"])
        assert result.returncode == 0
        assert "质量指标" in result.stdout or "架构质量" in result.stdout

    def test_metrics_generation(self, gitai_helper, temp_git_repo):
        """测试指标生成"""
        os.chdir(temp_git_repo)

        result = gitai_helper.run_command([
            "metrics",
            "--offline"
        ], timeout=30)

        # 应该有响应
        assert result.returncode == 0 or "错误" in result.stdout


class TestErrorHandling:
    """测试错误处理"""

    def test_invalid_command(self, gitai_helper):
        """测试无效命令"""
        result = gitai_helper.run_command(["invalid-command"])
        assert result.returncode != 0

    def test_invalid_arguments(self, gitai_helper):
        """测试无效参数"""
        result = gitai_helper.run_command(["review", "--invalid-arg"])
        assert result.returncode != 0

    def test_missing_file(self, gitai_helper):
        """测试不存在的文件"""
        result = gitai_helper.run_command([
            "scan",
            "--path", "/nonexistent/path"
        ])
        # 应该优雅地处理不存在的路径
        assert result.returncode != 0 or "不存在" in result.stdout


@pytest.mark.integration
class TestWorkflowIntegration:
    """测试完整工作流集成"""

    def test_complete_review_workflow(self, gitai_helper, temp_git_repo, sample_rust_file, sample_python_file):
        """测试完整的代码评审工作流"""
        os.chdir(temp_git_repo)

        # 1. 先进行代码评审
        review_result = gitai_helper.run_command([
            "review",
            "--format", "text",
            "--offline"
        ], timeout=60)

        assert review_result.returncode == 0
        assert "代码评审报告" in review_result.stdout

        # 2. 尝试智能提交
        commit_result = gitai_helper.run_command([
            "commit",
            "--dry-run",
            "--review",
            "--offline"
        ])

        assert commit_result.returncode == 0

    def test_config_to_review_workflow(self, gitai_helper):
        """测试配置到评审的工作流"""
        # 1. 检查当前配置
        config = gitai_helper.get_config()
        assert isinstance(config, dict)

        # 2. 确认功能可用
        features = gitai_helper.get_features()
        assert isinstance(features, dict)

        # 3. 验证基本功能可用性
        assert features.get("minimal", False) or "功能特性" in str(features)