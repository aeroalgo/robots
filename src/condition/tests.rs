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
    async fn test_rising_trend_condition() {
        let mut params = HashMap::new();
        params.insert("period".to_string(), 3.0);

        let condition = ConditionFactory::create_condition("RisingTrend", params).unwrap();
        let data = create_test_data();

        let result = condition.check(ConditionInputData::single(&data)).unwrap();

        assert_eq!(result.signals[0], false);
        assert_eq!(result.signals[1], false);
        assert_eq!(result.signals.len(), data.len());
        assert!(result.signals.iter().any(|&s| s));
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

        let result = condition.check(ConditionInputData::single(&short_data));
        assert!(result.is_err());

        let result = ConditionFactory::create_condition("UnknownCondition", HashMap::new());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_below_condition() {
        let condition = ConditionFactory::create_condition_default("Below").unwrap();
        let data1 = create_test_data();
        let data2 = vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0];

        let result = condition
            .check(ConditionInputData::dual(&data1, &data2))
            .unwrap();

        assert_eq!(result.signals.len(), data1.len());
        assert_eq!(result.signals[0], true);
        assert_eq!(result.signals[2], false);
        assert_eq!(result.signals[6], false);
        assert_eq!(result.strengths.len(), data1.len());
        assert_eq!(result.directions.len(), data1.len());
    }

    #[tokio::test]
    async fn test_greater_percent_condition() {
        let condition = ConditionFactory::create_condition_default("GreaterPercent").unwrap();
        let data1 = vec![110.0, 105.0, 108.0];
        let data2 = vec![100.0, 100.0, 100.0];
        let percent = 5.0;

        let result = condition
            .check(ConditionInputData::dual_with_percent(
                &data1, &data2, percent,
            ))
            .unwrap();

        assert_eq!(result.signals.len(), data1.len());
        assert_eq!(result.signals[0], true);
        assert_eq!(result.signals[1], true);
        assert_eq!(result.signals[2], true);
    }

    #[tokio::test]
    async fn test_greater_percent_condition_missing_percent() {
        let condition = ConditionFactory::create_condition_default("GreaterPercent").unwrap();
        let data1 = vec![110.0, 105.0, 108.0];
        let data2 = vec![100.0, 100.0, 100.0];

        let result = condition.check(ConditionInputData::dual(&data1, &data2));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lower_percent_condition() {
        let condition = ConditionFactory::create_condition_default("LowerPercent").unwrap();
        let data1 = vec![90.0, 95.0, 92.0];
        let data2 = vec![100.0, 100.0, 100.0];
        let percent = 5.0;

        let result = condition
            .check(ConditionInputData::dual_with_percent(
                &data1, &data2, percent,
            ))
            .unwrap();

        assert_eq!(result.signals.len(), data1.len());
        assert_eq!(result.signals[0], true);
        assert_eq!(result.signals[1], false);
        assert_eq!(result.signals[2], true);
    }

    #[tokio::test]
    async fn test_lower_percent_condition_missing_percent() {
        let condition = ConditionFactory::create_condition_default("LowerPercent").unwrap();
        let data1 = vec![90.0, 95.0, 92.0];
        let data2 = vec![100.0, 100.0, 100.0];

        let result = condition.check(ConditionInputData::dual(&data1, &data2));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_falling_trend_condition() {
        let mut params = HashMap::new();
        params.insert("period".to_string(), 3.0);

        let condition = ConditionFactory::create_condition("FallingTrend", params).unwrap();
        let data = vec![110.0, 108.0, 105.0, 103.0, 100.0, 98.0, 95.0];

        let result = condition.check(ConditionInputData::single(&data)).unwrap();

        assert_eq!(result.signals[0], false);
        assert_eq!(result.signals[1], false);
        assert_eq!(result.signals.len(), data.len());
        assert!(result.signals.iter().any(|&s| s));
    }

    #[tokio::test]
    async fn test_empty_data() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let empty_data1: Vec<f32> = vec![];
        let empty_data2: Vec<f32> = vec![];

        let result = condition.check(ConditionInputData::dual(&empty_data1, &empty_data2));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_insufficient_data() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let data1 = vec![1.0];
        let data2 = vec![2.0];

        let result = condition.check(ConditionInputData::dual(&data1, &data2));
        assert!(result.is_err());

        let empty_data1: Vec<f32> = vec![];
        let empty_data2: Vec<f32> = vec![];
        let result = condition.check(ConditionInputData::dual(&empty_data1, &empty_data2));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_large_data_arrays() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let data1: Vec<f32> = (0..10000).map(|i| i as f32).collect();
        let data2: Vec<f32> = (0..10000).map(|i| (i as f32) * 0.5).collect();

        let result = condition
            .check(ConditionInputData::dual(&data1, &data2))
            .unwrap();
        assert_eq!(result.signals.len(), 10000);
        assert_eq!(result.signals[0], false);
        assert!(result.signals[1..].iter().all(|&s| s));
    }

    #[tokio::test]
    async fn test_nan_values() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let data1 = vec![f32::NAN, 2.0, 3.0];
        let data2 = vec![1.0, 2.0, 3.0];

        let result = condition.check(ConditionInputData::dual(&data1, &data2));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_infinity_values() {
        let condition = ConditionFactory::create_condition_default("Above").unwrap();
        let data1 = vec![f32::INFINITY, 2.0, 3.0];
        let data2 = vec![1.0, 2.0, 3.0];

        let result = condition.check(ConditionInputData::dual(&data1, &data2));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_greater_percent_boundary_values() {
        let condition = ConditionFactory::create_condition_default("GreaterPercent").unwrap();
        let data1 = vec![100.0, 100.0, 100.0];
        let data2 = vec![100.0, 100.0, 100.0];
        let percent = 0.0;

        let result = condition
            .check(ConditionInputData::dual_with_percent(
                &data1, &data2, percent,
            ))
            .unwrap();

        assert_eq!(result.signals.len(), data1.len());
    }

    #[tokio::test]
    async fn test_lower_percent_boundary_values() {
        let condition = ConditionFactory::create_condition_default("LowerPercent").unwrap();
        let data1 = vec![100.0, 100.0, 100.0];
        let data2 = vec![100.0, 100.0, 100.0];
        let percent = 0.0;

        let result = condition
            .check(ConditionInputData::dual_with_percent(
                &data1, &data2, percent,
            ))
            .unwrap();

        assert_eq!(result.signals.len(), data1.len());
    }

    #[tokio::test]
    async fn test_trend_condition_period_one() {
        let mut params = HashMap::new();
        params.insert("period".to_string(), 1.0);

        let condition = ConditionFactory::create_condition("RisingTrend", params).unwrap();
        let data = vec![100.0, 102.0, 105.0];

        let result = condition.check(ConditionInputData::single(&data)).unwrap();
        assert_eq!(result.signals.len(), data.len());
    }

    #[tokio::test]
    async fn test_trend_condition_period_larger_than_data() {
        let mut params = HashMap::new();
        params.insert("period".to_string(), 100.0);

        let condition = ConditionFactory::create_condition("RisingTrend", params).unwrap();
        let data = vec![100.0, 102.0, 105.0];

        let result = condition.check(ConditionInputData::single(&data));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_all_conditions_available() {
        let available = ConditionFactory::get_available_conditions();
        assert!(available.contains(&"Above"));
        assert!(available.contains(&"Below"));
        assert!(available.contains(&"RisingTrend"));
        assert!(available.contains(&"FallingTrend"));
        assert!(available.contains(&"GreaterPercent"));
        assert!(available.contains(&"LowerPercent"));
    }
}
