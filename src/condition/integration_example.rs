use crate::condition::{
    factory::ConditionFactory,
    base::Condition,
};
use crate::indicators::{
    factory::IndicatorFactory,
    types::OHLCData,
};
use std::collections::HashMap;

/// Пример интеграции условий с индикаторами
pub async fn indicator_condition_integration() -> Result<(), String> {
    println!("=== Интеграция условий с индикаторами ===\n");
    
    // Создаем тестовые OHLC данные
    let ohlc_data = create_test_ohlc_data();
    
    // 1. Создаем RSI индикатор
    println!("1. Создание RSI индикатора...");
    let mut rsi_params = HashMap::new();
    rsi_params.insert("period".to_string(), 14.0);
    
    let rsi_indicator = IndicatorFactory::create_indicator("RSI", rsi_params)
        .map_err(|e| format!("Ошибка создания RSI: {:?}", e))?;
    
    // 2. Вычисляем RSI
    println!("2. Вычисление RSI...");
    let rsi_data = rsi_indicator.calculate_ohlc(&ohlc_data).await
        .map_err(|e| format!("Ошибка вычисления RSI: {:?}", e))?;
    
    println!("   RSI значения: {:?}", rsi_data);
    
    // 3. Создаем условие "RSI выше 70" (перекупленность)
    println!("3. Создание условия 'RSI выше 70'...");
    let threshold_data = vec![70.0; rsi_data.len()]; // Вектор с постоянным значением 70
    
    let overbought_condition = ConditionFactory::create_condition_default("Above")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;
    
    // 4. Проверяем условие на данных RSI
    println!("4. Проверка условия на данных RSI...");
    let overbought_result = overbought_condition.check_dual(&rsi_data, &threshold_data).await
        .map_err(|e| format!("Ошибка проверки условия: {:?}", e))?;
    
    println!("   Сигналы перекупленности: {:?}", overbought_result.signals);
    println!("   Силы сигналов: {:?}", overbought_result.strengths);
    
    // 5. Создаем условие "RSI ниже 30" (перепроданность)
    println!("5. Создание условия 'RSI ниже 30'...");
    let threshold_data_30 = vec![30.0; rsi_data.len()]; // Вектор с постоянным значением 30
    
    let oversold_condition = ConditionFactory::create_condition_default("Above")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;
    
    // 6. Проверяем условие на данных RSI
    println!("6. Проверка условия на данных RSI...");
    let oversold_result = oversold_condition.check_dual(&rsi_data, &threshold_data_30).await
        .map_err(|e| format!("Ошибка проверки условия: {:?}", e))?;
    
    // Инвертируем сигналы (так как мы проверяем "выше 30", а нам нужно "ниже 30")
    let oversold_signals: Vec<bool> = oversold_result.signals
        .iter()
        .map(|&s| !s)
        .collect();
    
    println!("   Сигналы перепроданности: {:?}", oversold_signals);
    
    // 7. Комбинируем сигналы для создания торговой стратегии
    println!("7. Создание торговой стратегии...");
    let mut trading_signals = Vec::with_capacity(rsi_data.len());
    let mut signal_types = Vec::with_capacity(rsi_data.len());
    
    for i in 0..rsi_data.len() {
        let overbought = overbought_result.signals[i];
        let oversold = oversold_signals[i];
        
        if overbought {
            trading_signals.push("SELL");
            signal_types.push("Перекупленность");
        } else if oversold {
            trading_signals.push("BUY");
            signal_types.push("Перепроданность");
        } else {
            trading_signals.push("HOLD");
            signal_types.push("Нейтрально");
        }
    }
    
    println!("   Торговые сигналы: {:?}", trading_signals);
    println!("   Типы сигналов: {:?}", signal_types);
    
    // 8. Анализ результатов
    println!("8. Анализ результатов...");
    let buy_signals = trading_signals.iter().filter(|&&s| s == "BUY").count();
    let sell_signals = trading_signals.iter().filter(|&&s| s == "SELL").count();
    let hold_signals = trading_signals.iter().filter(|&&s| s == "HOLD").count();
    
    println!("   Покупки: {}", buy_signals);
    println!("   Продажи: {}", sell_signals);
    println!("   Удержание: {}", hold_signals);
    
    // 9. Пример с другим индикатором - SMA
    println!("9. Пример с SMA индикатором...");
    let mut sma_params = HashMap::new();
    sma_params.insert("period".to_string(), 5.0);
    
    let sma_indicator = IndicatorFactory::create_indicator("SMA", sma_params)
        .map_err(|e| format!("Ошибка создания SMA: {:?}", e))?);
    
    let sma_data = sma_indicator.calculate_ohlc(&ohlc_data).await
        .map_err(|e| format!("Ошибка вычисления SMA: {:?}", e))?;
    
    println!("   SMA значения: {:?}", sma_data);
    
    // 10. Создаем условие пересечения цены выше SMA
    println!("10. Создание условия 'Цена выше SMA'...");
    let price_above_sma_condition = ConditionFactory::create_condition_default("CrossesAbove")
        .map_err(|e| format!("Ошибка создания условия: {:?}", e))?;
    
    let price_above_sma_result = price_above_sma_condition.check_dual(&ohlc_data.close, &sma_data).await
        .map_err(|e| format!("Ошибка проверки условия: {:?}", e))?;
    
    println!("   Сигналы 'Цена выше SMA': {:?}", price_above_sma_result.signals);
    
    // 11. Комбинируем все условия для сложной стратегии
    println!("11. Создание сложной торговой стратегии...");
    let mut complex_signals = Vec::with_capacity(rsi_data.len());
    
    for i in 0..rsi_data.len() {
        let rsi_overbought = overbought_result.signals[i];
        let rsi_oversold = oversold_signals[i];
        let price_above_sma = price_above_sma_result.signals[i];
        
        let signal = if rsi_oversold && price_above_sma {
            "STRONG_BUY" // Сильная покупка: RSI перепродан + цена выше SMA
        } else if rsi_overbought && !price_above_sma {
            "STRONG_SELL" // Сильная продажа: RSI перекуплен + цена ниже SMA
        } else if rsi_oversold {
            "BUY" // Покупка: только RSI перепродан
        } else if rsi_overbought {
            "SELL" // Продажа: только RSI перекуплен
        } else {
            "HOLD" // Удержание
        };
        
        complex_signals.push(signal);
    }
    
    println!("   Сложные торговые сигналы: {:?}", complex_signals);
    
    // 12. Статистика сложной стратегии
    println!("12. Статистика сложной стратегии...");
    let strong_buy = complex_signals.iter().filter(|&&s| s == "STRONG_BUY").count();
    let strong_sell = complex_signals.iter().filter(|&&s| s == "STRONG_SELL").count();
    let buy = complex_signals.iter().filter(|&&s| s == "BUY").count();
    let sell = complex_signals.iter().filter(|&&s| s == "SELL").count();
    let hold = complex_signals.iter().filter(|&&s| s == "HOLD").count();
    
    println!("   Сильные покупки: {}", strong_buy);
    println!("   Сильные продажи: {}", strong_sell);
    println!("   Покупки: {}", buy);
    println!("   Продажи: {}", sell);
    println!("   Удержание: {}", hold);
    
    println!("\n✅ Интеграция условий с индикаторами завершена успешно!");
    Ok(())
}

/// Создание тестовых OHLC данных
fn create_test_ohlc_data() -> OHLCData {
    let open = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0];
    let high = vec![101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0];
    let low = vec![99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0];
    let close = vec![100.5, 101.5, 102.5, 103.5, 104.5, 105.5, 106.5, 107.5, 108.5, 109.5];
    
    OHLCData::new(open, high, low, close)
}

/// Запуск всех примеров интеграции
pub async fn run_integration_examples() -> Result<(), String> {
    println!("🚀 Запуск примеров интеграции условий с индикаторами\n");
    
    indicator_condition_integration().await?;
    
    println!("\n✅ Все примеры интеграции выполнены успешно!");
    Ok(())
}
