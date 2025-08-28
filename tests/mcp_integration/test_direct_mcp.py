#!/usr/bin/env python3
import json
import subprocess
import time
import sys
import os

# 可以通过环境变量设置等待时间
SERVER_STARTUP_WAIT = float(os.environ.get('MCP_SERVER_STARTUP_WAIT', '2.0'))
SERVER_SHUTDOWN_WAIT = float(os.environ.get('MCP_SERVER_SHUTDOWN_WAIT', '1.0'))

def test_direct_mcp():
    """直接测试 MCP 服务器"""
    print("🧪 直接测试 MCP 服务器...")
    
    # 清理旧进程
    subprocess.run(["pkill", "-f", "gitai mcp"], capture_output=True, shell=False)
    # 等待旧进程完全终止
    time.sleep(SERVER_SHUTDOWN_WAIT)
    
    # 启动 MCP 服务器
    env = os.environ.copy()
    env["RUST_LOG"] = "debug"
    
    print("🚀 启动 MCP 服务器...")
    mcp_process = subprocess.Popen(
        ["./target/release/gitai", "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )
    
    # 等待服务器启动，可通过环境变量 MCP_SERVER_STARTUP_WAIT 调整
    time.sleep(SERVER_STARTUP_WAIT)
    
    try:
        # 发送初始化请求
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        print("📤 发送初始化请求...")
        mcp_process.stdin.write(json.dumps(init_request) + "\n")
        mcp_process.stdin.flush()
        
        # 读取初始化响应
        init_response = mcp_process.stdout.readline()
        print(f"📥 初始化响应: {init_response}")
        
        # 发送扫描请求
        scan_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "execute_scan",
                "arguments": {
                    "path": "/Users/huchen/Projects/java-sec-code",
                    "tool": "opengrep",
                    "timeout": 60
                }
            }
        }
        
        print("📤 发送扫描请求...")
        mcp_process.stdin.write(json.dumps(scan_request) + "\n")
        mcp_process.stdin.flush()
        
        # 读取扫描响应
        scan_response = mcp_process.stdout.readline()
        print(f"📥 扫描响应: {scan_response}")
        
        # 读取错误输出
        print("📋 读取错误输出...")
        stderr_output = []
        for _ in range(20):  # 读取最多20行错误输出
            line = mcp_process.stderr.readline()
            if line:
                stderr_output.append(line.strip())
                print(f"  STDERR: {line.strip()}")
            else:
                time.sleep(0.1)  # 短暂等待以避免CPU过载
                break
        
        # 解析扫描响应
        if scan_response:
            try:
                response_data = json.loads(scan_response)
                if "result" in response_data:
                    result = response_data["result"]
                    print(f"✅ 扫描成功!")
                    print(f"   - 成功: {result.get('success', 'unknown')}")
                    print(f"   - 消息: {result.get('message', 'no message')}")
                    print(f"   - 发现问题: {result.get('summary', {}).get('total_findings', 0)}")
                else:
                    print(f"❌ 响应中没有 result 字段")
            except json.JSONDecodeError as e:
                print(f"❌ JSON 解析失败: {e}")
        else:
            print("❌ 没有收到扫描响应")
        
    except Exception as e:
        print(f"❌ 测试过程中出错: {e}")
        import traceback
        traceback.print_exc()
        
    finally:
        print("🧹 清理进程...")
        mcp_process.terminate()
        try:
            mcp_process.wait(timeout=5)
        except:
            mcp_process.kill()

if __name__ == "__main__":
    test_direct_mcp()
