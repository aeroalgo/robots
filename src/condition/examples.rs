use crate::condition::{factory::ConditionFactory, types::ConditionInputData};
use crate::indicators::OHLCData;
use std::collections::HashMap;

/// Пример использования условия "выше другого вектора"
pub async fn above_condition_example() -> Result<(), String> {
    println!("=== Пример условия 'Above' ===");

    let condition = ConditionFactory::create_condition_default("Above")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;

    let data1 = vec![95.0, 98.0, 102.0, 105.0, 103.0, 108.0, 110.0];
    let data2 = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

    let result = condition
        .check(ConditionInputData::dual(&data1, &data2))
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

    let condition = ConditionFactory::create_condition_default("CrossesAbove")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;

    let line1 = vec![95.0, 98.0, 102.0, 105.0, 103.0, 108.0, 110.0];
    let line2 = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

    let result = condition
        .check(ConditionInputData::dual(&line1, &line2))
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

    let mut params = HashMap::new();
    params.insert("period".to_string(), 3.0);

    let condition = ConditionFactory::create_condition("RisingTrend", params)
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;

    let data = vec![100.0, 102.0, 105.0, 103.0, 108.0, 110.0, 112.0];

    let result = condition
        .check(ConditionInputData::single(&data))
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

    let above_condition = ConditionFactory::create_condition_default("Above")
        .map_err(|e| format!("Ошибка создания условия Above: {:?}", e))?;

    let trend_condition = ConditionFactory::create_condition_default("RisingTrend")
        .map_err(|e| format!("Ошибка создания условия RisingTrend: {:?}", e))?;

    let data = vec![95.0, 98.0, 102.0, 105.0, 103.0, 108.0, 110.0];
    let threshold = vec![100.0; data.len()];

    let above_result = above_condition
        .check(ConditionInputData::dual(&data, &threshold))
        .await
        .map_err(|e| format!("Ошибка проверки Above: {:?}", e))?;

    let trend_result = trend_condition
        .check(ConditionInputData::single(&data))
        .await
        .map_err(|e| format!("Ошибка проверки RisingTrend: {:?}", e))?;

    let combined_signals: Vec<bool> = above_result
        .signals
        .iter()
        .zip(trend_result.signals.iter())
        .map(|(a, b)| *a && *b)
        .collect();

    println!("Данные: {:?}", data);
    println!("Above сигналы: {:?}", above_result.signals);
    println!("Trend сигналы: {:?}", trend_result.signals);
    println!("Комбинированные сигналы (И): {:?}", combined_signals);

    Ok(())
}

/// Пример работы с OHLC данными
pub async fn ohlc_conditions_example() -> Result<(), String> {
    println!("\n=== Пример работы с OHLC данными ===");

    let ohlc_data = create_test_ohlc_data();
    let closes = ohlc_data.close.clone();
    let threshold = vec![102.0; closes.len()];

    let condition = ConditionFactory::create_condition_default("Above")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;

    let result = condition
        .check(ConditionInputData::dual(&closes, &threshold))
        .await
        .map_err(|e| format!("Ошибка проверки условия: {:?}", e))?;

    println!("OHLC данные:");
    println!("  Open: {:?}", ohlc_data.open);
    println!("  High: {:?}", ohlc_data.high);
    println!("  Low: {:?}", ohlc_data.low);
    println!("  Close: {:?}", ohlc_data.close);
    println!("Порог: {:?}", threshold);
    println!("Сигналы: {:?}", result.signals);

    Ok(())
}

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

    above_condition_example().await?;
    crosses_above_example().await?;
    rising_trend_example().await?;
    combined_conditions_example().await?;
    ohlc_conditions_example().await?;

    println!("\n{}", "=".repeat(50));

    println!("\n✅ Все примеры выполнены успешно!");
    Ok(())
}
