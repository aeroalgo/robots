#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{factory::ConditionFactory, types::ConditionInputData};
    use crate::indicators::OHLCData;
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

        let result = condition
            .check(ConditionInputData::dual(&data1, &data2))
            .unwrap();

        assert_eq!(result.signals.len(), data1.len());
        assert_eq!(result.signals[0], false);
        assert_eq!(result.signals[2], true);
        assert_eq!(result.signals[6], true);
        assert_eq!(result.strengths.len(), data1.len());
        assert_eq!(result.directions.len(), data1.len());
    }

    #[tokio::test]
    async fn test_crosses_above_condition() {
        let condition = ConditionFactory::create_condition_default("CrossesAbove").unwrap();
        let line1 = vec![95.0, 98.0, 102.0, 105.0];
        let line2 = vec![100.0, 100.0, 100.0, 100.0];

        let result = condition
            .check(ConditionInputData::dual(&line1, &line2))
            .unwrap();

        assert_eq!(result.signals[0], false);
        assert_eq!(result.signals.len(), line1.len());
        assert!(result.signals.iter().any(|&s| s));
    }

    #[tokio::test]
    async fn test_rising_trend_condition() {
        let mut params = HashMap::new();
        params.insert("period".to_string(), 3.0);

        let condition = ConditionFactory::create_condition("RisingTrend", params).unwrap();
        let data = create_test_data();

        let result = condition
            .check(ConditionInputData::single(&data))
            .unwrap();

        assert_eq!(result.signals[0], false);
        assert_eq!(result.signals[1], false);
        assert_eq!(result.signals.len(), data.len());
        assert!(result.signals.iter().any(|&s| s));
    }

    #[tokio::test]
    async fn test_crosses_above_dual_condition() {
        let condition = ConditionFactory::create_condition_default("CrossesAbove").unwrap();
        let line1 = vec![95.0, 98.0, 102.0, 105.0];
        let line2 = vec![100.0, 100.0, 100.0, 100.0];

        let result = condition
            .check(ConditionInputData::dual(&line1, &line2))
            .unwrap();

        assert_eq!(result.signals[0], false);
        assert_eq!(result.signals[2], true);
        assert_eq!(result.signals[3], false);
    }

    #[tokio::test]
    async fn test_ohlc_condition_returns_error() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let ohlc_data = create_test_ohlc_data();

        let result = condition.check(ConditionInputData::ohlc(&ohlc_data));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_condition_parameters() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let data1 = vec![1.0, 2.0, 3.0];
        let data2 = vec![0.5, 0.5, 0.5];

        assert!(condition
            .validate(&ConditionInputData::dual(&data1, &data2))
            .is_ok());

        let short_data = vec![1.0];
        assert!(condition
            .validate(&ConditionInputData::dual(&short_data, &data2[..1]))
            .is_err());
    }

    #[tokio::test]
    async fn test_condition_factory() {
        let available = ConditionFactory::get_available_conditions();
        assert!(available.contains(&"Above"));
        assert!(available.contains(&"CrossesAbove"));
        assert!(available.contains(&"RisingTrend"));

        let above_info = ConditionFactory::get_condition_info("Above");
        assert!(above_info.is_some());

        let info = above_info.unwrap();
        assert_eq!(info.name, "Above");
        assert_eq!(info.min_data_points, 2);

        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        assert_eq!(condition.min_data_points(), 2);
    }

    #[tokio::test]
    async fn test_condition_cloning() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let cloned = condition.clone_box();

        let data1 = create_test_data();
        let data2 = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

        let original_result = condition
            .check(ConditionInputData::dual(&data1, &data2))
            .unwrap();
        let cloned_result = cloned
            .check(ConditionInputData::dual(&data1, &data2))
            .unwrap();

        assert_eq!(original_result.signals, cloned_result.signals);
        assert_eq!(original_result.strengths, cloned_result.strengths);
        assert_eq!(original_result.directions, cloned_result.directions);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let mut params = HashMap::new();
        params.insert("period".to_string(), 10.0);

        let condition = ConditionFactory::create_condition("RisingTrend", params).unwrap();
        let short_data = vec![1.0, 2.0, 3.0];

        let result = condition
            .check(ConditionInputData::single(&short_data))
;
        assert!(result.is_err());

        let result = ConditionFactory::create_condition("UnknownCondition", HashMap::new());
        assert!(result.is_err());
    }
}
