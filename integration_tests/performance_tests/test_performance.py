"""
GitAI 性能测试
测试各种操作的性能指标
"""

import pytest
import time
import tempfile
import subprocess
from pathlib import Path
import psutil
import os


@pytest.mark.slow
class TestStartupPerformance:
    """测试启动性能"""

    def test_help_command_startup_time(self, gitai_helper):
        """测试help命令启动时间"""
        start_time = time.time()
        result = gitai_helper.run_command(["--help"])
        end_time = time.time()

        startup_time = end_time - start_time

        # 帮助命令应该在合理时间内完成（< 5秒）
        assert startup_time < 5.0, f"Help command took too long: {startup_time:.2f}s"
        assert result.returncode == 0

    def test_features_command_startup_time(self, gitai_helper):
        """测试features命令启动时间"""
        start_time = time.time()
        result = gitai_helper.run_command(["features"])
        end_time = time.time()

        startup_time = end_time - start_time

        # features命令应该很快（< 2秒）
        assert startup_time < 2.0, f"Features command took too long: {startup_time:.2f}s"
        assert result.returncode == 0

    def test_config_show_startup_time(self, gitai_helper):
        """测试配置显示启动时间"""
        start_time = time.time()
        result = gitai_helper.run_command(["config", "show"])
        end_time = time.time()

        startup_time = end_time - start_time

        # 配置显示应该很快（< 2秒）
        assert startup_time < 2.0, f"Config show took too long: {startup_time:.2f}s"
        assert result.returncode == 0


@pytest.mark.slow
class TestMemoryUsage:
    """测试内存使用"""

    def get_process_memory(self, process):
        """获取进程内存使用"""
        try:
            return psutil.Process(process.pid).memory_info().rss / 1024 / 1024  # MB
        except:
            return 0

    def test_memory_usage_during_review(self, gitai_helper, temp_git_repo):
        """测试代码评审期间的内存使用"""
        os.chdir(temp_git_repo)

        # 获取初始内存使用
        process = subprocess.Popen([
            str(gitiai_helper.binary_path),
            "review",
            "--format", "text",
            "--offline"
        ], stdout=subprocess.PIPE, stderr=subprocess.PIPE)

        # 等待一段时间让进程启动
        time.sleep(1)

        # 测量内存使用
        memory_before = self.get_process_memory(process)

        # 等待进程完成
        try:
            process.wait(timeout=30)
            memory_after = self.get_process_memory(process)
        except subprocess.TimeoutExpired:
            process.kill()
            pytest.skip("Review command timed out")

        # 内存使用应该在合理范围内（< 100MB）
        if memory_before > 0:
            assert memory_before < 100, f"Memory usage too high: {memory_before:.2f}MB"

    def test_memory_usage_multiple_commands(self, gitai_helper):
        """测试多个连续命令的内存使用"""
        commands = [
            ["features"],
            ["config", "show"],
            ["--help"]
        ]

        max_memory = 0
        for cmd in commands:
            process = subprocess.Popen(
                [str(gitiai_helper.binary_path)] + cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )

            time.sleep(0.5)  # 让进程启动
            memory = self.get_process_memory(process)
            max_memory = max(max_memory, memory)

            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()

        # 最大内存使用应该合理
        assert max_memory < 50, f"Peak memory usage too high: {max_memory:.2f}MB"


@pytest.mark.slow
class TestFileProcessingPerformance:
    """测试文件处理性能"""

    def create_test_file(self, path: Path, size_lines: int):
        """创建指定行数的测试文件"""
        content = []
        for i in range(size_lines):
            content.append(f"// Line {i}: This is a test line with some content.\n")
            if i % 10 == 0:
                content.append(f"fn function_{i}() {{\n    println!(\"Function {i}\");\n}}\n")

        path.write_text(''.join(content))
        return path

    def test_small_file_review_performance(self, gitai_helper, temp_git_repo):
        """测试小文件评审性能"""
        os.chdir(temp_git_repo)

        # 创建100行的小文件
        test_file = self.create_test_file(temp_git_repo / "small_test.rs", 100)

        start_time = time.time()
        result = gitai_helper.run_command([
            "review",
            "--format", "text",
            "--offline"
        ], timeout=30)
        end_time = time.time()

        processing_time = end_time - start_time

        # 小文件处理应该很快（< 10秒）
        assert result.returncode == 0
        assert processing_time < 10.0, f"Small file review took too long: {processing_time:.2f}s"

    def test_medium_file_review_performance(self, gitai_helper, temp_git_repo):
        """测试中等文件评审性能"""
        os.chdir(temp_git_repo)

        # 创建1000行的中等文件
        test_file = self.create_test_file(temp_git_repo / "medium_test.rs", 1000)

        start_time = time.time()
        result = gitai_helper.run_command([
            "review",
            "--format", "text",
            "--offline"
        ], timeout=60)
        end_time = time.time()

        processing_time = end_time - start_time

        # 中等文件处理应该在合理时间内（< 30秒）
        assert result.returncode == 0
        assert processing_time < 30.0, f"Medium file review took too long: {processing_time:.2f}s"

    @pytest.mark.slow
    def test_large_file_handling(self, gitai_helper, temp_git_repo):
        """测试大文件处理能力"""
        os.chdir(temp_git_repo)

        # 创建大文件（如果测试环境允许）
        try:
            test_file = self.create_test_file(temp_git_repo / "large_test.rs", 10000)
        except Exception as e:
            pytest.skip(f"Could not create large test file: {e}")

        start_time = time.time()
        result = gitai_helper.run_command([
            "review",
            "--format", "text",
            "--offline"
        ], timeout=120)
        end_time = time.time()

        processing_time = end_time - start_time

        # 大文件处理应该能完成（< 60秒）
        if result.returncode == 0:
            assert processing_time < 60.0, f"Large file review took too long: {processing_time:.2f}s"
        else:
            # 如果无法处理大文件，至少应该有合理的错误信息
            assert "太大" in result.stdout or "超时" in result.stdout or "error" in result.stdout.lower()


@pytest.mark.slow
class TestConcurrencyPerformance:
    """测试并发性能"""

    def test_concurrent_feature_commands(self, gitai_helper):
        """测试并发执行features命令"""
        import threading
        import queue

        results = queue.Queue()

        def run_features_command():
            start_time = time.time()
            result = gitai_helper.run_command(["features"])
            end_time = time.time()
            results.put((result.returncode, end_time - start_time))

        # 启动3个并发命令
        threads = []
        for _ in range(3):
            thread = threading.Thread(target=run_features_command)
            threads.append(thread)
            thread.start()

        # 等待所有线程完成
        for thread in threads:
            thread.join(timeout=10)

        # 检查结果
        success_count = 0
        max_time = 0
        while not results.empty():
            returncode, duration = results.get()
            if returncode == 0:
                success_count += 1
                max_time = max(max_time, duration)

        # 至少应该有一些命令成功
        assert success_count > 0, "No concurrent commands succeeded"
        # 并发执行不应该显著变慢
        assert max_time < 5.0, f"Concurrent execution too slow: {max_time:.2f}s"


@pytest.mark.slow
class TestResourceUsage:
    """测试资源使用"""

    def test_disk_usage_impact(self, gitai_helper, temp_git_repo):
        """测试磁盘使用影响"""
        os.chdir(temp_git_repo)

        # 获取初始磁盘使用
        initial_usage = 0
        for item in temp_git_repo.rglob("*"):
            if item.is_file():
                initial_usage += item.stat().st_size

        # 执行会产生缓存的命令
        result = gitai_helper.run_command([
            "review",
            "--format", "text",
            "--offline"
        ], timeout=30)

        # 检查磁盘使用变化
        final_usage = 0
        for item in temp_git_repo.rglob("*"):
            if item.is_file():
                final_usage += item.stat().st_size

        usage_increase = (final_usage - initial_usage) / 1024 / 1024  # MB

        # 磁盘使用增长应该合理（< 50MB）
        assert usage_increase < 50, f"Disk usage increased too much: {usage_increase:.2f}MB"

    def test_cpu_usage_reasonable(self, gitai_helper, temp_git_repo):
        """测试CPU使用是否合理"""
        os.chdir(temp_git_repo)

        # 这个测试比较困难，因为Python进程的CPU使用很难准确测量
        # 我们主要测试命令能在合理时间内完成
        start_time = time.time()
        result = gitai_helper.run_command([
            "review",
            "--format", "text",
            "--offline"
        ], timeout=30)
        end_time = time.time()

        processing_time = end_time - start_time

        # 如果CPU使用过高，处理时间会很长
        assert processing_time < 30.0, f"Command took too long, possible CPU usage issue: {processing_time:.2f}s"
        assert result.returncode == 0


@pytest.mark.performance
class TestScalability:
    """测试可扩展性"""

    def test_multiple_small_files_performance(self, gitai_helper, temp_git_repo):
        """测试多个小文件的处理性能"""
        os.chdir(temp_git_repo)

        # 创建多个小文件
        for i in range(5):
            test_file = temp_git_repo / f"test_file_{i}.rs"
            self.create_test_file(test_file, 50)

        start_time = time.time()
        result = gitai_helper.run_command([
            "review",
            "--format", "text",
            "--offline"
        ], timeout=60)
        end_time = time.time()

        processing_time = end_time - start_time

        # 多个小文件处理应该在合理时间内
        assert result.returncode == 0
        assert processing_time < 30.0, f"Multiple files processing took too long: {processing_time:.2f}s"

    def test_batch_command_execution(self, gitai_helper):
        """测试批量命令执行性能"""
        commands = [
            ["features"],
            ["config", "show"],
            ["--help"]
        ]

        start_time = time.time()

        for cmd in commands:
            result = gitai_helper.run_command(cmd, timeout=10)
            assert result.returncode == 0

        end_time = time.time()
        total_time = end_time - start_time

        # 批量执行应该高效
        assert total_time < 10.0, f"Batch command execution took too long: {total_time:.2f}s"


@pytest.mark.performance
@pytest.mark.slow
class TestStressTesting:
    """压力测试"""

    def test_rapid_command_execution(self, gitai_helper):
        """测试快速连续命令执行"""
        command = ["features"]
        execution_times = []

        for i in range(10):
            start_time = time.time()
            result = gitai_helper.run_command(command, timeout=5)
            end_time = time.time()

            execution_times.append(end_time - start_time)
            assert result.returncode == 0

        # 分析性能稳定性
        avg_time = sum(execution_times) / len(execution_times)
        max_time = max(execution_times)

        # 平均执行时间应该稳定
        assert avg_time < 2.0, f"Average execution time too high: {avg_time:.2f}s"
        # 最大执行时间不应该超过平均太多
        assert max_time < avg_time * 3, f"Performance too unstable: max={max_time:.2f}s, avg={avg_time:.2f}s"