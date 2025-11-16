use crate::data_model::types::TimeFrame;

/// Генератор комбинаций таймфреймов
pub struct TimeFrameGenerator;

impl TimeFrameGenerator {
    /// Генерирует комбинации таймфреймов на основе базового таймфрейма
    /// Возвращает все возможные комбинации из N таймфреймов
    pub fn generate_combinations(base: TimeFrame, count: usize) -> Vec<Vec<TimeFrame>> {
        let base_minutes = Self::timeframe_to_minutes(&base);
        let mut timeframes = vec![base.clone()];

        // Генерируем дополнительные таймфреймы как кратные базовому
        for i in 2..=count {
            let minutes = base_minutes * i as u32;
            timeframes.push(TimeFrame::Minutes(minutes));
        }

        // Генерируем все возможные комбинации
        Self::combinations(timeframes, count)
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

