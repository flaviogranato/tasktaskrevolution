use criterion::{black_box, criterion_group, criterion_main, Criterion};
use TaskTaskRevolution::domain::task::Task;
use TaskTaskRevolution::application::task::TaskService;

fn task_creation_benchmark(c: &mut Criterion) {
    c.bench_function("criar_tarefa", |b| {
        b.iter(|| {
            let task = Task::new(
                black_box("Tarefa de teste".to_string()),
                black_box("Descrição da tarefa".to_string()),
                black_box(chrono::Utc::now().naive_utc()),
            );
            black_box(task);
        })
    });
}

fn task_validation_benchmark(c: &mut Criterion) {
    let task = Task::new(
        "Tarefa de teste".to_string(),
        "Descrição da tarefa".to_string(),
        chrono::Utc::now().naive_utc(),
    );

    c.bench_function("validar_tarefa", |b| {
        b.iter(|| {
            black_box(task.validate_domain());
        })
    });
}

criterion_group!(benches, task_creation_benchmark, task_validation_benchmark);
criterion_main!(benches); 