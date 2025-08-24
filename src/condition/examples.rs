use crate::condition::{base::Condition, factory::ConditionFactory, types::OHLCData};
use std::collections::HashMap;

/// Пример использования условия "выше другого вектора"
pub async fn above_condition_example() -> Result<(), String> {
    println!("=== Пример условия 'Above' ===");

    // Создаем условие
    let condition = ConditionFactory::create_condition_default("Above")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;

    // Тестовые данные - два вектора для сравнения
    let data1 = vec![95.0, 98.0, 102.0, 105.0, 103.0, 108.0, 110.0];
    let data2 = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

    // Проверяем условие на двух векторах
    let result = condition
        .check_dual(&data1, &data2)
        .await
        .map_err(|e| format!("Ошибка проверки условия: {:?}", e))?;

    println!("Вектор 1: {:?}", data1);
    println!("Вектор 2: {:?}", data2);
    println!("Сигналы: {:?}", result.signals);
    println!("Силы сигналов: {:?}", result.strengths);
    println!("Направления: {:?}", result.directions);

    Ok(())
}

/// Пример использования условия "пересечение выше"
pub async fn crosses_above_example() -> Result<(), String> {
    println!("\n=== Пример условия 'CrossesAbove' ===");

    // Создаем условие
    let condition = ConditionFactory::create_condition_default("CrossesAbove")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;

    // Тестовые данные - две линии
    let line1 = vec![95.0, 98.0, 102.0, 105.0, 103.0, 108.0, 110.0];
    let line2 = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

    // Проверяем условие на двух линиях
    let result = condition
        .check_dual(&line1, &line2)
        .await
        .map_err(|e| format!("Ошибка проверки условия: {:?}", e))?;

    println!("Линия 1: {:?}", line1);
    println!("Линия 2: {:?}", line2);
    println!("Сигналы пересечения: {:?}", result.signals);
    println!("Силы сигналов: {:?}", result.strengths);

    Ok(())
}

/// Пример использования трендового условия
pub async fn rising_trend_example() -> Result<(), String> {
    println!("\n=== Пример условия 'RisingTrend' ===");

    // Создаем условие
    let mut params = HashMap::new();
    params.insert("period".to_string(), 3.0);

    let condition = ConditionFactory::create_condition("RisingTrend", params)
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;

    // Тестовые данные
    let data = vec![100.0, 102.0, 105.0, 103.0, 108.0, 110.0, 112.0];

    // Проверяем условие
    let result = condition
        .check_simple(&data)
        .await
        .map_err(|e| format!("Ошибка проверки условия: {:?}", e))?;

    println!("Данные: {:?}", data);
    println!("Период тренда: 3");
    println!("Сигналы тренда: {:?}", result.signals);
    println!("Силы сигналов: {:?}", result.strengths);
    println!("Направления: {:?}", result.directions);

    Ok(())
}

/// Пример комбинирования условий
pub async fn combined_conditions_example() -> Result<(), String> {
    println!("\n=== Пример комбинирования условий ===");

    // Создаем несколько условий
    let above_condition = ConditionFactory::create_condition_default("Above")
        .map_err(|e| format!("Ошибка создания условия Above: {:?}", e))?;

    let trend_condition = ConditionFactory::create_condition_default("RisingTrend")
        .map_err(|e| format!("Ошибка создания условия RisingTrend: {:?}", e))?;

    // Тестовые данные
    let data = vec![95.0, 98.0, 102.0, 105.0, 103.0, 108.0, 110.0];

    // Проверяем первое условие
    let above_result = above_condition
        .check_simple(&data)
        .await
        .map_err(|e| format!("Ошибка проверки Above: {:?}", e))?;

    // Проверяем второе условие
    let trend_result = trend_condition
        .check_simple(&data)
        .await
        .map_err(|e| format!("Ошибка проверки RisingTrend: {:?}", e))?;

    // Комбинируем результаты (логическое И)
    let mut combined_signals = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        let combined = above_result.signals[i] && trend_result.signals[i];
        combined_signals.push(combined);
    }

    println!("Данные: {:?}", data);
    println!("Above сигналы: {:?}", above_result.signals);
    println!("Trend сигналы: {:?}", trend_result.signals);
    println!("Комбинированные сигналы (И): {:?}", combined_signals);

    Ok(())
}

/// Пример работы с OHLC данными
pub async fn ohlc_conditions_example() -> Result<(), String> {
    println!("\n=== Пример работы с OHLC данными ===");

    // Создаем тестовые OHLC данные
    let ohlc_data = create_test_ohlc_data();

    // Создаем условие
    let condition = ConditionFactory::create_condition_default("Above")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;

    // Проверяем условие на OHLC данных
    let result = condition
        .check_ohlc(&ohlc_data)
        .await
        .map_err(|e| format!("Ошибка проверки условия: {:?}", e))?;

    println!("OHLC данные:");
    println!("  Open: {:?}", ohlc_data.open);
    println!("  High: {:?}", ohlc_data.high);
    println!("  Low: {:?}", ohlc_data.low);
    println!("  Close: {:?}", ohlc_data.close);
    println!("Сигналы (на основе Close): {:?}", result.signals);

    Ok(())
}

/// Создание тестовых OHLC данных
fn create_test_ohlc_data() -> OHLCData {
    let open = vec![100.0, 101.0, 102.0, 103.0, 104.0];
    let high = vec![101.0, 102.0, 103.0, 104.0, 105.0];
    let low = vec![99.0, 100.0, 101.0, 102.0, 103.0];
    let close = vec![100.5, 101.5, 102.5, 103.5, 104.5];

    OHLCData::new(open, high, low, close)
}

/// Запуск всех примеров
pub async fn run_all_examples() -> Result<(), String> {
    println!("🚀 Запуск примеров системы условий\n");

    above_condition_example().await?;
    crosses_above_example().await?;
    rising_trend_example().await?;
    combined_conditions_example().await?;
    ohlc_conditions_example().await?;

    println!("\n✅ Все примеры выполнены успешно!");
    Ok(())
}

/// Запуск всех примеров включая интеграцию
pub async fn run_all_examples_with_integration() -> Result<(), String> {
    println!("🚀 Запуск всех примеров системы условий\n");

    // Базовые примеры
    above_condition_example().await?;
    crosses_above_example().await?;
    rising_trend_example().await?;
    combined_conditions_example().await?;
    ohlc_conditions_example().await?;

    println!("\n" + "=".repeat(50));

    // Примеры интеграции с индикаторами
    use crate::condition::integration_example::run_integration_examples;
    run_integration_examples().await?;

    println!("\n✅ Все примеры выполнены успешно!");
    Ok(())
}
