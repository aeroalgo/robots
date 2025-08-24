use crate::app::charts::crud::TickerRepository;
use crate::core::agt::candles::source::Source;
use crate::core::agt::indicators::any::SimpleIndicators;
use crate::core::agt::opt::iterating::indicators_example::IndicatorsExample;
use crate::core::agt::strategy::{
    example_usage::ConditionExample, memory_analyzer::MemoryAnalyzer,
};
use app::charts::model::TickerCandle;
mod app;
mod core;
mod indicators;
mod condition;

#[tokio::main]
async fn main() {
    println!("=== Демонстрация оптимизированного модуля condition ===");

    // Демонстрация базового использования
    println!("\n1. Базовое использование StrategyCondition:");
    ConditionExample::demonstrate_basic_usage().await;

    // Демонстрация оптимизированного использования
    println!("\n2. Оптимизированное использование:");
    ConditionExample::demonstrate_optimized_usage();

    // Демонстрация работы с несколькими условиями
    println!("\n3. Работа с несколькими условиями:");
    ConditionExample::demonstrate_multiple_conditions();

    // Демонстрация работы с пересечениями
    println!("\n4. Работа с пересечениями сигналов:");
    ConditionExample::demonstrate_crossings();

    // Сравнение производительности
    println!("\n5. Сравнение производительности:");
    ConditionExample::demonstrate_performance_comparison().await;

    // Анализ использования памяти
    println!("\n6. Анализ использования памяти:");
    let memory_comparison = MemoryAnalyzer::compare_memory_usage();
    println!("Сравнение памяти: {:?}", memory_comparison);

    let data_analysis = MemoryAnalyzer::analyze_data_set_memory_usage(1000, 1000);
    println!("Анализ для 1000 элементов: {:?}", data_analysis);

    let recommendations = MemoryAnalyzer::get_memory_optimization_recommendations();
    println!("Рекомендации по оптимизации:");
    for (i, recommendation) in recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, recommendation);
    }

    println!("\n=== Демонстрация завершена ===");

    // Демонстрация работы с SimpleCombinationIndicators
    println!("\n=== Демонстрация SimpleCombinationIndicators ===");

    // Базовое использование
    IndicatorsExample::demonstrate_basic_execute();

    // Фильтрованное использование
    IndicatorsExample::demonstrate_filtered_execute();

    // Уникальные комбинации
    IndicatorsExample::demonstrate_unique_execute();

    // Сбалансированные комбинации
    IndicatorsExample::demonstrate_balanced_execute();

    // Статистика
    IndicatorsExample::demonstrate_stats();

    // Сравнение производительности
    IndicatorsExample::demonstrate_performance_comparison();

    // Практическое использование в контексте торгового робота
    IndicatorsExample::demonstrate_trading_context();

    println!("\n=== Демонстрация SimpleCombinationIndicators завершена ===");

    // Оригинальный код для работы с данными
    let ticker = TickerRepository::new("charts".to_string(), "ALRS.MM".to_string()).await;
    let all_data: Vec<TickerCandle> = ticker.get_all().await;
    let source_data = Source::new(all_data).await;

    let mut x = SimpleIndicators::new(source_data.close.clone()).await;
    let data_rsi = x.get_rsi(20.0, false).await;
    let mut x = SimpleIndicators::new(data_rsi.data.clone()).await;
    let _data_sp = x.get_super_trend(20.0, 3.1, false).await; // Префикс _ для неиспользуемой переменной
    println!("RSI данные: {:?}", data_rsi.data);
}
