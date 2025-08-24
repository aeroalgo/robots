use crate::indicators::registry::IndicatorFactory;
use std::collections::HashMap;

#[test]
fn test_factory_bollinger_bands() {
    // Тестируем создание Bollinger Bands компонентов через фабрику
    let mut params = HashMap::new();
    params.insert("period".to_string(), 20.0);
    params.insert("deviation".to_string(), 2.0);

    // Тестируем BBMiddle
    let bb_middle = IndicatorFactory::create_indicator("BBMIDDLE", params.clone());
    assert!(bb_middle.is_ok());

    // Тестируем BBUpper
    let bb_upper = IndicatorFactory::create_indicator("BBUPPER", params.clone());
    assert!(bb_upper.is_ok());

    // Тестируем BBLower
    let bb_lower = IndicatorFactory::create_indicator("BBLOWER", params);
    assert!(bb_lower.is_ok());
}

#[test]
fn test_factory_keltner_channel() {
    // Тестируем создание Keltner Channel компонентов через фабрику
    let mut params = HashMap::new();
    params.insert("period".to_string(), 20.0);
    params.insert("atr_period".to_string(), 10.0);
    params.insert("atr_multiplier".to_string(), 2.0);

    // Тестируем KCMiddle
    let mut kc_middle_params = HashMap::new();
    kc_middle_params.insert("period".to_string(), 20.0);
    let kc_middle = IndicatorFactory::create_indicator("KCMIDDLE", kc_middle_params);
    assert!(kc_middle.is_ok());

    // Тестируем KCUpper
    let kc_upper = IndicatorFactory::create_indicator("KCUPPER", params.clone());
    assert!(kc_upper.is_ok());

    // Тестируем KCLower
    let kc_lower = IndicatorFactory::create_indicator("KCLOWER", params);
    assert!(kc_lower.is_ok());
}

#[test]
fn test_factory_available_indicators() {
    let available = IndicatorFactory::get_available_indicators();

    // Проверяем, что новые индикаторы есть в списке
    assert!(available.contains(&"BBMiddle"));
    assert!(available.contains(&"BBUpper"));
    assert!(available.contains(&"BBLower"));
    assert!(available.contains(&"KCMiddle"));
    assert!(available.contains(&"KCUpper"));
    assert!(available.contains(&"KCLower"));
}

#[test]
fn test_factory_indicator_info() {
    // Тестируем получение информации о Bollinger Bands
    let bb_middle_info = IndicatorFactory::get_indicator_info("BBMIDDLE");
    assert!(bb_middle_info.is_some());

    let info = bb_middle_info.unwrap();
    assert_eq!(info.name, "BBMiddle");
    assert_eq!(info.parameters.len(), 2);
    assert!(info.parameters.contains(&"period".to_string()));
    assert!(info.parameters.contains(&"deviation".to_string()));

    // Тестируем получение информации о Keltner Channel
    let kc_upper_info = IndicatorFactory::get_indicator_info("KCUPPER");
    assert!(kc_upper_info.is_some());

    let info = kc_upper_info.unwrap();
    assert_eq!(info.name, "KCUpper");
    assert_eq!(info.parameters.len(), 3);
    assert!(info.parameters.contains(&"period".to_string()));
    assert!(info.parameters.contains(&"atr_period".to_string()));
    assert!(info.parameters.contains(&"atr_multiplier".to_string()));
}

#[test]
fn test_factory_default_parameters() {
    // Тестируем создание с параметрами по умолчанию
    let empty_params = HashMap::new();

    // BBMiddle с параметрами по умолчанию
    let bb_middle = IndicatorFactory::create_indicator("BBMIDDLE", empty_params.clone());
    assert!(bb_middle.is_ok());

    // BBUpper с параметрами по умолчанию
    let bb_upper = IndicatorFactory::create_indicator("BBUPPER", empty_params.clone());
    assert!(bb_upper.is_ok());

    // KCMiddle с параметрами по умолчанию
    let kc_middle = IndicatorFactory::create_indicator("KCMIDDLE", empty_params);
    assert!(kc_middle.is_ok());
}
