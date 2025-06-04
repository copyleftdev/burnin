use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn cpu_workload_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_workloads");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("floating_point", |b| {
        b.iter(|| {
            let mut sum = 0.0f64;
            for i in 0..1000 {
                sum += black_box((i as f64).sqrt() * 2.718281828);
            }
            black_box(sum)
        });
    });
    
    group.bench_function("integer_arithmetic", |b| {
        b.iter(|| {
            let mut result = 1u64;
            for i in 1..100 {
                result = black_box(result.wrapping_mul(i).wrapping_add(i * i));
            }
            black_box(result)
        });
    });
    
    group.bench_function("matrix_multiply_small", |bench| {
        let size = 10;
        let a = vec![vec![1.0f64; size]; size];
        let b = vec![vec![2.0f64; size]; size];
        
        bench.iter(|| {
            let mut c = vec![vec![0.0f64; size]; size];
            for i in 0..size {
                for j in 0..size {
                    for k in 0..size {
                        c[i][j] += a[i][k] * b[k][j];
                    }
                }
            }
            black_box(c)
        });
    });
    
    group.finish();
}

fn memory_access_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_access");
    
    group.bench_function("sequential_read", |b| {
        let data = vec![0u8; 1_000_000];
        b.iter(|| {
            let mut sum = 0u64;
            for &byte in data.iter() {
                sum += byte as u64;
            }
            black_box(sum)
        });
    });
    
    group.bench_function("random_access", |b| {
        let data = vec![0u8; 1_000_000];
        let indices: Vec<usize> = (0..1000).map(|i| (i * 997) % data.len()).collect();
        
        b.iter(|| {
            let mut sum = 0u64;
            for &idx in indices.iter() {
                sum += data[idx] as u64;
            }
            black_box(sum)
        });
    });
    
    group.finish();
}

criterion_group!(benches, cpu_workload_benchmark, memory_access_benchmark);
criterion_main!(benches);
