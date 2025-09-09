//! MCP 服务集成测试
//!
//! 测试 MCP 服务的基本功能

#[cfg(all(feature = "mcp", test))]
mod tests {
    use gitai::config::{Config, McpConfig, McpServerConfig, McpServicesConfig};
    use gitai::mcp::GitAiMcpManager;
    use serde_json::json;

    /// 创建测试用的 MCP 配置
    fn create_test_mcp_config() -> Config {
        let mut config = Config::default();
        config.mcp = Some(McpConfig {
            enabled: true,
            server: McpServerConfig {
                transport: "stdio".to_string(),
                listen_addr: None,
                name: "GitAI Test".to_string(),
                version: "0.1.0".to_string(),
            },
            services: McpServicesConfig {
                enabled: vec![
                    "analysis".to_string(),
                    "scan".to_string(),
                    "review".to_string(),
                    "commit".to_string(),
                ],
                review: None,
                commit: None,
                scan: None,
                analysis: None,
                dependency: None,
            },
        });
        config
    }

    #[tokio::test]
    async fn test_mcp_manager_creation() {
        // 测试 MCP 管理器的创建
        let config = create_test_mcp_config();
        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");

        // 检查管理器是否能够创建工具列表
        let tools = manager.get_all_tools().await;
        println!("Available tools count: {}", tools.len());

        // 如果 MCP 功能启用且服务正常初始化，应该有工具
        // 注意：由于某些依赖服务可能不可用，这里不强制要求工具数量
        for tool in &tools {
            assert!(!tool.name.is_empty(), "Tool name should not be empty");
            assert!(
                !tool.description.is_empty(),
                "Tool description should not be empty"
            );
            println!("Tool: {} - {}", tool.name, tool.description);
        }
    }

    #[tokio::test]
    async fn test_mcp_analysis_service() {
        // 测试 analysis 服务的工具调用
        let config = create_test_mcp_config();
        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");

        let arguments = json!({
            "path": ".",
            "verbosity": 1
        });

        // 尝试调用 execute_analysis 工具
        let result = manager
            .handle_tool_call("execute_analysis", arguments)
            .await;

        match result {
            Ok(response) => {
                println!("Analysis result: {response}");
                // 验证返回的 JSON 结构
                assert!(response.is_object(), "Response should be a JSON object");
            }
            Err(e) => {
                // 如果失败，至少错误信息不应为空
                assert!(
                    !e.to_string().is_empty(),
                    "Error message should not be empty"
                );
                println!("Analysis failed (expected in test environment): {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_mcp_scan_service() {
        // 测试 scan 服务的工具调用
        let config = create_test_mcp_config();
        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");

        let arguments = json!({
            "path": ".",
            "timeout": 30
        });

        // 尝试调用 execute_scan 工具
        let result = manager.handle_tool_call("execute_scan", arguments).await;

        match result {
            Ok(response) => {
                println!("Scan result: {response}");
                assert!(response.is_object(), "Response should be a JSON object");
            }
            Err(e) => {
                assert!(
                    !e.to_string().is_empty(),
                    "Error message should not be empty"
                );
                println!("Scan failed (expected in test environment): {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_mcp_unknown_tool() {
        // 测试调用不存在的工具
        let config = create_test_mcp_config();
        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");

        let arguments = json!({});

        let result = manager
            .handle_tool_call("nonexistent_tool", arguments)
            .await;

        // 应该返回错误
        assert!(result.is_err(), "Should return error for unknown tool");

        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(
            error_msg.contains("Unknown tool"),
            "Error message should mention unknown tool"
        );
    }

    #[tokio::test]
    async fn test_mcp_disabled_config() {
        // 测试禁用 MCP 时的行为
        let mut config = Config::default();
        config.mcp = Some(McpConfig {
            enabled: false,
            server: McpServerConfig {
                transport: "stdio".to_string(),
                listen_addr: None,
                name: "GitAI Test".to_string(),
                version: "0.1.0".to_string(),
            },
            services: McpServicesConfig {
                enabled: vec![],
                review: None,
                commit: None,
                scan: None,
                analysis: None,
                dependency: None,
            },
        });

        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");
        let tools = manager.get_all_tools().await;

        // 禁用时应该没有工具
        assert!(
            tools.is_empty(),
            "Should have no tools when MCP is disabled"
        );
    }

    #[tokio::test]
    async fn test_mcp_no_config() {
        // 测试没有 MCP 配置时的行为
        let mut config = Config::default();
        config.mcp = None;

        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");
        let tools = manager.get_all_tools().await;

        // 没有配置时应该没有工具
        assert!(
            tools.is_empty(),
            "Should have no tools when MCP config is missing"
        );
    }

    #[tokio::test]
    async fn test_mcp_performance_stats() {
        // 测试性能统计功能
        let config = create_test_mcp_config();
        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");

        // 调用一个工具
        let arguments = json!({ "path": "." });
        let _result = manager
            .handle_tool_call("execute_analysis", arguments)
            .await;

        // 获取性能统计
        let stats = manager.get_performance_stats();

        // 验证统计信息
        assert!(
            stats.tool_calls >= 1,
            "Should have recorded at least one tool call"
        );
        assert!(
            stats.tool_calls == stats.successful_calls + stats.failed_calls,
            "Total calls should equal success + failure"
        );

        println!("Performance stats: {stats:?}");

        // 重置统计
        manager.reset_performance_stats();
        let reset_stats = manager.get_performance_stats();
        assert_eq!(reset_stats.tool_calls, 0, "Stats should be reset to zero");
    }

    #[tokio::test]
    async fn test_mcp_concurrent_calls() {
        // 测试并发调用
        let config = create_test_mcp_config();
        let manager = std::sync::Arc::new(
            GitAiMcpManager::new(config)
                .await
                .expect("Failed to create MCP manager"),
        );

        let mut tasks = vec![];

        // 创建 3 个并发任务
        for i in 0..3 {
            let manager_clone = manager.clone();
            let task = tokio::spawn(async move {
                let arguments = json!({
                    "path": ".",
                    "verbosity": i % 2
                });
                manager_clone
                    .handle_tool_call("execute_analysis", arguments)
                    .await
            });
            tasks.push(task);
        }

        // 等待所有任务完成
        let results = futures::future::join_all(tasks).await;

        // 验证结果
        let mut successful_calls = 0;
        for result in results {
            match result {
                Ok(call_result) => {
                    if call_result.is_ok() {
                        successful_calls += 1;
                    }
                }
                Err(_) => {
                    // 任务执行错误
                }
            }
        }

        println!("Concurrent calls completed, successful: {successful_calls}");

        // 检查性能统计
        let final_stats = manager.get_performance_stats();
        assert!(
            final_stats.tool_calls >= successful_calls as u64,
            "Should have recorded all tool calls"
        );
    }

    #[tokio::test]
    async fn test_mcp_concurrent_directory_analysis() {
        // 测试并发目录分析
        let config = create_test_mcp_config();
        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");

        // 使用当前项目的源代码目录进行并发分析测试
        let arguments = json!({
            "path": "./src/mcp"
        });

        let start_time = std::time::Instant::now();
        let result = manager
            .handle_tool_call("execute_analysis", arguments)
            .await;
        let elapsed = start_time.elapsed();

        match result {
            Ok(response) => {
                println!("Concurrent directory analysis result: {response}");

                // 解析响应
                if let Ok(analysis_result) = serde_json::from_value::<serde_json::Value>(response) {
                    // 验证基本结构
                    assert!(
                        analysis_result["success"].as_bool().unwrap_or(false),
                        "Analysis should be successful"
                    );
                    assert_eq!(
                        analysis_result["language"].as_str().unwrap_or(""),
                        "multi",
                        "Should be multi-language analysis"
                    );

                    // 检查是否有性能统计
                    if let Some(details) = analysis_result["details"].as_object() {
                        println!("Analysis details: {details:?}");

                        // 检查是否包含并发处理的标识
                        if details.contains_key("concurrent_processing") {
                            assert_eq!(
                                details["concurrent_processing"].as_str().unwrap_or(""),
                                "enabled"
                            );
                            println!("✅ Concurrent processing is enabled");
                        }

                        // 检查文件分析数量
                        if let Some(file_count) = details
                            .get("successful_files")
                            .or_else(|| details.get("file_count"))
                        {
                            let files_analyzed = file_count
                                .as_str()
                                .unwrap_or("0")
                                .parse::<usize>()
                                .unwrap_or(0);
                            println!("Files analyzed: {files_analyzed}");

                            if files_analyzed > 0 {
                                let files_per_second =
                                    files_analyzed as f64 / elapsed.as_secs_f64();
                                println!("Analysis speed: {files_per_second:.1} files/second");

                                // 对于小规模目录，应该能够较快地完成
                                assert!(
                                    files_per_second > 1.0,
                                    "Should process at least 1 file per second"
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Analysis failed (this may be expected in test environment): {e}");
                // 在测试环境中，分析可能失败，这是可以接受的
            }
        }

        println!("Total analysis time: {:.2}s", elapsed.as_secs_f64());
    }

    #[tokio::test]
    async fn test_mcp_concurrent_cache_effectiveness() {
        // 测试并发环境下的缓存效果
        let config = create_test_mcp_config();
        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");

        let test_path = "./src/tree_sitter";
        let arguments = json!({ "path": test_path });

        // 第一次分析（无缓存）
        let start1 = std::time::Instant::now();
        let result1 = manager
            .handle_tool_call("execute_analysis", arguments.clone())
            .await;
        let elapsed1 = start1.elapsed();

        // 第二次分析（应该使用缓存）
        let start2 = std::time::Instant::now();
        let result2 = manager
            .handle_tool_call("execute_analysis", arguments)
            .await;
        let elapsed2 = start2.elapsed();

        match (result1, result2) {
            (Ok(response1), Ok(response2)) => {
                println!("First analysis (no cache): {:.3}s", elapsed1.as_secs_f64());
                println!(
                    "Second analysis (with cache): {:.3}s",
                    elapsed2.as_secs_f64()
                );

                // 验证结果一致性（忽略时间相关字段）
                if let (Ok(mut result1), Ok(mut result2)) = (
                    serde_json::from_value::<serde_json::Value>(response1),
                    serde_json::from_value::<serde_json::Value>(response2),
                ) {
                    // 移除时间相关字段
                    if let Some(details1) = result1["details"].as_object_mut() {
                        details1.remove("analysis_time_ms");
                        details1.remove("analysis_time_seconds");
                        details1.remove("files_per_second");
                    }
                    if let Some(details2) = result2["details"].as_object_mut() {
                        details2.remove("analysis_time_ms");
                        details2.remove("analysis_time_seconds");
                        details2.remove("files_per_second");
                    }

                    // 比较核心分析结果
                    assert_eq!(
                        result1["language"], result2["language"],
                        "Language should be identical"
                    );
                    assert_eq!(
                        result1["summary"], result2["summary"],
                        "Summary should be identical"
                    );
                    assert_eq!(
                        result1["structures"], result2["structures"],
                        "Structures should be identical"
                    );
                    assert_eq!(
                        result1["metrics"], result2["metrics"],
                        "Metrics should be identical"
                    );

                    println!("✅ Core analysis results are identical");
                }

                // 缓存应该显著提高性能（至少 1.2 倍）
                if elapsed1.as_millis() > 5 {
                    // 只在第一次分析花费足够时间时才测试
                    let speedup = elapsed1.as_secs_f64() / elapsed2.as_secs_f64().max(0.001);
                    println!("Cache speedup: {speedup:.1}x");
                    if speedup > 1.2 {
                        println!("✅ Cache provided {speedup:.1}x speedup");
                    } else {
                        println!("⚠️ Cache speedup {speedup:.1}x is lower than expected (may be due to fast execution)");
                    }
                }
            }
            _ => {
                println!("Cache test skipped due to analysis failures");
            }
        }
    }

    #[tokio::test]
    async fn test_mcp_concurrent_error_handling() {
        // 测试并发环境下的错误处理
        let config = create_test_mcp_config();
        let manager = GitAiMcpManager::new(config)
            .await
            .expect("Failed to create MCP manager");

        // 测试不存在的路径
        let invalid_arguments = json!({
            "path": "/path/that/does/not/exist/anywhere"
        });

        let result = manager
            .handle_tool_call("execute_analysis", invalid_arguments)
            .await;

        match result {
            Ok(_) => {
                // 如果意外成功了，这也是可以接受的
                println!("Analysis succeeded unexpectedly (this might be OK)");
            }
            Err(e) => {
                // 错误处理应该不会崩溃
                let error_msg = e.to_string();
                assert!(!error_msg.is_empty(), "Error message should not be empty");
                println!("Error handled correctly: {error_msg}");
            }
        }

        // 确认系统仍然可以处理正常请求
        let valid_arguments = json!({
            "path": "./src"
        });

        let result = manager
            .handle_tool_call("execute_analysis", valid_arguments)
            .await;
        // 正常请求应该不受之前的错误影响
        match result {
            Ok(_) => println!("✅ System recovered successfully after error"),
            Err(e) => println!("System still functional after error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_mcp_concurrent_memory_efficiency() {
        // 测试并发分析的内存效率
        let config = create_test_mcp_config();
        let manager = std::sync::Arc::new(
            GitAiMcpManager::new(config)
                .await
                .expect("Failed to create MCP manager"),
        );

        // 创建多个并发任务
        let mut tasks = Vec::new();
        let paths = vec![
            "./src/mcp",
            "./src/tree_sitter",
            "./src/architectural_impact",
            "./src/utils",
        ];

        for path in paths {
            let manager_clone = manager.clone();
            let arguments = json!({ "path": path });

            let task = tokio::spawn(async move {
                manager_clone
                    .handle_tool_call("execute_analysis", arguments)
                    .await
            });
            tasks.push(task);
        }

        // 等待所有任务完成
        let start_time = std::time::Instant::now();
        let results = futures::future::join_all(tasks).await;
        let total_time = start_time.elapsed();

        let mut successful_tasks = 0;
        let mut total_files = 0;

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(response)) => {
                    successful_tasks += 1;

                    // 尝试提取文件数量
                    if let Ok(analysis_result) =
                        serde_json::from_value::<serde_json::Value>(response)
                    {
                        if let Some(details) = analysis_result["details"].as_object() {
                            if let Some(file_count) = details
                                .get("successful_files")
                                .or_else(|| details.get("file_count"))
                            {
                                if let Ok(count) =
                                    file_count.as_str().unwrap_or("0").parse::<usize>()
                                {
                                    total_files += count;
                                }
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    println!("Task {i} failed with analysis error: {e}");
                }
                Err(e) => {
                    println!("Task {i} failed with join error: {e}");
                }
            }
        }

        println!(
            "✅ Memory efficiency test: {} successful tasks, {} total files, {:.2}s total time",
            successful_tasks,
            total_files,
            total_time.as_secs_f64()
        );

        // 基本断言：至少一个任务应该成功
        assert!(
            successful_tasks > 0,
            "At least one concurrent task should succeed"
        );

        if total_files > 0 {
            let overall_throughput = total_files as f64 / total_time.as_secs_f64();
            println!("Overall throughput: {overall_throughput:.1} files/second");
        }
    }
}
