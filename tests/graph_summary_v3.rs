use std::fs;

#[tokio::test]
async fn test_budget_truncation_sets_flag() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建一个临时目录，写入极简 Rust 源文件
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("a.rs");
    fs::write(
        &file,
        r#"
        fn f2() {}
        fn f1() { f2(); }
        "#,
    )?;

    // 调用摘要导出：radius=2、top_k=300、极小预算 -> 应触发降级并设置 truncated=true
    // 为测试降低最小字符预算，确保能触发降级
    std::env::set_var("GITAI_GRAPH_SUMMARY_MIN_CHAR_BUDGET", "0");

    let json = gitai::architectural_impact::graph_export::export_summary_string(
        dir.path(),      // scan_dir
        2,               // radius (允许被降级)
        300,             // top_k (较大，便于降级策略命中)
        false,           // seeds_from_diff
        "json",         // format
        10,              // budget_tokens（极小预算，强制触发降级）
        false,           // with_communities
        "labelprop",    // comm_alg（无关，本测试禁用社区）
        50,              // max_communities
        10,              // max_nodes_per_community
        false,           // with_paths
        5,               // path_samples
        5,               // path_max_hops
    )
    .await?;

    let v: serde_json::Value = serde_json::from_str(&json)?;
    assert!(
        v.get("truncated").and_then(|b| b.as_bool()).unwrap_or(false),
        "expected truncated=true under tight budget"
    );

    Ok(())
}
