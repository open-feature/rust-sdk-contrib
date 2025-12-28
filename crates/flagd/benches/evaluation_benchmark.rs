use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use open_feature_flagd::in_process::*;
use serde_json::json;

fn create_test_flags() -> serde_json::Value {
    json!({
        "flags": {
            "simple-bool": {
                "state": "ENABLED",
                "variants": {
                    "on": true,
                    "off": false
                },
                "defaultVariant": "on"
            },
            "targeted-string": {
                "state": "ENABLED",
                "variants":  {
                    "variant-a": "A",
                    "variant-b": "B"
                },
                "defaultVariant": "variant-a",
                "targeting": {
                    "if": [
                        {"==": [{"var": "email"}, "user@example.com"]},
                        "variant-b",
                        null
                    ]
                }
            },
            "fractional-rollout": {
                "state":  "ENABLED",
                "variants": {
                    "red": "red",
                    "blue": "blue",
                    "green": "green"
                },
                "defaultVariant": "red",
                "targeting": {
                    "fractional":  [
                        {"var": "$flagd.flagKey"},
                        [["red", 25], ["blue", 25], ["green", 50]]
                    ]
                }
            }
        }
    })
}

fn benchmark_evaluations(c: &mut Criterion) {
    let mut group = c.benchmark_group("flag_evaluation");

    // Benchmark simple boolean flag
    group.bench_function("simple_bool", |b| {
        let flags = create_test_flags();
        // Initialize your resolver here with the flags
        b.iter(|| {
            // Evaluate the flag
            black_box(/* your evaluation call */);
        });
    });

    // Benchmark targeted evaluation
    group.bench_function("targeted_with_context", |b| {
        let flags = create_test_flags();
        let context = json!({"email": "user@example.com"});
        b.iter(|| {
            black_box(/* evaluation with context */);
        });
    });

    // Benchmark fractional evaluation
    group.bench_function("fractional_rollout", |b| {
        let flags = create_test_flags();
        b.iter(|| {
            black_box(/* fractional evaluation */);
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_evaluations);
criterion_main!(benches);