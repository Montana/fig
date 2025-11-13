use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fig::FigBuf;
use std::sync::Arc;

fn bench_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("creation");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("figbuf_from_vec", size), &size, |b, &size| {
            let data: Vec<u64> = (0..size).collect();
            b.iter(|| {
                let buf = FigBuf::from_vec(black_box(data.clone()));
                black_box(buf)
            });
        });

        group.bench_with_input(BenchmarkId::new("arc_from_vec", size), &size, |b, &size| {
            let data: Vec<u64> = (0..size).collect();
            b.iter(|| {
                let arc: Arc<[u64]> = Arc::from(black_box(data.clone()));
                black_box(arc)
            });
        });
    }

    group.finish();
}

fn bench_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("cloning");

    for size in [100, 1000, 10000] {
        let data: Vec<u64> = (0..size).collect();
        let figbuf = FigBuf::from_vec(data.clone());
        let arc: Arc<[u64]> = Arc::from(data);

        group.bench_with_input(BenchmarkId::new("figbuf_clone", size), &figbuf, |b, buf| {
            b.iter(|| {
                let clone = black_box(buf.clone());
                black_box(clone)
            });
        });

        group.bench_with_input(BenchmarkId::new("arc_clone", size), &arc, |b, arc| {
            b.iter(|| {
                let clone = black_box(Arc::clone(arc));
                black_box(clone)
            });
        });
    }

    group.finish();
}

fn bench_slicing(c: &mut Criterion) {
    let mut group = c.benchmark_group("slicing");

    let data: Vec<u64> = (0..10000).collect();
    let figbuf = FigBuf::from_vec(data);

    group.bench_function("figbuf_single_slice", |b| {
        b.iter(|| {
            let slice = black_box(figbuf.slice(1000..5000));
            black_box(slice)
        });
    });

    group.bench_function("figbuf_nested_slices", |b| {
        b.iter(|| {
            let slice1 = figbuf.slice(1000..9000);
            let slice2 = slice1.slice(500..4000);
            let slice3 = slice2.slice(100..2000);
            black_box(slice3)
        });
    });

    group.bench_function("figbuf_multiple_slices", |b| {
        b.iter(|| {
            let slices: Vec<_> = (0..10)
                .map(|i| figbuf.slice(i * 1000..(i + 1) * 1000))
                .collect();
            black_box(slices)
        });
    });

    group.finish();
}

fn bench_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("access");

    let data: Vec<u64> = (0..10000).collect();
    let figbuf = FigBuf::from_vec(data.clone());
    let arc: Arc<[u64]> = Arc::from(data);

    group.bench_function("figbuf_iteration", |b| {
        b.iter(|| {
            let sum: u64 = figbuf.iter().sum();
            black_box(sum)
        });
    });

    group.bench_function("arc_iteration", |b| {
        b.iter(|| {
            let sum: u64 = arc.iter().sum();
            black_box(sum)
        });
    });

    group.bench_function("figbuf_index_access", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for i in 0..figbuf.len() {
                sum += figbuf[i];
            }
            black_box(sum)
        });
    });

    group.finish();
}

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    let text = "Hello, World! This is a test string for benchmarking. ".repeat(100);
    let figbuf = FigBuf::from_string(text.clone());
    let arc: Arc<str> = Arc::from(text.as_str());

    group.bench_function("figbuf_string_clone", |b| {
        b.iter(|| {
            let clone = black_box(figbuf.clone());
            black_box(clone)
        });
    });

    group.bench_function("arc_string_clone", |b| {
        b.iter(|| {
            let clone = black_box(Arc::clone(&arc));
            black_box(clone)
        });
    });

    group.bench_function("figbuf_string_slice", |b| {
        b.iter(|| {
            let slice = black_box(figbuf.slice(0..100));
            black_box(slice)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_creation,
    bench_cloning,
    bench_slicing,
    bench_access,
    bench_string_operations
);
criterion_main!(benches);