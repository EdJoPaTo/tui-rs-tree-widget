use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use tui_tree_widget::{Selector, Tree, TreeItem, TreeState};

fn example_items() -> Vec<TreeItem<'static, &'static str>> {
    vec![
        TreeItem::new_leaf("a", "Alfa"),
        TreeItem::new(
            "b",
            "Bravo",
            vec![
                TreeItem::new_leaf("c", "Charlie"),
                TreeItem::new(
                    "d",
                    "Delta",
                    vec![
                        TreeItem::new_leaf("e", "Echo"),
                        TreeItem::new_leaf("f", "Foxtrot"),
                    ],
                )
                .expect("all item identifiers are unique"),
                TreeItem::new_leaf("g", "Golf"),
            ],
        )
        .expect("all item identifiers are unique"),
        TreeItem::new_leaf("h", "Hotel"),
        TreeItem::new(
            "i",
            "India",
            vec![
                TreeItem::new_leaf("j", "Juliett"),
                TreeItem::new_leaf("k", "Kilo"),
                TreeItem::new_leaf("l", "Lima"),
                TreeItem::new_leaf("m", "Mike"),
                TreeItem::new_leaf("n", "November"),
            ],
        )
        .expect("all item identifiers are unique"),
        TreeItem::new_leaf("o", "Oscar"),
        TreeItem::new(
            "p",
            "Papa",
            vec![
                TreeItem::new_leaf("q", "Quebec"),
                TreeItem::new_leaf("r", "Romeo"),
                TreeItem::new_leaf("s", "Sierra"),
                TreeItem::new_leaf("t", "Tango"),
                TreeItem::new_leaf("u", "Uniform"),
                TreeItem::new(
                    "v",
                    "Victor",
                    vec![
                        TreeItem::new_leaf("w", "Whiskey"),
                        TreeItem::new_leaf("x", "Xray"),
                        TreeItem::new_leaf("y", "Yankee"),
                    ],
                )
                .expect("all item identifiers are unique"),
            ],
        )
        .expect("all item identifiers are unique"),
        TreeItem::new_leaf("z", "Zulu"),
    ]
}

fn metadata() -> serde_json::Value {
    let output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--all-features")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "cargo metadata should be executed successfully"
    );
    let stdout = String::from_utf8(output.stdout).expect("Should be able to parse metadata");
    let metadata: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    metadata
}

fn key(key: &str) -> Selector {
    Selector::ObjectKey(key.to_owned())
}

fn init(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("init");
    group.throughput(Throughput::Elements(1)); // Frames per second

    group.bench_function("empty", |bencher| {
        bencher.iter(|| {
            black_box(Tree::<usize>::new(black_box(vec![])).unwrap());
        });
    });

    group.bench_function("example-items", |bencher| {
        bencher.iter(|| {
            black_box(Tree::new(black_box(example_items())).unwrap());
        });
    });

    let metadata = metadata();
    group.bench_function("metadata", |bencher| {
        bencher.iter(|| {
            black_box(Tree::new(tui_tree_widget::json::tree_items(black_box(&metadata))).unwrap());
        });
    });

    group.finish();
}

fn renders(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("render");
    group.throughput(Throughput::Elements(1)); // Frames per second

    let buffer_size = Rect::new(0, 0, 100, 100);

    let tree = Tree::new(vec![]).unwrap();
    group.bench_function("empty", |bencher| {
        bencher.iter_batched(
            || (tree.clone(), TreeState::<usize>::default()),
            |(tree, mut state)| {
                let mut buffer = Buffer::empty(buffer_size);
                black_box(tree).render(buffer_size, black_box(&mut buffer), &mut state);
            },
            BatchSize::SmallInput,
        );
    });

    let tree = Tree::new(example_items()).unwrap();
    group.bench_function("example-items", |bencher| {
        bencher.iter_batched(
            || {
                let mut state = TreeState::default();
                state.open(vec!["b"]);
                state.open(vec!["b", "d"]);
                (tree.clone(), state)
            },
            |(tree, mut state)| {
                let mut buffer = Buffer::empty(buffer_size);
                black_box(tree).render(buffer_size, black_box(&mut buffer), &mut state);
            },
            BatchSize::SmallInput,
        );
    });

    let metadata = metadata();
    let tree = Tree::new(tui_tree_widget::json::tree_items(&metadata)).unwrap();
    group.bench_function("metadata", |bencher| {
        bencher.iter_batched(
            || {
                let mut state = TreeState::default();
                state.open(vec![key("packages")]);
                state.open(vec![key("packages"), Selector::ArrayIndex(0)]);
                state.open(vec![key("resolve")]);
                state.open(vec![key("resolve"), key("nodes")]);
                state.open(vec![key("resolve"), key("nodes"), Selector::ArrayIndex(0)]);
                (tree.clone(), state)
            },
            |(tree, mut state)| {
                let mut buffer = Buffer::empty(buffer_size);
                black_box(tree).render(buffer_size, black_box(&mut buffer), &mut state);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Create flamegraphs with `cargo bench --bench bench -- --profile-time=5`
#[cfg(unix)]
fn profiled() -> Criterion {
    use pprof::criterion::{Output, PProfProfiler};
    Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)))
}
#[cfg(not(unix))]
fn profiled() -> Criterion {
    Criterion::default()
}

criterion_group! {
    name = benches;
    config = profiled();
    targets = init, renders
}
criterion_main!(benches);
