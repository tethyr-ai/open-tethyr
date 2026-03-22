use criterion::{criterion_group, criterion_main, Criterion};
use open_tethyr::ax::*;
use open_tethyr::cache::memory::{CacheEntry, MemoryCache};
use open_tethyr::config::*;
use serde_json;
use std::time::{Duration, Instant};

fn bench_cache_put(c: &mut Criterion) {
    let cache = MemoryCache::new(10_000);
    c.bench_function("cache_put", |b| {
        let mut i = 0u64;
        b.iter(|| {
            i += 1;
            cache
                .put(
                    format!("domain-{}.com", i),
                    CacheEntry {
                        data: r#"{"records":[]}"#.into(),
                        created_at: Instant::now(),
                        ttl: Duration::from_secs(3600),
                        no_cache: false,
                    },
                )
                .unwrap();
        })
    });
}

fn bench_cache_get(c: &mut Criterion) {
    let cache = MemoryCache::new(10_000);
    for i in 0..1000 {
        cache
            .put(
                format!("domain-{}.com", i),
                CacheEntry {
                    data: r#"{"records":[]}"#.into(),
                    created_at: Instant::now(),
                    ttl: Duration::from_secs(3600),
                    no_cache: false,
                },
            )
            .unwrap();
    }
    c.bench_function("cache_get", |b| b.iter(|| cache.get("domain-500.com")));
}

fn bench_ax_parse(c: &mut Criterion) {
    let json = serde_json::to_string(&AgentExchangeDocument {
        records: vec![AgentExchangeRecord {
            record_type: "AX".into(),
            version: "1.0".into(),
            agent: Agent {
                name: "test".into(),
                description: "desc".into(),
                provider: "corp".into(),
            },
            endpoints: vec![Endpoint {
                protocol: Protocol::Rest,
                url: "https://api.example.com".into(),
                auth: vec!["OAuth2".into()],
                content_type: None,
                extra: serde_json::Map::new(),
            }],
            capabilities: None,
            schema: None,
            limits: None,
            security: None,
            extensions: None,
        }],
    })
    .unwrap();
    c.bench_function("ax_parse", |b| {
        b.iter(|| serde_json::from_str::<AgentExchangeDocument>(&json).unwrap())
    });
}

fn bench_ax_validate(c: &mut Criterion) {
    let record = AgentExchangeRecord {
        record_type: "AX".into(),
        version: "1.0".into(),
        agent: Agent {
            name: "test".into(),
            description: "desc".into(),
            provider: "corp".into(),
        },
        endpoints: vec![Endpoint {
            protocol: Protocol::Rest,
            url: "https://api.example.com".into(),
            auth: vec!["OAuth2".into()],
            content_type: None,
            extra: serde_json::Map::new(),
        }],
        capabilities: None,
        schema: None,
        limits: None,
        security: None,
        extensions: None,
    };
    c.bench_function("ax_validate", |b| {
        b.iter(|| AxValidator::validate_record(&record))
    });
}

fn bench_config_merge(c: &mut Criterion) {
    let defaults = AgentDefaults {
        provider: Some("corp".into()),
        auth: vec!["OAuth2".into()],
        ..Default::default()
    };
    let agent = AgentDefinition {
        name: "agent".into(),
        description: "desc".into(),
        provider: None,
        endpoints: vec![],
        auth: vec![],
        capabilities: None,
        limits: None,
        security: None,
        extensions: None,
        oauth_provider: None,
        oauth_domain: None,
    };
    c.bench_function("config_merge", |b| {
        b.iter(|| ConfigMerger::merge_agent(&defaults, &agent))
    });
}

criterion_group!(
    benches,
    bench_cache_put,
    bench_cache_get,
    bench_ax_parse,
    bench_ax_validate,
    bench_config_merge
);
criterion_main!(benches);
