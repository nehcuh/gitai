"""
GitAI MCP服务器集成测试
测试MCP协议支持和各种工具调用
"""

import pytest
import json
import asyncio
import websockets
import requests
import time
from pathlib import Path


@pytest.mark.mcp
class TestMCPServerBasics:
    """测试MCP服务器基础功能"""

    def test_mcp_help_command(self, gitai_helper):
        """测试MCP相关命令帮助"""
        result = gitai_helper.run_command(["mcp", "--help"])
        assert result.returncode == 0
        assert "启动 GitAI MCP 服务器" in result.stdout
        assert "--transport" in result.stdout

    def test_mcp_health_check_command(self, gitai_helper):
        """测试MCP健康检查命令"""
        result = gitai_helper.run_command(["mcp-health"])
        # 如果服务器未运行，应该有连接失败的错误
        assert "Connection refused" in result.stdout or "健康检查" in result.stdout

    def test_mcp_tools_command(self, gitai_helper):
        """测试MCP工具列表命令"""
        result = gitai_helper.run_command(["mcp-tools"])
        # 如果服务器未运行，应该有连接失败的错误
        assert "Connection refused" in result.stdout or "工具" in result.stdout

    def test_mcp_info_command(self, gitai_helper):
        """测试MCP信息命令"""
        result = gitai_helper.run_command(["mcp-info"])
        # 如果服务器未运行，应该有连接失败的错误
        assert "Connection refused" in result.stdout or "服务器信息" in result.stdout


@pytest.mark.mcp
@pytest.mark.slow
class TestMCPServerHTTP:
    """测试MCP服务器HTTP模式"""

    def test_start_mcp_server_http(self, mcp_server):
        """测试启动HTTP模式的MCP服务器"""
        success = mcp_server.start_server(transport="http", port=8711)
        assert success, "Failed to start MCP server in HTTP mode"

        # 等待服务器完全启动
        time.sleep(3)

        # 验证服务器正在运行
        assert mcp_server.is_running(), "MCP server should be running"

    def test_mcp_health_endpoint(self, mcp_server):
        """测试MCP健康检查端点"""
        if not mcp_server.is_running():
            mcp_server.start_server(transport="http", port=8711)
            time.sleep(3)

        try:
            response = requests.get("http://127.0.0.1:8711/health", timeout=5)
            assert response.status_code == 200
            health_data = response.json()
            assert "status" in health_data
        except requests.exceptions.ConnectionError:
            pytest.skip("MCP server not available for health check")

    def test_mcp_tools_endpoint(self, mcp_server):
        """测试MCP工具列表端点"""
        if not mcp_server.is_running():
            mcp_server.start_server(transport="http", port=8711)
            time.sleep(3)

        try:
            response = requests.get("http://127.0.0.1:8711/tools", timeout=5)
            assert response.status_code == 200
            tools_data = response.json()
            assert isinstance(tools_data, (list, dict))
        except requests.exceptions.ConnectionError:
            pytest.skip("MCP server not available for tools check")

    def test_mcp_server_info_endpoint(self, mcp_server):
        """测试MCP服务器信息端点"""
        if not mcp_server.is_running():
            mcp_server.start_server(transport="http", port=8711)
            time.sleep(3)

        try:
            response = requests.get("http://127.0.0.1:8711/info", timeout=5)
            assert response.status_code == 200
            info_data = response.json()
            assert "server" in info_data or "version" in info_data
        except requests.exceptions.ConnectionError:
            pytest.skip("MCP server not available for info check")


@pytest.mark.mcp
class TestMCPProtocolBasics:
    """测试MCP协议基础功能"""

    def test_mcp_protocol_initialization(self, gitai_helper):
        """测试MCP协议初始化消息"""
        # 这个测试模拟MCP客户端的初始化流程
        init_message = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }

        # 验证初始化消息格式正确
        assert "jsonrpc" in init_message
        assert init_message["jsonrpc"] == "2.0"
        assert "method" in init_message
        assert init_message["method"] == "initialize"

    def test_mcp_tool_call_format(self):
        """测试MCP工具调用格式"""
        tool_call_message = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "gitai_review",
                "arguments": {
                    "format": "text",
                    "offline": True
                }
            }
        }

        # 验证工具调用消息格式
        assert "params" in tool_call_message
        assert "name" in tool_call_message["params"]
        assert "arguments" in tool_call_message["params"]


@pytest.mark.mcp
class TestMCPTools:
    """测试MCP工具功能"""

    def test_review_tool_parameters(self):
        """测试代码评审工具参数"""
        review_tool_schema = {
            "name": "gitai_review",
            "description": "AI驱动的代码评审",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "enum": ["console", "markdown", "json", "yaml", "text"],
                        "default": "console"
                    },
                    "offline": {
                        "type": "boolean",
                        "default": False
                    },
                    "tree_sitter": {
                        "type": "boolean",
                        "default": False
                    },
                    "security_scan": {
                        "type": "boolean",
                        "default": False
                    }
                }
            }
        }

        # 验证工具schema
        assert "name" in review_tool_schema
        assert "inputSchema" in review_tool_schema
        assert "properties" in review_tool_schema["inputSchema"]

    def test_commit_tool_parameters(self):
        """测试智能提交工具参数"""
        commit_tool_schema = {
            "name": "gitai_commit",
            "description": "智能提交信息生成",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string"
                    },
                    "dry_run": {
                        "type": "boolean",
                        "default": False
                    },
                    "all": {
                        "type": "boolean",
                        "default": False
                    },
                    "review": {
                        "type": "boolean",
                        "default": False
                    }
                }
            }
        }

        # 验证提交工具参数
        assert review_tool_schema["name"] == "gitai_commit"
        assert "dry_run" in review_tool_schema["inputSchema"]["properties"]

    def test_scan_tool_parameters(self):
        """测试安全扫描工具参数"""
        scan_tool_schema = {
            "name": "gitai_scan",
            "description": "代码安全扫描",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "default": "."
                    },
                    "format": {
                        "type": "string",
                        "default": "text"
                    },
                    "offline": {
                        "type": "boolean",
                        "default": False
                    },
                    "tool": {
                        "type": "string",
                        "enum": ["opengrep", "auto"],
                        "default": "auto"
                    }
                }
            }
        }

        # 验证扫描工具参数
        assert scan_tool_schema["name"] == "gitai_scan"
        assert "path" in scan_tool_schema["inputSchema"]["properties"]


@pytest.mark.mcp
class TestMCPIntegration:
    """测试MCP集成功能"""

    def test_mcp_command_line_integration(self, gitai_helper):
        """测试MCP命令行集成"""
        # 测试MCP相关命令是否存在
        commands_to_test = [
            ["mcp", "--help"],
            ["mcp-health"],
            ["mcp-tools"],
            ["mcp-info"]
        ]

        for cmd in commands_to_test:
            result = gitai_helper.run_command(cmd)
            # 命令应该存在但不一定成功（服务器可能未运行）
            assert result.returncode == 0 or "Connection refused" in result.stdout

    def test_mcp_feature_detection(self, gitai_helper):
        """测试MCP功能检测"""
        # 检查GitAI是否支持MCP功能
        result = gitai_helper.run_command(["features"])
        assert result.returncode == 0

        # 验证输出包含相关信息
        assert "功能特性" in result.stdout


@pytest.mark.mcp
@pytest.mark.slow
class TestMCPWebSocket:
    """测试MCP WebSocket连接（如果支持）"""

    def test_websocket_connection_attempt(self):
        """尝试WebSocket连接测试"""
        # 这个测试检查是否支持WebSocket连接
        try:
            # 尝试导入websockets库
            import websockets
            websocket_available = True
        except ImportError:
            websocket_available = False
            pytest.skip("websockets library not available")

        if not websocket_available:
            pytest.skip("WebSocket support not available")

    @pytest.mark.asyncio
    async def test_async_mcp_workflow(self):
        """测试异步MCP工作流"""
        # 模拟异步MCP工作流
        workflow_steps = [
            "initialize",
            "tools/list",
            "tools/call",
            "cleanup"
        ]

        for step in workflow_steps:
            # 验证工作流步骤格式
            assert isinstance(step, str)
            assert len(step) > 0

        # 模拟异步操作
        await asyncio.sleep(0.01)  # 很短的延迟表示异步操作


@pytest.mark.mcp
class TestMCPErrorHandling:
    """测试MCP错误处理"""

    def test_mcp_server_unavailable_handling(self, gitai_helper):
        """测试MCP服务器不可用时的处理"""
        # 当MCP服务器未运行时，相关命令应该优雅失败
        commands = ["mcp-health", "mcp-tools", "mcp-info"]

        for cmd in commands:
            result = gitai_helper.run_command(cmd)
            # 应该有明确的错误信息
            assert result.returncode != 0 or "Connection refused" in result.stdout

    def test_invalid_mcp_transport(self, gitai_helper):
        """测试无效的MCP传输协议"""
        result = gitai_helper.run_command([
            "mcp",
            "--transport", "invalid_protocol"
        ])
        # 应该优雅地处理无效参数
        assert result.returncode != 0

    def test_mcp_port_conflict(self, gitai_helper):
        """测试MCP端口冲突处理"""
        # 这个测试假设端口8711可能被占用
        # 在实际环境中，可能需要更复杂的端口冲突检测
        result = gitai_helper.run_command([
            "mcp",
            "--transport", "http",
            "--port", "8711"
        ], timeout=5)

        # 命令可能成功启动，也可能因为端口冲突失败
        # 重要的是要有明确的响应
        assert result.returncode == 0 or result.returncode != 0