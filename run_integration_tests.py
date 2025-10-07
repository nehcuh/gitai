#!/usr/bin/env python3
"""
GitAI集成测试运行器
简化的测试运行脚本，用于快速验证GitAI功能
"""

import sys
import os
import subprocess
import time
from pathlib import Path


def check_gitai_binary():
    """检查GitAI二进制文件是否存在"""
    gitai_binary = Path(__file__).parent / "target" / "release" / "gitai"
    if not gitai_binary.exists():
        print("❌ GitAI binary not found")
        print("Please run: cargo build --release")
        return False

    print(f"✅ GitAI binary found: {gitai_binary}")
    return True


def run_quick_tests():
    """运行快速功能测试"""
    print("\n🧪 Running Quick Functionality Tests...")

    gitai_binary = Path(__file__).parent / "target" / "release" / "gitai"
    tests = [
        (["--help"], "Help command"),
        (["features"], "Features command"),
        (["config", "show"], "Config show command"),
    ]

    passed = 0
    total = len(tests)

    for cmd, description in tests:
        print(f"  Testing {description}...", end=" ")
        try:
            result = subprocess.run(
                [str(gitai_binary)] + cmd,
                capture_output=True,
                text=True,
                timeout=10
            )

            if result.returncode == 0:
                print("✅ PASS")
                passed += 1
            else:
                print("❌ FAIL")
                print(f"    Error: {result.stderr}")
        except subprocess.TimeoutExpired:
            print("❌ TIMEOUT")
        except Exception as e:
            print(f"❌ ERROR: {e}")

    print(f"\nQuick Tests: {passed}/{total} passed")
    return passed == total


def run_review_test():
    """测试代码评审功能"""
    print("\n🔍 Testing Code Review Functionality...")

    gitai_binary = Path(__file__).parent / "target" / "release" / "gitai"

    try:
        # 创建临时测试目录
        test_dir = Path(__file__).parent / "test_review_temp"
        test_dir.mkdir(exist_ok=True)

        # 初始化Git仓库
        subprocess.run(["git", "init"], cwd=test_dir, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Test User"], cwd=test_dir, check=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=test_dir, check=True)

        # 创建测试文件
        test_file = test_dir / "test.rs"
        test_file.write_text('''
fn main() {
    println!("Hello, GitAI!");

    let numbers = vec![1, 2, 3];
    let sum: i32 = numbers.iter().sum();

    if sum > 0 {
        println!("Sum is positive");
    }
}
''')

        # 添加到Git
        subprocess.run(["git", "add", "."], cwd=test_dir, check=True)
        subprocess.run(["git", "commit", "-m", "Initial commit"], cwd=test_dir, check=True)

        # 运行代码评审
        result = subprocess.run(
            [str(gitai_binary), "review", "--format", "text", "--offline"],
            cwd=test_dir,
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode == 0 and "代码评审报告" in result.stdout:
            print("  ✅ Code Review: PASS")

            # 清理
            import shutil
            shutil.rmtree(test_dir)
            return True
        else:
            print("  ❌ Code Review: FAIL")
            print(f"    Output: {result.stdout}")
            print(f"    Error: {result.stderr}")
            return False

    except Exception as e:
        print(f"  ❌ Code Review: ERROR - {e}")
        return False


def run_mcp_test():
    """测试MCP功能"""
    print("\n🌐 Testing MCP Functionality...")

    gitai_binary = Path(__file__).parent / "target" / "release" / "gitai"

    try:
        # 测试MCP命令帮助
        result = subprocess.run(
            [str(gitai_binary), "mcp", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )

        if result.returncode == 0:
            print("  ✅ MCP Help: PASS")
        else:
            print("  ❌ MCP Help: FAIL")
            return False

        # 测试MCP健康检查（预期会失败，因为服务器未运行）
        result = subprocess.run(
            [str(gitai_binary), "mcp-health"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # 这应该失败，但错误信息应该包含Connection refused
        if "Connection refused" in result.stdout or result.returncode != 0:
            print("  ✅ MCP Health Check: Expected behavior (server not running)")
            return True
        else:
            print("  ❌ MCP Health Check: Unexpected result")
            return False

    except Exception as e:
        print(f"  ❌ MCP Test: ERROR - {e}")
        return False


def run_performance_test():
    """简单性能测试"""
    print("\n⚡ Testing Performance...")

    gitai_binary = Path(__file__).parent / "target" / "release" / "gitai"

    try:
        # 测试启动时间
        start_time = time.time()
        result = subprocess.run(
            [str(gitai_binary), "features"],
            capture_output=True,
            text=True,
            timeout=10
        )
        end_time = time.time()

        startup_time = end_time - start_time

        if result.returncode == 0 and startup_time < 2.0:
            print(f"  ✅ Startup Time: {startup_time:.2f}s (PASS)")
            return True
        else:
            print(f"  ❌ Startup Time: {startup_time:.2f}s (FAIL - should be < 2.0s)")
            return False

    except Exception as e:
        print(f"  ❌ Performance Test: ERROR - {e}")
        return False


def main():
    """主函数"""
    print("🚀 GitAI v2.1.0 Integration Test Runner")
    print("=" * 50)

    # 检查GitAI二进制文件
    if not check_gitai_binary():
        sys.exit(1)

    # 运行测试
    tests = [
        ("Quick Functionality", run_quick_tests),
        ("Code Review", run_review_test),
        ("MCP Integration", run_mcp_test),
        ("Performance", run_performance_test),
    ]

    passed = 0
    total = len(tests)

    for test_name, test_func in tests:
        try:
            if test_func():
                passed += 1
        except Exception as e:
            print(f"  ❌ {test_name}: UNEXPECTED ERROR - {e}")

    # 结果汇总
    print("\n" + "=" * 50)
    print(f"📊 Test Results: {passed}/{total} test suites passed")

    if passed == total:
        print("🎉 All tests passed! GitAI is ready for production.")
        return 0
    else:
        print("⚠️  Some tests failed. Please check the issues above.")
        return 1


if __name__ == "__main__":
    sys.exit(main())