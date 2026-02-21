use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use stoa::db;
use stoa::diff;
use stoa::export;
use stoa::model::{ChatMessage, Conversation, Role};

fn make_conversation(msg_count: usize) -> Conversation {
    let mut conv = Conversation::new();
    conv.title = format!("Bench conversation with {msg_count} messages");
    for i in 0..msg_count {
        let role = if i % 2 == 0 { Role::User } else { Role::Assistant };
        conv.messages.push(ChatMessage {
            role,
            content: format!(
                "This is message number {i} with some content to make it realistic \
                 enough for benchmarking purposes. The quick brown fox jumps over the lazy dog."
            ),
            streaming: false,
            model: if i % 2 == 1 { Some("gpt-4.1".to_string()) } else { None },
            token_count: Some(50),
            rating: 0,
            latency_ms: if i % 2 == 1 { Some(250) } else { None },
            images: Vec::new(),
        });
    }
    conv
}

fn generate_text(word_count: usize) -> String {
    let words = [
        "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog", "a", "big", "red", "cat",
        "sits", "on", "warm", "mat",
    ];
    (0..word_count)
        .map(|i| words[i % words.len()])
        .collect::<Vec<_>>()
        .join(" ")
}

fn bench_db_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_save");
    for count in [1, 10, 50, 100] {
        let conv = make_conversation(count);
        group.bench_with_input(
            BenchmarkId::new("save_conversation", count),
            &conv,
            |b, conv| {
                let conn = db::open_in_memory();
                b.iter(|| db::save_conversation(&conn, conv).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_db_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_load");
    for count in [10, 50, 200] {
        group.bench_with_input(BenchmarkId::new("load_all", count), &count, |b, &count| {
            let conn = db::open_in_memory();
            for _ in 0..count {
                let conv = make_conversation(5);
                db::save_conversation(&conn, &conv).unwrap();
            }
            b.iter(|| db::load_all(&conn));
        });
    }
    group.finish();
}

fn bench_db_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_search");
    let conn = db::open_in_memory();
    for i in 0..50 {
        let mut conv = make_conversation(10);
        conv.title = format!("Topic number {i} about various things");
        if i == 25 {
            conv.messages[0].content = "unique_search_target_word appears here".to_string();
        }
        db::save_conversation(&conn, &conv).unwrap();
    }
    group.bench_function("fts5_hit", |b| {
        b.iter(|| db::search_conversations(&conn, "unique_search_target_word"));
    });
    group.bench_function("like_fallback", |b| {
        b.iter(|| db::search_conversations(&conn, "message number"));
    });
    group.bench_function("miss", |b| {
        b.iter(|| db::search_conversations(&conn, "zzzznonexistent"));
    });
    group.finish();
}

fn bench_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff");
    for count in [10, 100, 500, 1999] {
        let a = generate_text(count);
        let words_b: Vec<String> = a
            .split_whitespace()
            .enumerate()
            .map(|(i, w)| {
                if i % 5 == 0 {
                    "CHANGED".to_string()
                } else {
                    w.to_string()
                }
            })
            .collect();
        let b = words_b.join(" ");
        group.bench_with_input(
            BenchmarkId::new("word_diff", count),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| diff::word_diff(a, b));
            },
        );
    }
    group.finish();
}

fn bench_diff_agreement(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_agreement");
    for count in [10, 100, 500, 1999] {
        let a = generate_text(count);
        let words_b: Vec<String> = a
            .split_whitespace()
            .enumerate()
            .map(|(i, w)| {
                if i % 5 == 0 {
                    "CHANGED".to_string()
                } else {
                    w.to_string()
                }
            })
            .collect();
        let b = words_b.join(" ");
        group.bench_with_input(
            BenchmarkId::new("agreement_percentage", count),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| diff::agreement_percentage(a, b));
            },
        );
    }
    group.finish();
}

fn bench_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("export");
    for count in [10, 50] {
        let conv = make_conversation(count);
        group.bench_with_input(
            BenchmarkId::new("to_markdown", count),
            &conv,
            |b, conv| {
                b.iter(|| export::conversation_to_markdown(conv));
            },
        );
        group.bench_with_input(BenchmarkId::new("to_json", count), &conv, |b, conv| {
            b.iter(|| export::conversation_to_json(conv));
        });
        group.bench_with_input(BenchmarkId::new("to_html", count), &conv, |b, conv| {
            b.iter(|| export::conversation_to_html(conv));
        });
    }
    group.finish();
}

fn bench_model_fork(c: &mut Criterion) {
    let mut group = c.benchmark_group("model_fork");
    for count in [10, 100] {
        let conv = make_conversation(count);
        group.bench_with_input(BenchmarkId::new("fork", count), &conv, |b, conv| {
            b.iter(|| conv.fork(conv.messages.len().saturating_sub(1)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_db_save,
    bench_db_load,
    bench_db_search,
    bench_diff,
    bench_diff_agreement,
    bench_export,
    bench_model_fork,
);
criterion_main!(benches);
