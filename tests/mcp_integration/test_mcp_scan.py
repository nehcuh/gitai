#!/usr/bin/env python3
import json
import subprocess
import time
import os

def test_mcp_scan():
    """测试 MCP 扫描功能"""
    print("🔍 测试 MCP 扫描功能...")
    
    # 确保 MCP 服务器在后台运行
    # 先停止可能存在的服务器
    subprocess.run(["pkill", "-f", "gitai mcp"], capture_output=True)
    time.sleep(1)
    
    # 启动 MCP 服务器
    print("🚀 启动 MCP 服务器...")
    mcp_process = subprocess.Popen(
        ["./target/release/gitai", "mcp"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # 等待服务器启动
    time.sleep(2)
    
    try:
        # 创建一个简单的 MCP 请求
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "execute_scan",
                "arguments": {
                    "path": "/Users/huchen/Projects/gitai/security_test.java",
                    "tool": "opengrep",
                    "timeout": 30
                }
            }
        }
        
        print("📤 发送 MCP 请求...")
        print(f"请求内容: {json.dumps(request, indent=2, ensure_ascii=False)}")
        
        # 发送请求
        result = subprocess.run(
            ["echo", json.dumps(request)],
            stdout=subprocess.PIPE,
            text=True
        )
        
        print(f"📥 MCP 响应: {result.stdout}")
        
    except Exception as e:
        print(f"❌ 测试失败: {e}")
    finally:
        # 清理
        print("🧹 清理进程...")
        mcp_process.terminate()
        mcp_process.wait()

if __name__ == "__main__":
    test_mcp_scan()