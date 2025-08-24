use crate::indicators::{
    base::Indicator,
    implementations::{
        BBLower, BBMiddle, BBUpper, KCLower, KCMiddle, KCUpper, Stochastic, SuperTrend,
    },
    types::OHLCData,
};

/// Пример использования Bollinger Bands компонентов
pub async fn bollinger_bands_example() -> Result<(), String> {
    // Создаем тестовые OHLC данные
    let ohlc_data = create_test_ohlc_data();

    // Создаем компоненты Bollinger Bands
    let bb_middle = BBMiddle::new(20.0, 2.0).map_err(|e| e.to_string())?;
    let bb_upper = BBUpper::new(20.0, 2.0).map_err(|e| e.to_string())?;
    let bb_lower = BBLower::new(20.0, 2.0).map_err(|e| e.to_string())?;

    // Вычисляем значения
    let middle_values = bb_middle
        .calculate_ohlc(&ohlc_data)
        .await
        .map_err(|e| e.to_string())?;
    let upper_values = bb_upper
        .calculate_ohlc(&ohlc_data)
        .await
        .map_err(|e| e.to_string())?;
    let lower_values = bb_lower
        .calculate_ohlc(&ohlc_data)
        .await
        .map_err(|e| e.to_string())?;

    // Выводим результаты
    println!("Bollinger Bands Example:");
    println!(
        "Middle (SMA): {:?}",
        &middle_values[middle_values.len() - 5..]
    );
    println!("Upper: {:?}", &upper_values[upper_values.len() - 5..]);
    println!("Lower: {:?}", &lower_values[lower_values.len() - 5..]);

    Ok(())
}

/// Пример использования Keltner Channel компонентов
pub async fn keltner_channel_example() -> Result<(), String> {
    // Создаем тестовые OHLC данные
    let ohlc_data = create_test_ohlc_data();

    // Создаем компоненты Keltner Channel
    let kc_middle = KCMiddle::new(20.0).map_err(|e| e.to_string())?;
    let kc_upper = KCUpper::new(20.0, 10.0, 2.0).map_err(|e| e.to_string())?;
    let kc_lower = KCLower::new(20.0, 10.0, 2.0).map_err(|e| e.to_string())?;

    // Вычисляем значения
    let middle_values = kc_middle
        .calculate_ohlc(&ohlc_data)
        .await
        .map_err(|e| e.to_string())?;
    let upper_values = kc_upper
        .calculate_ohlc(&ohlc_data)
        .await
        .map_err(|e| e.to_string())?;
    let lower_values = kc_lower
        .calculate_ohlc(&ohlc_data)
        .await
        .map_err(|e| e.to_string())?;

    // Выводим результаты
    println!("Keltner Channel Example:");
    println!(
        "Middle (EMA): {:?}",
        &middle_values[middle_values.len() - 5..]
    );
    println!("Upper: {:?}", &upper_values[upper_values.len() - 5..]);
    println!("Lower: {:?}", &lower_values[lower_values.len() - 5..]);

    Ok(())
}

/// Пример использования SuperTrend на основе Stochastic
pub async fn supertrend_on_stochastic_example() -> Result<(), String> {
    // Создаем тестовые OHLC данные
    let ohlc_data = create_test_ohlc_data();

    // 1. Вычисляем Stochastic
    let stochastic = Stochastic::new(14.0).map_err(|e| e.to_string())?;
    let stochastic_values = stochastic
        .calculate_ohlc(&ohlc_data)
        .await
        .map_err(|e| e.to_string())?;

    // 2. Создаем SuperTrend с параметрами
    let supertrend = SuperTrend::new(14.0, 3.0).map_err(|e| e.to_string())?;

    // 3. Применяем SuperTrend к данным Stochastic
    let supertrend_on_stochastic = supertrend
        .calculate_simple(&stochastic_values)
        .await
        .map_err(|e| e.to_string())?;

    // Выводим результаты
    println!("SuperTrend on Stochastic Example:");
    println!(
        "Stochastic values: {:?}",
        &stochastic_values[stochastic_values.len() - 10..]
    );
    println!(
        "SuperTrend on Stochastic: {:?}",
        &supertrend_on_stochastic[supertrend_on_stochastic.len() - 10..]
    );

    Ok(())
}

/// Создание тестовых OHLC данных
fn create_test_ohlc_data() -> OHLCData {
    // Создаем простые тестовые данные
    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();

    // Генерируем данные для 100 баров
    for i in 0..100 {
        let base = 100.0 + (i as f32 / 10.0).sin() * 10.0;
        open.push(base);
        high.push(base + 1.0 + (i as f32 / 20.0).cos() * 2.0);
        low.push(base - 1.0 - (i as f32 / 15.0).sin() * 2.0);
        close.push(base + (i as f32 / 25.0).cos() * 1.5);
    }

    OHLCData::new(open, high, low, close)
}

/// Пример использования SuperTrend на разных индикаторах
pub async fn supertrend_on_various_indicators() -> Result<(), String> {
    // Создаем тестовые OHLC данные
    let ohlc_data = create_test_ohlc_data();

    // Создаем SuperTrend с параметрами
    let supertrend = SuperTrend::new(14.0, 2.0).map_err(|e| e.to_string())?;

    // Создаем различные индикаторы
    let stochastic = Stochastic::new(14.0).map_err(|e| e.to_string())?;

    // Вычисляем значения индикаторов
    let stochastic_values = stochastic
        .calculate_ohlc(&ohlc_data)
        .await
        .map_err(|e| e.to_string())?;

    // Применяем SuperTrend к данным индикаторов
    let supertrend_on_stochastic = supertrend
        .calculate_simple(&stochastic_values)
        .await
        .map_err(|e| e.to_string())?;

    // Выводим результаты
    println!("SuperTrend on Various Indicators Example:");
    println!(
        "SuperTrend on Stochastic: {:?}",
        &supertrend_on_stochastic[supertrend_on_stochastic.len() - 5..]
    );

    Ok(())
}
