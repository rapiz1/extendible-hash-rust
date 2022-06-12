use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

type K = u64;
type V = u64;
const SET_SIZE: usize = 1 << 10;
fn create_workload() -> Vec<(K, V)> {
    let mut v = vec![];
    v.resize(SET_SIZE, (0, 0));
    for i in 1..SET_SIZE {
        v[i] = (i as K, i as V);
    }
    v
}

fn stress_crate(workload: &Vec<(K, V)>) {
    let mut m = extendible_hash::HashTable::new();
    for (k, v) in workload {
        m.put(k, v);
    }
    for (k, _) in workload {
        m.delete(&k);
    }
}
fn stress_std(workload: &Vec<(K, V)>) {
    let mut m = HashMap::new();
    for (k, v) in workload {
        m.insert(k, v);
    }
    for (k, _) in workload {
        m.remove(&k);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let workload = create_workload();
    c.bench_function("crate", |b| b.iter(|| stress_crate(black_box(&workload))));
    c.bench_function("std", |b| b.iter(|| stress_std(black_box(&workload))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
