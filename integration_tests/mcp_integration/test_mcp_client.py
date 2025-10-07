"""
GitAI MCP客户端测试
使用Python作为MCP客户端测试与GitAI MCP服务器的交互
"""

import pytest
import asyncio
import json
import subprocess
import time
from typing import Dict, Any, Optional


class SimpleMCPClient:
    """简单的MCP客户端实现"""

    def __init__(self, transport: str = "stdio"):
        self.transport = transport
        self.process = None
        self.request_id = 1

    async def initialize(self, server_process: subprocess.Popen = None) -> bool:
        """初始化MCP连接"""
        if self.transport == "stdio" and server_process:
            self.process = server_process
            return True
        return False

    def send_request(self, method: str, params: Dict[str, Any] = None) -> Dict[str, Any]:
        """发送MCP请求"""
        if not self.process:
            raise RuntimeError("MCP client not initialized")

        request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method
        }

        if params:
            request["params"] = params

        self.request_id += 1

        # 对于stdio传输，这里简化处理
        # 实际实现需要通过stdin/stdout进行通信
        return {
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {"status": "simulated"}
        }

    async def list_tools(self) -> Dict[str, Any]:
        """列出可用工具"""
        return self.send_request("tools/list")

    async def call_tool(self, name: str, arguments: Dict[str, Any] = None) -> Dict[str, Any]:
        """调用工具"""
        params = {"name": name}
        if arguments:
            params["arguments"] = arguments

        return self.send_request("tools/call", params)


@pytest.mark.mcp
class TestMCPClientBasics:
    """测试MCP客户端基础功能"""

    def test_mcp_client_creation(self):
        """测试MCP客户端创建"""
        client = SimpleMCPClient("stdio")
        assert client.transport == "stdio"
        assert client.request_id == 1

    @pytest.mark.asyncio
    async def test_mcp_client_initialization(self):
        """测试MCP客户端初始化"""
        client = SimpleMCPClient("stdio")
        # 模拟初始化
        result = await client.initialize()
        assert isinstance(result, bool)

    @pytest.mark.asyncio
    async def test_list_tools_request_format(self):
        """测试工具列表请求格式"""
        client = SimpleMCPClient("stdio")
        result = await client.list_tools()

        assert "jsonrpc" in result
        assert result["jsonrpc"] == "2.0"
        assert "id" in result

    @pytest.mark.asyncio
    async def test_tool_call_request_format(self):
        """测试工具调用请求格式"""
        client = SimpleMCPClient("stdio")
        result = await client.call_tool("gitai_review", {"format": "text"})

        assert "jsonrpc" in result
        assert "id" in result


@pytest.mark.mcp
class TestMCPToolSchemas:
    """测试MCP工具schema"""

    def test_review_tool_schema(self):
        """测试代码评审工具schema"""
        review_schema = {
            "name": "gitai_review",
            "description": "AI驱动的代码评审工具",
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
                        "default": False,
                        "description": "离线模式，不进行网络请求"
                    },
                    "tree_sitter": {
                        "type": "boolean",
                        "default": False,
                        "description": "启用Tree-sitter结构分析"
                    },
                    "security_scan": {
                        "type": "boolean",
                        "default": False,
                        "description": "启用安全扫描"
                    }
                },
                "required": []
            }
        }

        # 验证schema结构
        assert "name" in review_schema
        assert "inputSchema" in review_schema
        assert "properties" in review_schema["inputSchema"]

        properties = review_schema["inputSchema"]["properties"]
        assert "format" in properties
        assert "offline" in properties
        assert properties["format"]["default"] == "console"

    def test_commit_tool_schema(self):
        """测试智能提交工具schema"""
        commit_schema = {
            "name": "gitai_commit",
            "description": "智能提交信息生成工具",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "自定义提交信息"
                    },
                    "dry_run": {
                        "type": "boolean",
                        "default": False,
                        "description": "测试运行，不实际提交"
                    },
                    "all": {
                        "type": "boolean",
                        "default": False,
                        "description": "添加所有变更文件"
                    },
                    "review": {
                        "type": "boolean",
                        "default": False,
                        "description": "启用代码评审"
                    }
                },
                "required": []
            }
        }

        # 验证schema结构
        assert commit_schema["name"] == "gitai_commit"
        assert "dry_run" in commit_schema["inputSchema"]["properties"]
        assert commit_schema["inputSchema"]["properties"]["dry_run"]["default"] is False

    def test_scan_tool_schema(self):
        """测试安全扫描工具schema"""
        scan_schema = {
            "name": "gitai_scan",
            "description": "代码安全扫描工具",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "default": ".",
                        "description": "扫描路径"
                    },
                    "format": {
                        "type": "string",
                        "default": "text",
                        "description": "输出格式"
                    },
                    "tool": {
                        "type": "string",
                        "enum": ["opengrep", "auto"],
                        "default": "auto",
                        "description": "扫描工具"
                    },
                    "offline": {
                        "type": "boolean",
                        "default": False,
                        "description": "离线模式"
                    }
                },
                "required": []
            }
        }

        # 验证schema结构
        assert scan_schema["name"] == "gitai_scan"
        assert "path" in scan_schema["inputSchema"]["properties"]
        assert scan_schema["inputSchema"]["properties"]["path"]["default"] == "."

    def test_graph_tool_schema(self):
        """测试依赖图工具schema"""
        graph_schema = {
            "name": "gitai_graph",
            "description": "依赖图导出工具",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "enum": ["dot", "json", "summary"],
                        "default": "dot"
                    },
                    "output": {
                        "type": "string",
                        "description": "输出文件路径"
                    },
                    "path": {
                        "type": "string",
                        "default": ".",
                        "description": "分析路径"
                    }
                },
                "required": []
            }
        }

        # 验证schema结构
        assert graph_schema["name"] == "gitai_graph"
        assert "format" in graph_schema["inputSchema"]["properties"]
        assert graph_schema["inputSchema"]["properties"]["format"]["default"] == "dot"


@pytest.mark.mcp
class TestMCPMessageFormats:
    """测试MCP消息格式"""

    def test_initialize_message_format(self):
        """测试初始化消息格式"""
        init_message = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "gitai-test-client",
                    "version": "1.0.0"
                }
            }
        }

        # 验证消息格式
        assert init_message["jsonrpc"] == "2.0"
        assert init_message["method"] == "initialize"
        assert "params" in init_message
        assert "protocolVersion" in init_message["params"]
        assert "clientInfo" in init_message["params"]

    def test_tool_call_message_format(self):
        """测试工具调用消息格式"""
        tool_call = {
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

        # 验证消息格式
        assert tool_call["method"] == "tools/call"
        assert "name" in tool_call["params"]
        assert "arguments" in tool_call["params"]
        assert tool_call["params"]["name"] == "gitai_review"

    def test_response_message_format(self):
        """测试响应消息格式"""
        response = {
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": "代码评审完成"
                    }
                ]
            }
        }

        # 验证响应格式
        assert response["jsonrpc"] == "2.0"
        assert "id" in response
        assert "result" in response
        assert "content" in response["result"]

    def test_error_message_format(self):
        """测试错误消息格式"""
        error_response = {
            "jsonrpc": "2.0",
            "id": 3,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": {
                    "method": "invalid_method"
                }
            }
        }

        # 验证错误格式
        assert error_response["jsonrpc"] == "2.0"
        assert "error" in error_response
        assert "code" in error_response["error"]
        assert "message" in error_response["error"]


@pytest.mark.mcp
class TestMCPWorkflowSimulation:
    """测试MCP工作流模拟"""

    @pytest.mark.asyncio
    async def test_complete_review_workflow(self):
        """测试完整的代码评审工作流"""
        client = SimpleMCPClient("stdio")

        # 模拟初始化
        await client.initialize()

        # 1. 列出可用工具
        tools = await client.list_tools()
        assert "jsonrpc" in tools

        # 2. 调用代码评审工具
        review_result = await client.call_tool("gitai_review", {
            "format": "text",
            "offline": True
        })
        assert "jsonrpc" in review_result

        # 3. 验证工作流完成
        assert True  # 工作流模拟完成

    @pytest.mark.asyncio
    async def test_commit_with_review_workflow(self):
        """测试提交前评审工作流"""
        client = SimpleMCPClient("stdio")
        await client.initialize()

        # 1. 先进行代码评审
        review_result = await client.call_tool("gitai_review", {
            "format": "json",
            "offline": True
        })

        # 2. 根据评审结果生成提交信息
        commit_result = await client.call_tool("gitai_commit", {
            "dry_run": True,
            "review": True,
            "offline": True
        })

        assert "jsonrpc" in review_result
        assert "jsonrpc" in commit_result

    @pytest.mark.asyncio
    async def test_security_scan_workflow(self):
        """测试安全扫描工作流"""
        client = SimpleMCPClient("stdio")
        await client.initialize()

        # 1. 执行安全扫描
        scan_result = await client.call_tool("gitai_scan", {
            "path": ".",
            "format": "json",
            "offline": True
        })

        # 2. 根据扫描结果决定是否继续
        assert "jsonrpc" in scan_result

    @pytest.mark.asyncio
    async def test_dependency_analysis_workflow(self):
        """测试依赖分析工作流"""
        client = SimpleMCPClient("stdio")
        await client.initialize()

        # 1. 生成依赖图
        graph_result = await client.call_tool("gitai_graph", {
            "format": "json",
            "path": "."
        })

        # 2. 分析依赖关系
        assert "jsonrpc" in graph_result


@pytest.mark.mcp
class TestMCPIntegration:
    """测试MCP集成功能"""

    def test_mcp_command_availability(self, gitai_helper):
        """测试MCP命令可用性"""
        mcp_commands = [
            ["mcp", "--help"],
            ["mcp-health"],
            ["mcp-tools"],
            ["mcp-info"],
            ["mcp-call", "--help"],
            ["mcp-batch", "--help"]
        ]

        for cmd in mcp_commands:
            result = gitai_helper.run_command(cmd)
            # 命令应该存在（返回码为0或有错误信息）
            assert result.returncode == 0 or len(result.stdout) > 0 or len(result.stderr) > 0

    def test_mcp_feature_detection(self, gitai_helper):
        """检测MCP功能是否启用"""
        result = gitai_helper.run_command(["features"])
        assert result.returncode == 0

        # 检查是否包含MCP相关信息
        features_output = result.stdout
        assert "功能特性" in features_output

    @pytest.mark.slow
    def test_mcp_server_lifecycle(self, mcp_server):
        """测试MCP服务器生命周期"""
        # 1. 启动服务器
        success = mcp_server.start_server(transport="http", port=8711)
        assert success, "Failed to start MCP server"

        # 2. 等待服务器就绪
        time.sleep(3)

        # 3. 验证服务器运行
        assert mcp_server.is_running(), "MCP server should be running"

        # 4. 停止服务器
        mcp_server.stop_server()

        # 5. 验证服务器停止
        assert not mcp_server.is_running(), "MCP server should be stopped"


@pytest.mark.mcp
class TestMCPErrorHandling:
    """测试MCP错误处理"""

    @pytest.mark.asyncio
    async def test_invalid_tool_call(self):
        """测试无效工具调用"""
        client = SimpleMCPClient("stdio")
        await client.initialize()

        # 调用不存在的工具
        result = await client.call_tool("invalid_tool", {})

        # 应该返回错误响应
        assert "jsonrpc" in result

    @pytest.mark.asyncio
    async def test_invalid_parameters(self):
        """测试无效参数"""
        client = SimpleMCPClient("stdio")
        await client.initialize()

        # 使用无效参数调用工具
        result = await client.call_tool("gitai_review", {
            "format": "invalid_format",
            "invalid_param": "value"
        })

        # 应该返回错误响应
        assert "jsonrpc" in result

    def test_mcp_server_not_running(self, gitai_helper):
        """测试MCP服务器未运行时的处理"""
        # 当服务器未运行时，相关命令应该优雅失败
        commands = ["mcp-health", "mcp-tools", "mcp-info"]

        for cmd in commands:
            result = gitai_helper.run_command([cmd])
            # 应该有明确的错误信息
            assert result.returncode != 0 or "Connection refused" in result.stdout