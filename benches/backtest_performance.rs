use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use robots::strategy::executor::BacktestExecutor;
use robots::strategy::types::StrategyDefinition;
use robots::data_model::types::{TimeFrame, Symbol};
use robots::data_model::quote_frame::QuoteFrame;
use std::collections::HashMap;
use std::time::Instant;

/// Генерирует тестовые данные для backtest
fn generate_test_data(bars: usize) -> QuoteFrame {
    let mut frame = QuoteFrame::new(
        Symbol::new("TEST", "TEST"),
        TimeFrame::Minutes(60),
    );
    
    // Генерируем простые тестовые данные
    for i in 0..bars {
        let base_price = 100.0 + (i as f32 * 0.1);
        frame.add_bar(
            base_price,
            base_price + 1.0,
            base_price - 1.0,
            base_price + 0.5,
            1000.0,
        ).unwrap();
    }
    
    frame
}

/// Бенчмарк для измерения производительности backtest
fn bench_backtest_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("backtest_performance");
    
    // Тестируем разные размеры данных
    for bars in [100, 1000, 5000, 10000].iter() {
        let data = generate_test_data(*bars);
        let mut frames = HashMap::new();
        frames.insert(TimeFrame::Minutes(60), data);
        
        // Создаем простую стратегию (нужно будет создать тестовую стратегию)
        // Пока просто измеряем время создания executor
        
        group.bench_with_input(
            BenchmarkId::from_parameter(bars),
            bars,
            |b, _| {
                b.iter(|| {
                    // TODO: Создать тестовую стратегию и выполнить backtest
                    // let executor = BacktestExecutor::from_definition(...);
                    // executor.run_backtest()
                    black_box(bars)
                });
            },
        );
    }
    
    group.finish();
}

/// Бенчмарк для измерения времени конвертации стратегии
fn bench_strategy_conversion(c: &mut Criterion) {
    c.bench_function("strategy_conversion", |b| {
        // TODO: Создать StrategyCandidate и измерить время конвертации
        b.iter(|| {
            black_box(1)
        });
    });
}

criterion_group!(benches, bench_backtest_performance, bench_strategy_conversion);
criterion_main!(benches);



