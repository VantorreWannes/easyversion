use criterion::Criterion;
use easyversion::easyversion::EasyVersion;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

const FILE_COUNT: usize = 100;
const FILE_SIZE_MB: usize = 10;
const FILE_SIZE_BYTES: usize = FILE_SIZE_MB * 1024 * 1024;

const TARGET_TIME_SECONDS: f64 = 15.0;
const SAMPLE_COUNT: usize = 50;

fn create_realistic_workspace(dir: &Path) {
    let data = vec![0u8; FILE_SIZE_BYTES];
    for i in 0..FILE_COUNT {
        let filename = format!("file_{}.dat", i);
        fs::write(dir.join(filename), &data).unwrap();
    }
}

fn setup_easyversion() -> (TempDir, TempDir, EasyVersion) {
    let config_dir = TempDir::new().unwrap();
    let data_dir = TempDir::new().unwrap();
    let easy_version = EasyVersion::new(config_dir.path(), data_dir.path());
    (config_dir, data_dir, easy_version)
}

fn setup_workspace_with_files() -> (TempDir, TempDir, TempDir, EasyVersion) {
    let (config_dir, data_dir, easy_version) = setup_easyversion();
    let workspace_dir = TempDir::new().unwrap();
    create_realistic_workspace(workspace_dir.path());
    (config_dir, data_dir, workspace_dir, easy_version)
}

fn setup_workspace_with_versions(count: usize) -> (TempDir, TempDir, TempDir, EasyVersion) {
    let (config_dir, data_dir, workspace_dir, easy_version) = setup_workspace_with_files();
    for i in 0..count {
        let comment = format!("version {}", i);
        easy_version
            .save(workspace_dir.path(), Some(comment.as_str()))
            .unwrap();
        let filename = format!("file_{}.dat", i % FILE_COUNT);
        fs::write(
            workspace_dir.path().join(filename),
            vec![i as u8; FILE_SIZE_BYTES],
        )
        .unwrap();
    }
    (config_dir, data_dir, workspace_dir, easy_version)
}

fn bench_new(c: &mut Criterion) {
    c.bench_function("EasyVersion::new", |b| {
        b.iter(|| {
            let config_dir = TempDir::new().unwrap();
            let data_dir = TempDir::new().unwrap();
            black_box(EasyVersion::new(config_dir.path(), data_dir.path()));
        })
    });
}

fn bench_save_initial(c: &mut Criterion) {
    c.bench_function("EasyVersion::save_initial", |b| {
        b.iter_batched(
            || setup_workspace_with_files(),
            |(_config_dir, _data_dir, workspace_dir, easy_version)| {
                black_box(
                    easy_version
                        .save(workspace_dir.path(), Some("initial save"))
                        .unwrap(),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_save_subsequent(c: &mut Criterion) {
    c.bench_function("EasyVersion::save_subsequent", |b| {
        b.iter_batched(
            || {
                let (config_dir, data_dir, workspace_dir, easy_version) =
                    setup_workspace_with_files();
                easy_version
                    .save(workspace_dir.path(), Some("initial"))
                    .unwrap();
                fs::write(
                    workspace_dir.path().join("file_0.dat"),
                    vec![1u8; FILE_SIZE_BYTES],
                )
                .unwrap();
                (config_dir, data_dir, workspace_dir, easy_version)
            },
            |(_config_dir, _data_dir, workspace_dir, easy_version)| {
                black_box(
                    easy_version
                        .save(workspace_dir.path(), Some("subsequent save"))
                        .unwrap(),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_list(c: &mut Criterion) {
    c.bench_function("EasyVersion::list", |b| {
        b.iter_batched(
            || setup_workspace_with_versions(5),
            |(_config_dir, _data_dir, workspace_dir, easy_version)| {
                black_box(easy_version.list(workspace_dir.path()).unwrap());
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_split(c: &mut Criterion) {
    c.bench_function("EasyVersion::split", |b| {
        b.iter_batched(
            || {
                let (config_dir, data_dir, source_workspace_dir, easy_version) =
                    setup_workspace_with_versions(3);
                let target_workspace_dir = TempDir::new().unwrap();
                (
                    config_dir,
                    data_dir,
                    source_workspace_dir,
                    target_workspace_dir,
                    easy_version,
                )
            },
            |(_config_dir, _data_dir, source_workspace_dir, target_workspace_dir, easy_version)| {
                black_box(
                    easy_version
                        .split(
                            source_workspace_dir.path(),
                            target_workspace_dir.path(),
                            None,
                            true,
                        )
                        .unwrap(),
                );
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_clean(c: &mut Criterion) {
    c.bench_function("EasyVersion::clean", |b| {
        b.iter_batched(
            || setup_workspace_with_versions(3),
            |(_config_dir, _data_dir, workspace_dir, easy_version)| {
                black_box(easy_version.clean(workspace_dir.path()).unwrap());
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn main() {
    let mut criterion = Criterion::default()
        .measurement_time(Duration::from_secs_f64(TARGET_TIME_SECONDS))
        .sample_size(SAMPLE_COUNT);

    bench_new(&mut criterion);
    bench_save_initial(&mut criterion);
    bench_save_subsequent(&mut criterion);
    bench_list(&mut criterion);
    bench_split(&mut criterion);
    bench_clean(&mut criterion);

    criterion.final_summary();
}
