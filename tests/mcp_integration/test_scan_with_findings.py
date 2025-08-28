#!/usr/bin/env python3
import json
import subprocess

def test_scan_with_findings():
    """测试有安全漏洞发现的扫描"""
    print("🔍 测试有安全漏洞的扫描...")
    
    cmd = [
        "opengrep",
        "--json", 
        "--quiet",
        "--timeout=10",
        "--config=/Users/huchen/.cache/gitai/rules/test.yml",
        "/Users/huchen/Projects/gitai/security_test.java"
    ]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        print(f"📤 返回码: {result.returncode}")
        
        if result.stdout:
            data = json.loads(result.stdout)
            print(f"📋 版本: {data.get('version', 'unknown')}")
            print(f"📋 结果数量: {len(data.get('results', []))}")
            print(f"📋 错误数量: {len(data.get('errors', []))}")
            
            # 详细分析第一个结果
            if data.get('results'):
                first_result = data['results'][0]
                print(f"🔍 第一个结果详情:")
                print(f"   check_id: {first_result.get('check_id')}")
                print(f"   path: {first_result.get('path')}")
                print(f"   line: {first_result.get('start', {}).get('line')}")
                print(f"   message: {first_result.get('extra', {}).get('message')}")
                print(f"   severity: {first_result.get('extra', {}).get('severity')}")
                print(f"   lines: {first_result.get('extra', {}).get('lines', '')[:50]}...")
                
                # 验证我们的解析逻辑需要的数据结构
                print(f"📋 验证数据结构:")
                print(f"   有 extra.message: {'extra' in first_result and 'message' in first_result['extra']}")
                print(f"   有 path: {'path' in first_result}")
                print(f"   有 start.line: {'start' in first_result and 'line' in first_result['start']}")
                print(f"   有 check_id: {'check_id' in first_result}")
                print(f"   有 severity: {'extra' in first_result and 'severity' in first_result['extra']}")
                print(f"   有 lines: {'extra' in first_result and 'lines' in first_result['extra']}")
            
    except Exception as e:
        print(f"❌ 执行失败: {e}")

if __name__ == "__main__":
    test_scan_with_findings()