use gitai::tree_sitter::{cache::CacheKey, cache::TreeSitterCache, StructuralSummary};
use std::sync::Arc;

fn trivial_summary(lang: &str) -> StructuralSummary {
    StructuralSummary {
        language: lang.to_string(),
        language_summaries: std::collections::HashMap::new(),
        functions: vec![],
        classes: vec![],
        imports: vec![],
        exports: vec![],
        comments: vec![],
        complexity_hints: vec![],
        calls: vec![],
    }
}

#[tokio::test]
async fn test_cache_concurrent_set_same_key_atomic_on_disk() {
    // Use a shared cache directory across test instances
    std::env::set_var("GITAI_TS_CACHE_TEST_SHARED", "true");

    let cache = Arc::new(TreeSitterCache::new(64, 3600).expect("cache new"));

    let key = CacheKey::from_content("fn main(){}", "rust");
    let summary = trivial_summary("rust");

    // Spawn many writers setting the same key concurrently
    let mut joins = Vec::new();
    for _ in 0..16 {
        let c = cache.clone();
        let k = key.clone();
        let s = summary.clone();
        joins.push(tokio::spawn(async move {
            // Several rounds to create high contention
            for _ in 0..50 {
                c.set(k.clone(), s.clone()).expect("set ok");
            }
        }));
    }
    for j in joins {
        j.await.expect("join ok");
    }

    // New cache in the same shared directory should be able to read from disk
    let cache2 = TreeSitterCache::new(64, 3600).expect("cache2 new");
    let got = cache2.get(&key);
    assert!(
        got.is_some(),
        "expected disk-backed cache hit after concurrent writes"
    );
}

#[tokio::test]
async fn test_cache_get_set_race_single_instance() {
    // Use a shared cache directory across test instances (not strictly needed here)
    std::env::set_var("GITAI_TS_CACHE_TEST_SHARED", "true");

    let cache = Arc::new(TreeSitterCache::new(64, 3600).expect("cache new"));

    let key = CacheKey::from_content("pub fn f(){}", "rust");
    let summary = trivial_summary("rust");

    // Spawn tasks that perform get-or-set concurrently
    let mut joins = Vec::new();
    for _ in 0..32 {
        let c = cache.clone();
        let k = key.clone();
        let s = summary.clone();
        joins.push(tokio::spawn(async move {
            for _ in 0..100 {
                if c.get(&k).is_none() {
                    let _ = c.set(k.clone(), s.clone());
                }
            }
        }));
    }

    for j in joins {
        j.await.expect("join ok");
    }

    let stats = cache.stats();
    assert!(
        stats.hits + stats.misses > 0,
        "stats should record accesses"
    );
    // Sanity: ensure we can retrieve the value at the end
    assert!(cache.get(&key).is_some());
}
