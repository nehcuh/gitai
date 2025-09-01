#!/usr/bin/env python3
import json
import subprocess
import sys

def test_opengrep_scan():
    """测试 OpenGrep 扫描功能"""
    print("🔍 测试 OpenGrep 扫描...")
    
    # 测试文件
    test_file = "/Users/huchen/Projects/gitai/test_java.java"
    
    # 运行 OpenGrep
    cmd = [
        "opengrep",
        "--json", 
        "--quiet",
        "--timeout=10",
        "--config=auto",
        test_file
    ]
    
    try:
        # 使用 shell=False (默认值) 确保安全，防止命令注入
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30, shell=False)
        print(f"📤 OpenGrep 命令: {' '.join(cmd)}")
        print(f"📤 返回码: {result.returncode}")
        print(f"📤 标准输出: {result.stdout}")
        print(f"📤 标准错误: {result.stderr}")
        
        if result.stdout:
            try:
                data = json.loads(result.stdout)
                print(f"📋 JSON 解析成功")
                print(f"📋 版本: {data.get('version', 'unknown')}")
                print(f"📋 结果数量: {len(data.get('results', []))}")
                print(f"📋 错误数量: {len(data.get('errors', []))}")
                print(f"📋 扫描路径: {data.get('paths', {})}")
            except json.JSONDecodeError as e:
                print(f"❌ JSON 解析失败: {e}")
        
    except subprocess.TimeoutExpired:
        print("❌ 命令超时")
    except Exception as e:
        print(f"❌ 执行失败: {e}")

if __name__ == "__main__":
    test_opengrep_scan()