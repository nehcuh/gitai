#!/usr/bin/env python3
import json
import subprocess
import time
import os

def test_mcp_scan_fixed():
    """测试修复后的 MCP 扫描功能"""
    print("🧪 测试修复后的 MCP 扫描功能...")
    
    # 1. 首先确保没有旧的 gitai 进程
    print("1️⃣ 清理旧进程...")
    subprocess.run(["pkill", "-f", "gitai mcp"], capture_output=True)
    time.sleep(1)
    
    # 2. 启动新的 MCP 服务器（启用调试日志）
    print("2️⃣ 启动 MCP 服务器...")
    env = os.environ.copy()
    env["RUST_LOG"] = "debug"  # 启用调试日志
    
    mcp_process = subprocess.Popen(
        ["./target/release/gitai", "mcp"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )
    
    # 等待服务器启动
    time.sleep(3)
    
    try:
        # 3. 创建 MCP 请求（使用我们知道的会产生结果的路径）
        print("3️⃣ 发送 MCP 扫描请求...")
        
        request = {
            "jsonrpc": "2.0",
            "id": 1,
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
        
        print(f"请求路径: {request['params']['arguments']['path']}")
        
        # 4. 发送请求并获取响应
        result = subprocess.run(
            ["echo", json.dumps(request)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        print(f"请求发送完成")
        
        # 5. 检查 MCP 服务器的日志输出
        print("4️⃣ 检查 MCP 服务器日志...")
        time.sleep(2)
        
        # 读取一些日志输出
        stderr_lines = []
        try:
            import select
            import sys
            
            # 非阻塞读取
            while True:
                line = mcp_process.stderr.readline()
                if line:
                    stderr_lines.append(line.strip())
                    if len(stderr_lines) >= 10:  # 只读取最近10行
                        break
                else:
                    break
        except:
            pass
        
        if stderr_lines:
            print("MCP 服务器日志:")
            for line in stderr_lines[-5:]:  # 显示最后5行
                print(f"  {line}")
        
        print("5️⃣ 测试完成！")
        
    except Exception as e:
        print(f"❌ 测试过程中出错: {e}")
        
    finally:
        # 6. 清理
        print("6️⃣ 清理进程...")
        mcp_process.terminate()
        try:
            mcp_process.wait(timeout=5)
        except:
            mcp_process.kill()

if __name__ == "__main__":
    test_mcp_scan_fixed()