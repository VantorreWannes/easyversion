use std::fs;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use easyversion::{
    model::Id,
    operations::{Version, clean, save, split},
    store::FileStore,
};
use tempfile::tempdir;

fn bench_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("FileStore Operations");

    group.bench_function("set_100kb", |b| {
        b.iter_batched(
            || {
                let dir = tempdir().unwrap();
                let store = FileStore::new(dir.path()).unwrap();
                let data = vec![0u8; 100 * 1024];
                let id = Id { digest: 12345 };
                (dir, store, id, data)
            },
            |(_dir, store, id, data)| {
                store.set(id, &data).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("get_100kb", |b| {
        b.iter_batched(
            || {
                let dir = tempdir().unwrap();
                let store = FileStore::new(dir.path()).unwrap();
                let data = vec![0u8; 100 * 1024];
                let id = Id { digest: 12345 };
                store.set(id, &data).unwrap();
                (dir, store, id)
            },
            |(_dir, store, id)| {
                let _ = store.get(id).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("Save Operations");
    group.sample_size(50);

    group.bench_function("save_100_small_files", |b| {
        b.iter_batched(
            || {
                let dir = tempdir().unwrap();
                let data_store = FileStore::new(&dir.path().join("data")).unwrap();
                let history_store = FileStore::new(&dir.path().join("history")).unwrap();
                let workspace = dir.path().join("workspace");
                fs::create_dir_all(&workspace).unwrap();

                for i in 0..100 {
                    fs::write(workspace.join(format!("file_{}.txt", i)), b"test payload").unwrap();
                }

                (dir, data_store, history_store, workspace)
            },
            |(_dir, data_store, history_store, workspace)| {
                save(&data_store, &history_store, &workspace, None).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}
fn bench_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("Split Operations");
    group.sample_size(50);

    group.bench_function("split_100_small_files", |b| {
        b.iter_batched(
            || {
                let dir = tempdir().unwrap();
                let data_store = FileStore::new(&dir.path().join("data")).unwrap();
                let history_store = FileStore::new(&dir.path().join("history")).unwrap();
                let workspace = dir.path().join("workspace");
                fs::create_dir_all(&workspace).unwrap();

                for i in 0..100 {
                    fs::write(workspace.join(format!("file_{}.txt", i)), b"test payload").unwrap();
                }

                save(&data_store, &history_store, &workspace, None).unwrap();

                let target_dir = dir.path().join("target");
                (dir, data_store, history_store, workspace, target_dir)
            },
            |(_dir, data_store, history_store, workspace, target_dir)| {
                split(
                    &data_store,
                    &history_store,
                    &workspace,
                    &target_dir,
                    Version::Latest,
                )
                .unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_clean(c: &mut Criterion) {
    let mut group = c.benchmark_group("Clean Operations");

    group.bench_function("clean_100_orphaned_files", |b| {
        b.iter_batched(
            || {
                let dir = tempdir().unwrap();
                let data_store = FileStore::new(&dir.path().join("data")).unwrap();
                let history_store = FileStore::new(&dir.path().join("history")).unwrap();
                let workspace = dir.path().join("workspace");
                fs::create_dir_all(&workspace).unwrap();

                for i in 0..100 {
                    let id = Id { digest: i as u64 };
                    data_store.set(id, b"ghost payload").unwrap();
                }

                (dir, data_store, history_store, workspace)
            },
            |(_dir, data_store, history_store, workspace)| {
                clean(&data_store, &history_store, &workspace).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_store, bench_save, bench_split, bench_clean);
criterion_main!(benches);
