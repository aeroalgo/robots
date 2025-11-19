#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::{
        base::Indicator,
        implementations::{BBLower, BBMiddle, BBUpper, KCLower, KCMiddle, KCUpper},
        types::OHLCData,
    };

    fn create_test_ohlc_data() -> OHLCData {
        let open = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0,
        ];
        let high = vec![
            101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];
        let low = vec![
            99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0,
        ];
        let close = vec![
            100.5, 101.5, 102.5, 103.5, 104.5, 105.5, 106.5, 107.5, 108.5, 109.5,
        ];

        OHLCData::new(open, high, low, close)
    }

    #[tokio::test]
    async fn test_bollinger_bands_components() {
        let ohlc_data = create_test_ohlc_data();

        // Тестируем BBMiddle
        let bb_middle = BBMiddle::new(5.0, 2.0).unwrap();
        let middle_values = bb_middle.calculate_ohlc(&ohlc_data).unwrap();

        assert_eq!(middle_values.len(), ohlc_data.len());
        assert!(middle_values[4] > 0.0); // Первое значение SMA

        // Тестируем BBUpper
        let bb_upper = BBUpper::new(5.0, 2.0).unwrap();
        let upper_values = bb_upper.calculate_ohlc(&ohlc_data).unwrap();

        assert_eq!(upper_values.len(), ohlc_data.len());
        assert!(upper_values[4] > middle_values[4]); // Верхняя линия должна быть выше средней

        // Тестируем BBLower
        let bb_lower = BBLower::new(5.0, 2.0).unwrap();
        let lower_values = bb_lower.calculate_ohlc(&ohlc_data).unwrap();

        assert_eq!(lower_values.len(), ohlc_data.len());
        assert!(lower_values[4] < middle_values[4]); // Нижняя линия должна быть ниже средней
    }

    #[tokio::test]
    async fn test_keltner_channel_components() {
        let ohlc_data = create_test_ohlc_data();

        // Тестируем KCMiddle
        let kc_middle = KCMiddle::new(5.0).unwrap();
        let middle_values = kc_middle.calculate_ohlc(&ohlc_data).unwrap();

        assert_eq!(middle_values.len(), ohlc_data.len());
        assert!(middle_values[4] > 0.0); // EMA должна быть положительной после прогрева периода

        // Тестируем KCUpper
        let kc_upper = KCUpper::new(5.0, 5.0, 2.0).unwrap();
        let upper_values = kc_upper.calculate_ohlc(&ohlc_data).unwrap();

        assert_eq!(upper_values.len(), ohlc_data.len());
        assert!(upper_values[4] > middle_values[4]); // Верхняя линия должна быть выше средней после прогрева

        // Тестируем KCLower
        let kc_lower = KCLower::new(5.0, 5.0, 2.0).unwrap();
        let lower_values = kc_lower.calculate_ohlc(&ohlc_data).unwrap();

        assert_eq!(lower_values.len(), ohlc_data.len());
        assert!(lower_values[4] < middle_values[4]); // Нижняя линия должна быть ниже средней после прогрева
    }

    #[tokio::test]
    async fn test_bollinger_bands_relationships() {
        let ohlc_data = create_test_ohlc_data();

        let bb_middle = BBMiddle::new(5.0, 2.0).unwrap();
        let bb_upper = BBUpper::new(5.0, 2.0).unwrap();
        let bb_lower = BBLower::new(5.0, 2.0).unwrap();

        let middle_values = bb_middle.calculate_ohlc(&ohlc_data).unwrap();
        let upper_values = bb_upper.calculate_ohlc(&ohlc_data).unwrap();
        let lower_values = bb_lower.calculate_ohlc(&ohlc_data).unwrap();

        // Проверяем, что верхняя и нижняя линии симметричны относительно средней
        for i in 4..ohlc_data.len() {
            let middle = middle_values[i];
            let upper = upper_values[i];
            let lower = lower_values[i];

            let upper_diff = upper - middle;
            let lower_diff = middle - lower;

            // Разница должна быть примерно одинаковой (с учетом погрешности вычислений)
            assert!((upper_diff - lower_diff).abs() < 0.1);
        }
    }

    #[tokio::test]
    async fn test_keltner_channel_relationships() {
        let ohlc_data = create_test_ohlc_data();

        let kc_middle = KCMiddle::new(5.0).unwrap();
        let kc_upper = KCUpper::new(5.0, 5.0, 2.0).unwrap();
        let kc_lower = KCLower::new(5.0, 5.0, 2.0).unwrap();

        let middle_values = kc_middle.calculate_ohlc(&ohlc_data).unwrap();
        let upper_values = kc_upper.calculate_ohlc(&ohlc_data).unwrap();
        let lower_values = kc_lower.calculate_ohlc(&ohlc_data).unwrap();

        // Проверяем, что верхняя и нижняя линии симметричны относительно средней
        for i in 0..ohlc_data.len() {
            let middle = middle_values[i];
            let upper = upper_values[i];
            let lower = lower_values[i];

            let upper_diff = upper - middle;
            let lower_diff = middle - lower;

            // Разница должна быть примерно одинаковой (с учетом погрешности вычислений)
            assert!((upper_diff - lower_diff).abs() < 0.1);
        }
    }
}
