#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{base::Condition, factory::ConditionFactory, types::OHLCData};
    use std::collections::HashMap;

    fn create_test_data() -> Vec<f32> {
        vec![95.0, 98.0, 102.0, 105.0, 103.0, 108.0, 110.0]
    }

    fn create_test_ohlc_data() -> OHLCData {
        let open = vec![100.0, 101.0, 102.0, 103.0, 104.0];
        let high = vec![101.0, 102.0, 103.0, 104.0, 105.0];
        let low = vec![99.0, 100.0, 101.0, 102.0, 103.0];
        let close = vec![100.5, 101.5, 102.5, 103.5, 104.5];

        OHLCData::new(open, high, low, close)
    }

    #[tokio::test]
    async fn test_above_condition() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let data1 = create_test_data();
        let data2 = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

        let result = condition.check_dual(&data1, &data2).await.unwrap();

        // Проверяем, что сигналы корректны
        assert_eq!(result.signals.len(), data1.len());
        assert_eq!(result.signals[0], false); // 95 < 100
        assert_eq!(result.signals[2], true); // 102 > 100
        assert_eq!(result.signals[6], true); // 110 > 100

        // Проверяем силы сигналов
        assert_eq!(result.strengths.len(), data1.len());
        assert_eq!(result.directions.len(), data1.len());
    }

    #[tokio::test]
    async fn test_crosses_above_condition() {
        let condition = ConditionFactory::create_condition_default("CrossesAbove").unwrap();
        let data = create_test_data();

        let result = condition.check_simple(&data).await.unwrap();

        // Первый элемент не может быть пересечением
        assert_eq!(result.signals[0], false);

        // Проверяем, что есть сигналы пересечения
        assert_eq!(result.signals.len(), data.len());
        assert!(result.signals.iter().any(|&s| s));
    }

    #[tokio::test]
    async fn test_rising_trend_condition() {
        let mut params = HashMap::new();
        params.insert("period".to_string(), 3.0);

        let condition = ConditionFactory::create_condition("RisingTrend", params).unwrap();
        let data = create_test_data();

        let result = condition.check_simple(&data).await.unwrap();

        // Первые 2 элемента не могут быть трендом (период = 3)
        assert_eq!(result.signals[0], false);
        assert_eq!(result.signals[1], false);

        // Проверяем, что есть сигналы тренда
        assert_eq!(result.signals.len(), data.len());
        assert!(result.signals.iter().any(|&s| s));
    }

    #[tokio::test]
    async fn test_dual_condition() {
        let condition = ConditionFactory::create_condition_default("CrossesAbove").unwrap();
        let line1 = vec![95.0, 98.0, 102.0, 105.0];
        let line2 = vec![100.0, 100.0, 100.0, 100.0];

        let result = condition.check_dual(&line1, &line2).await.unwrap();

        // Проверяем пересечение выше
        assert_eq!(result.signals[0], false); // Первый элемент
        assert_eq!(result.signals[2], true); // 102 > 100 и 98 <= 100
        assert_eq!(result.signals[3], false); // 105 > 100 но 102 > 100 (уже выше)
    }

    #[tokio::test]
    async fn test_ohlc_condition() {
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), 102.0);

        let condition = ConditionFactory::create_condition("Above", params).unwrap();
        let ohlc_data = create_test_ohlc_data();

        let result = condition.check_ohlc(&ohlc_data).await.unwrap();

        // Проверяем, что результат основан на close ценах
        assert_eq!(result.signals.len(), ohlc_data.close.len());
        assert_eq!(result.signals[0], false); // 100.5 < 102.0
        assert_eq!(result.signals[2], true); // 102.5 > 102.0
    }

    #[tokio::test]
    async fn test_condition_parameters() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();

        // Валидация входных данных
        let data = vec![1.0, 2.0, 3.0];
        assert!(condition.validate_input_data(&data).is_ok());

        // Проверяем минимальное количество точек
        let short_data = vec![1.0];
        assert!(condition.validate_input_data(&short_data).is_err());
    }

    #[tokio::test]
    async fn test_condition_factory() {
        // Проверяем доступные условия
        let available = ConditionFactory::get_available_conditions();
        assert!(available.contains(&"Above"));
        assert!(available.contains(&"CrossesAbove"));
        assert!(available.contains(&"RisingTrend"));

        // Проверяем информацию об условиях
        let above_info = ConditionFactory::get_condition_info("Above");
        assert!(above_info.is_some());

        let info = above_info.unwrap();
        assert_eq!(info.name, "Above");
        assert_eq!(info.min_data_points, 2);

        // Проверяем создание с параметрами по умолчанию
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        assert_eq!(condition.min_data_points(), 2);
    }

    #[tokio::test]
    async fn test_condition_cloning() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let cloned = condition.clone_box();

        // Проверяем, что клонированное условие работает
        let data1 = create_test_data();
        let data2 = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

        let original_result = condition.check_dual(&data1, &data2).await.unwrap();
        let cloned_result = cloned.check_dual(&data1, &data2).await.unwrap();

        assert_eq!(original_result.signals, cloned_result.signals);
        assert_eq!(original_result.strengths, cloned_result.strengths);
        assert_eq!(original_result.directions, cloned_result.directions);
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Тест с недостаточными данными
        let mut params = HashMap::new();
        params.insert("period".to_string(), 10.0);

        let condition = ConditionFactory::create_condition("RisingTrend", params).unwrap();
        let short_data = vec![1.0, 2.0, 3.0]; // Меньше чем period

        let result = condition.check_simple(&short_data).await;
        assert!(result.is_err());

        // Тест с неверным именем условия
        let result = ConditionFactory::create_condition("UnknownCondition", HashMap::new());
        assert!(result.is_err());
    }
}
