#!/usr/bin/env python3
import json
import subprocess
import os

def compare_scan_methods():
    """比较命令行和 MCP 扫描的差异"""
    print("🔍 比较命令行和 MCP 扫描的差异...")
    
    # 1. 测试命令行扫描
    print("\n1️⃣ 测试命令行扫描...")
    cmd_result = subprocess.run([
        "./target/release/gitai", "scan", 
        "--path", "/Users/huchen/Projects/java-sec-code",
        "--timeout", "30",
        "--format", "json"
    ], capture_output=True, text=True, timeout=120)
    
    print(f"命令行返回码: {cmd_result.returncode}")
    if cmd_result.stdout:
        try:
            cli_data = json.loads(cmd_result.stdout)
            print(f"命令行结果: {len(cli_data.get('findings', []))} 个问题")
            print(f"命令行错误: {cli_data.get('error', 'None')}")
            print(f"命令行工具: {cli_data.get('tool', 'unknown')}")
            print(f"命令行版本: {cli_data.get('version', 'unknown')}")
        except:
            print(f"命令行输出不是有效的 JSON")
            print(f"输出前200字符: {cmd_result.stdout[:200]}")
    
    if cmd_result.stderr:
        print(f"命令行错误输出: {cmd_result.stderr[:200]}")
    
    # 2. 检查规则目录
    print("\n2️⃣ 检查规则目录...")
    possible_rule_dirs = [
        "~/.cache/gitai/rules",
        "~/.gitai/cache/rules", 
        "/tmp/gitai/rules",
        "./rules"
    ]
    
    for dir_path in possible_rule_dirs:
        expanded_path = os.path.expanduser(dir_path)
        if os.path.exists(expanded_path):
            print(f"找到规则目录: {expanded_path}")
            try:
                files = os.listdir(expanded_path)
                print(f"  文件: {files[:10]}")  # 显示前10个文件
            except:
                pass
    
    # 3. 检查 OpenGrep 直接调用
    print("\n3️⃣ 测试 OpenGrep 直接调用...")
    opengrep_result = subprocess.run([
        "opengrep", "--json", "--quiet", "--timeout=30",
        "--config=auto", "/Users/huchen/Projects/java-sec-code"
    ], capture_output=True, text=True, timeout=60)
    
    print(f"OpenGrep 返回码: {opengrep_result.returncode}")
    if opengrep_result.stdout:
        try:
            og_data = json.loads(opengrep_result.stdout)
            print(f"OpenGrep 结果: {len(og_data.get('results', []))} 个问题")
            print(f"OpenGrep 版本: {og_data.get('version', 'unknown')}")
        except:
            print(f"OpenGrep 输出不是有效的 JSON")
            print(f"输出前200字符: {opengrep_result.stdout[:200]}")
    
    if opengrep_result.stderr:
        print(f"OpenGrep 错误输出: {opengrep_result.stderr[:200]}")

if __name__ == "__main__":
    compare_scan_methods()