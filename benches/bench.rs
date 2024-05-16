use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use tui_tree_widget::{Tree, TreeItem, TreeState};

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

fn init(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("init");
    group.throughput(Throughput::Elements(1)); // Frames per second

    group.bench_function("empty", |bencher| {
        bencher.iter(|| {
            let items: Vec<TreeItem<usize>> = vec![];
            black_box(Tree::new(black_box(&items))).unwrap();
        });
    });

    group.bench_function("example-items", |bencher| {
        bencher.iter(|| {
            let items = example_items();
            black_box(Tree::new(black_box(&items))).unwrap();
        });
    });

    group.finish();
}

fn renders(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("render");
    group.throughput(Throughput::Elements(1)); // Frames per second

    let buffer_size = Rect::new(0, 0, 100, 100);

    group.bench_function("empty", |bencher| {
        let items: Vec<TreeItem<usize>> = vec![];
        let tree = Tree::new(&items).unwrap();
        let mut state = TreeState::default();
        bencher.iter_batched(
            || (tree.clone(), Buffer::empty(buffer_size)),
            |(tree, mut buffer)| {
                black_box(tree).render(buffer_size, black_box(&mut buffer), black_box(&mut state));
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("example-items", |bencher| {
        let items = example_items();
        let tree = Tree::new(&items).unwrap();
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        state.open(vec!["b", "d"]);
        bencher.iter_batched(
            || (tree.clone(), Buffer::empty(buffer_size)),
            |(tree, mut buffer)| {
                black_box(tree).render(buffer_size, black_box(&mut buffer), black_box(&mut state));
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
