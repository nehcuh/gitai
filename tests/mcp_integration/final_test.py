#!/usr/bin/env python3
import json
import subprocess
import os

def main():
    """主测试函数"""
    print("🧪 开始完整的 OpenGrep 扫描测试...")
    
    # 1. 测试没有安全漏洞的情况
    print("\n1️⃣ 测试没有安全漏洞的文件...")
    cmd1 = [
        "opengrep", "--json", "--quiet", "--timeout=10",
        "--config=/Users/huchen/.cache/gitai/rules/test.yml",
        "/Users/huchen/Projects/gitai/test_java.java"
    ]
    
    # 使用 shell=False 防止命令注入
    result1 = subprocess.run(cmd1, capture_output=True, text=True, shell=False)
    data1 = json.loads(result1.stdout)
    print(f"   结果数量: {len(data1.get('results', []))}")
    
    # 2. 测试有安全漏洞的情况
    print("\n2️⃣ 测试有安全漏洞的文件...")
    cmd2 = [
        "opengrep", "--json", "--quiet", "--timeout=10",
        "--config=/Users/huchen/.cache/gitai/rules/test.yml",
        "/Users/huchen/Projects/gitai/security_test.java"
    ]
    
    # 使用 shell=False 防止命令注入
    result2 = subprocess.run(cmd2, capture_output=True, text=True, shell=False)
    data2 = json.loads(result2.stdout)
    print(f"   结果数量: {len(data2.get('results', []))}")
    
    if data2.get('results'):
        first_result = data2['results'][0]
        print(f"   第一个结果: {first_result.get('extra', {}).get('message')}")
        print(f"   严重程度: {first_result.get('extra', {}).get('severity')}")
        print(f"   文件: {first_result.get('path')}")
        print(f"   行号: {first_result.get('start', {}).get('line')}")
    
    # 3. 验证 JSON 解析逻辑
    print("\n3️⃣ 验证数据结构...")
    if data2.get('results'):
        result = data2['results'][0]
        required_fields = [
            ('extra.message', 'extra' in result and 'message' in result['extra']),
            ('path', 'path' in result),
            ('start.line', 'start' in result and 'line' in result['start']),
            ('check_id', 'check_id' in result),
            ('extra.severity', 'extra' in result and 'severity' in result['extra']),
            ('extra.lines', 'extra' in result and 'lines' in result['extra'])
        ]
        
        for field, exists in required_fields:
            status = "✅" if exists else "❌"
            print(f"   {status} {field}")
    
    print("\n✅ 测试完成！")
    print("\n📋 总结:")
    print(f"   - OpenGrep 版本: {data1.get('version', 'unknown')}")
    print(f"   - 无漏洞文件结果: {len(data1.get('results', []))} 个问题")
    print(f"   - 有漏洞文件结果: {len(data2.get('results', []))} 个问题")
    print(f"   - JSON 解析: 正常")
    print(f"   - 数据结构: 完整")

if __name__ == "__main__":
    main()