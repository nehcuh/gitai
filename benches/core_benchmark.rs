use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_basic_operations(c: &mut Criterion) {
    // 基准测试：基本字符串操作
    c.bench_function("string_clone", |b| {
        let s = "Hello, World!".to_string();
        b.iter(|| {
            let cloned = s.clone();
            black_box(cloned)
        })
    });

    // 基准测试：向量操作
    c.bench_function("vector_operations", |b| {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..100 {
                v.push(i);
            }
            black_box(v)
        })
    });
}

fn benchmark_string_operations(c: &mut Criterion) {
    let sample_code = r#"
    fn hello_world() {
        println!("Hello, world!");
        let x = 42;
        let y = x * 2;
        if y > 50 {
            println!("y is greater than 50");
        }
    }
    "#;

    // 基准测试：字符串处理
    c.bench_function("string_processing", |b| {
        b.iter(|| {
            let lines: Vec<&str> = sample_code.lines().collect();
            let filtered: Vec<&str> = lines.into_iter()
                .filter(|line| !line.trim().is_empty())
                .collect();
            black_box(filtered)
        })
    });
}

criterion_group!(benches, benchmark_basic_operations, benchmark_string_operations);
criterion_main!(benches);