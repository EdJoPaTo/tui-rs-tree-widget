use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[must_use]
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
            let _: Tree<_> = black_box(Tree::new(black_box(&items)));
        });
    });

    group.bench_function("example-items", |bencher| {
        bencher.iter(|| {
            let items = example_items();
            let _: Tree<_> = black_box(Tree::new(black_box(&items)));
        });
    });

    // While the metadata is quite big its only a reference with no executions -> fast
    #[cfg(feature = "json")]
    group.bench_function("metadata", |bencher| {
        let metadata = json::metadata();
        bencher.iter(|| {
            let _: Tree<_> = black_box(Tree::new(black_box(&metadata)));
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
        let mut state = TreeState::default();
        bencher.iter_batched(
            || Buffer::empty(buffer_size),
            |mut buffer| {
                Tree::new(black_box(&items)).render(
                    buffer_size,
                    black_box(&mut buffer),
                    black_box(&mut state),
                );
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("example-items", |bencher| {
        let items = example_items();
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        state.open(vec!["b", "d"]);
        bencher.iter_batched(
            || Buffer::empty(buffer_size),
            |mut buffer| {
                Tree::new(black_box(&items)).render(
                    buffer_size,
                    black_box(&mut buffer),
                    black_box(&mut state),
                );
            },
            BatchSize::SmallInput,
        );
    });

    #[cfg(feature = "json")]
    {
        let metadata = json::metadata();

        group.bench_function("metadata/no_open", |bencher| {
            let mut state = TreeState::default();
            bencher.iter_batched(
                || Buffer::empty(buffer_size),
                |mut buffer| {
                    Tree::new(black_box(&metadata)).render(
                        buffer_size,
                        black_box(&mut buffer),
                        black_box(&mut state),
                    );
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("metadata/few_open", |bencher| {
            use json::{index, key};
            let mut state = TreeState::default();
            state.open(vec![key("packages")]);
            state.open(vec![key("packages"), index(0)]);
            state.open(vec![key("resolve")]);
            state.open(vec![key("resolve"), key("nodes")]);
            state.open(vec![key("resolve"), key("nodes"), index(0)]);
            bencher.iter_batched(
                || Buffer::empty(buffer_size),
                |mut buffer| {
                    Tree::new(black_box(&metadata)).render(
                        buffer_size,
                        black_box(&mut buffer),
                        black_box(&mut state),
                    );
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function("metadata/all_open", |bencher| {
            let mut state = TreeState::default();
            json::open_all(&mut state, &metadata, &[]);
            bencher.iter_batched(
                || Buffer::empty(buffer_size),
                |mut buffer| {
                    Tree::new(black_box(&metadata)).render(
                        buffer_size,
                        black_box(&mut buffer),
                        black_box(&mut state),
                    );
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Helpers for json related benches
#[cfg(feature = "json")]
mod json {
    use tui_tree_widget::third_party::json::Selector;
    use tui_tree_widget::TreeState;

    pub fn key(key: &str) -> Selector {
        Selector::Key(key.to_owned())
    }

    pub const fn index(index: usize) -> Selector {
        Selector::Index(index)
    }

    pub fn metadata() -> serde_json::Value {
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

    pub fn open_all(
        state: &mut TreeState<Vec<Selector>>,
        json: &serde_json::Value,
        selector: &[Selector],
    ) {
        match json {
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => {}
            serde_json::Value::Array(array) if array.is_empty() => {}
            serde_json::Value::Array(array) => {
                state.open(selector.to_vec());
                for (index, value) in array.iter().enumerate() {
                    let mut child_selector = selector.to_vec();
                    child_selector.push(self::index(index));
                    open_all(state, value, &child_selector);
                }
            }
            serde_json::Value::Object(object) if object.is_empty() => {}
            serde_json::Value::Object(object) => {
                state.open(selector.to_vec());
                for (key, value) in object {
                    let mut child_selector = selector.to_vec();
                    child_selector.push(self::key(key));
                    open_all(state, value, &child_selector);
                }
            }
        }
    }
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
