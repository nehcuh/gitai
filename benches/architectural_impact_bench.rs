use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gitai::architectural_impact::{
    git_state_analyzer::GitStateAnalyzer, ArchitecturalImpactAnalysis, BreakingChange,
    BreakingChangeType, ImpactLevel,
};
use gitai::tree_sitter::{ClassInfo, FunctionInfo, StructuralSummary};
/// 创建一个大型的测试 StructuralSummary
fn create_large_summary(function_count: usize, class_count: usize) -> StructuralSummary {
    let mut functions = Vec::new();
    for i in 0..function_count {
        functions.push(FunctionInfo {
            name: format!("function_{}", i),
            visibility: if i % 2 == 0 { "public" } else { "private" }.to_string(),
            is_async: i % 3 == 0,
            parameters: vec!["String".to_string(); i % 5],
            return_type: Some(format!("Type{}", i % 10)),
            start_line: i * 10,
            end_line: i * 10 + 8,
            complexity: (i % 10) as u32,
            calls_count: (i % 20) as u32,
        });
    }

    let mut classes = Vec::new();
    for i in 0..class_count {
        classes.push(ClassInfo {
            name: format!("Class_{}", i),
            visibility: if i % 2 == 0 { "public" } else { "private" }.to_string(),
            methods_count: (i % 20) as u32,
            fields_count: (i % 10) as u32,
            is_abstract: i % 5 == 0,
            has_tests: i % 3 == 0,
            start_line: i * 50,
            end_line: i * 50 + 45,
        });
    }

    StructuralSummary {
        functions,
        classes,
        comments: vec![],
        imports: vec![],
        complexity_metrics: Default::default(),
        code_quality_metrics: Default::default(),
    }
}

/// 创建一个大型的测试 diff
fn create_large_diff(file_count: usize, changes_per_file: usize) -> String {
    let mut diff = String::new();

    for f in 0..file_count {
        diff.push_str(&format!(
            "diff --git a/src/file_{}.rs b/src/file_{}.rs\n",
            f, f
        ));
        diff.push_str(&format!("index abc{}..def{} 100644\n", f, f));
        diff.push_str(&format!("--- a/src/file_{}.rs\n", f));
        diff.push_str(&format!("+++ b/src/file_{}.rs\n", f));

        for c in 0..changes_per_file {
            diff.push_str(&format!("@@ -{},7 +{},7 @@\n", c * 10, c * 10));
            if c % 3 == 0 {
                diff.push_str(&format!("-fn old_function_{}() {{\n", c));
                diff.push_str(&format!("+fn new_function_{}(param: String) {{\n", c));
            } else if c % 3 == 1 {
                diff.push_str(&format!("+struct NewStruct_{} {{\n", c));
                diff.push_str("+    field: String,\n");
                diff.push_str("+}\n");
            } else {
                diff.push_str(&format!("+trait NewTrait_{} {{\n", c));
                diff.push_str("+    fn method(&self);\n");
                diff.push_str("+}\n");
            }
        }
    }

    diff
}

fn benchmark_ast_comparison(c: &mut Criterion) {
    let before = create_large_summary(100, 20);
    let mut after = before.clone();

    // 修改一些函数和类
    for i in 0..10 {
        after.functions[i].parameters.push("NewParam".to_string());
    }
    for i in 0..5 {
        after.classes[i].methods_count += 5;
    }

    c.bench_function("ast_comparison_100_functions", |b| {
        b.iter(|| {
            gitai::architectural_impact::ast_comparison::compare_structural_summaries(
                black_box(&before),
                black_box(&after),
            )
        })
    });
}

fn benchmark_risk_assessment(c: &mut Criterion) {
    let mut analysis = ArchitecturalImpactAnalysis::new();

    // 添加多个破坏性变更
    for i in 0..50 {
        analysis.add_breaking_change(BreakingChange {
            change_type: if i % 3 == 0 {
                BreakingChangeType::FunctionRemoved
            } else if i % 3 == 1 {
                BreakingChangeType::ParameterCountChanged
            } else {
                BreakingChangeType::StructureChanged
            },
            component: format!("component_{}", i),
            description: format!("Change {}", i),
            impact_level: if i % 2 == 0 {
                ImpactLevel::Module
            } else {
                ImpactLevel::Local
            },
            suggestions: vec![],
            before: Some(format!("before_{}", i)),
            after: Some(format!("after_{}", i)),
            file_path: format!("src/file_{}.rs", i % 10),
        });
    }

    c.bench_function("risk_assessment_50_changes", |b| {
        b.iter(|| {
            let mut a = analysis.clone();
            a.calculate_overall_risk();
            black_box(a.risk_level)
        })
    });
}

fn benchmark_ai_context_generation(c: &mut Criterion) {
    let mut analysis = ArchitecturalImpactAnalysis::new();

    // 添加各种类型的变更
    for i in 0..30 {
        analysis.add_breaking_change(BreakingChange {
            change_type: match i % 5 {
                0 => BreakingChangeType::FunctionRemoved,
                1 => BreakingChangeType::ParameterCountChanged,
                2 => BreakingChangeType::StructureChanged,
                3 => BreakingChangeType::InterfaceChanged,
                _ => BreakingChangeType::FunctionAdded,
            },
            component: format!("component_{}", i),
            description: format!("Detailed description of change {}", i),
            impact_level: match i % 3 {
                0 => ImpactLevel::Project,
                1 => ImpactLevel::Module,
                _ => ImpactLevel::Local,
            },
            suggestions: vec![
                format!("Suggestion 1 for change {}", i),
                format!("Suggestion 2 for change {}", i),
            ],
            before: Some(format!("// Before state {}", i)),
            after: Some(format!("// After state {}", i)),
            file_path: format!("src/module_{}/file_{}.rs", i % 5, i),
        });
    }

    c.bench_function("ai_context_generation_30_changes", |b| {
        b.iter(|| {
            let mut a = analysis.clone();
            a.generate_ai_context();
            black_box(a.get_ai_context())
        })
    });
}

fn benchmark_diff_analysis(c: &mut Criterion) {
    let diff_small = create_large_diff(5, 10);
    let diff_large = create_large_diff(20, 20);

    c.bench_function("diff_analysis_small", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|_| async {
                let analyzer = GitStateAnalyzer::new();
                black_box(analyzer.analyze_git_diff(&diff_small).await)
            })
    });

    c.bench_function("diff_analysis_large", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|_| async {
                let analyzer = GitStateAnalyzer::new();
                black_box(analyzer.analyze_git_diff(&diff_large).await)
            })
    });
}

criterion_group!(
    benches,
    benchmark_ast_comparison,
    benchmark_risk_assessment,
    benchmark_ai_context_generation,
    benchmark_diff_analysis
);
criterion_main!(benches);
