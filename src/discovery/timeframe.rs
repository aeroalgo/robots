use crate::data_model::types::TimeFrame;

/// Генератор комбинаций таймфреймов
pub struct TimeFrameGenerator;

impl TimeFrameGenerator {
    /// Генерирует комбинации таймфреймов на основе базового таймфрейма
    /// Генерирует таймфреймы кратные базовому до max_timeframe_minutes
    /// Возвращает все возможные комбинации от 1 до count таймфреймов
    pub fn generate_combinations(
        base: TimeFrame,
        count: usize,
        max_timeframe_minutes: u32,
    ) -> Vec<Vec<TimeFrame>> {
        let base_minutes = Self::timeframe_to_minutes(&base);

        if base_minutes == 0 || base_minutes > max_timeframe_minutes {
            return vec![vec![base.clone()]];
        }

        let mut timeframes = vec![base.clone()];

        // Генерируем дополнительные таймфреймы как кратные базовому до max_timeframe_minutes
        let mut multiplier = 2;
        loop {
            let minutes = base_minutes * multiplier;
            if minutes > max_timeframe_minutes {
                break;
            }
            timeframes.push(TimeFrame::Minutes(minutes));
            multiplier += 1;
        }

        let mut result = Vec::new();
        for combo_len in 1..=count.min(timeframes.len()) {
            let combinations = Self::combinations(timeframes.clone(), combo_len);
            result.extend(combinations);
        }
        result
    }

    /// Преобразует TimeFrame в минуты
    fn timeframe_to_minutes(tf: &TimeFrame) -> u32 {
        match tf {
            TimeFrame::Minutes(m) => *m,
            TimeFrame::Hours(h) => *h * 60,
            TimeFrame::Days(d) => *d * 24 * 60,
            TimeFrame::Weeks(w) => *w * 7 * 24 * 60,
            TimeFrame::Months(m) => *m * 30 * 24 * 60,
            TimeFrame::Custom(_) => {
                // Для кастомных таймфреймов возвращаем 0 (не поддерживается)
                0
            }
        }
    }

    /// Генерирует все комбинации из элементов списка длиной k
    fn combinations<T: Clone>(items: Vec<T>, k: usize) -> Vec<Vec<T>> {
        if k == 0 {
            return vec![vec![]];
        }
        if k > items.len() {
            return vec![];
        }

        let mut result = Vec::new();
        for i in 0..=items.len() - k {
            let first = items[i].clone();
            let rest_combinations = Self::combinations(items[i + 1..].to_vec(), k - 1);
            for mut combo in rest_combinations {
                combo.insert(0, first.clone());
                result.push(combo);
            }
        }
        result
    }
}
