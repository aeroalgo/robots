use enum_iterator::all;
use itertools::Itertools; // itertools = "0.12"

use super::indicators::QuantityIndicators;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, enum_iterator::Sequence)]
pub enum ConditionEnum {
    ABOVE,
    BELOW,
    CROSSESABOVE,
    CROSSESBELOW,
    LOWERPERCENTBARS,
    GREATERPERCENTBARS,
    FAILINGDATABARS,
    FALLINGTORISINGDATA,
    RISINGDATABARS,
    RISINGTOFALLINGDATA,
}

pub struct SourceCombinationCondition {}

impl SourceCombinationCondition {
    /// n — длина комбинации
    /// target — необязательная фильтрация по конкретной комбинации
    pub async fn combination_condition(n: usize, target: Option<Vec<ConditionEnum>>) -> Vec<Vec<ConditionEnum>> {
        let all_conditions: Vec<ConditionEnum> = all::<ConditionEnum>().collect();

        // Создаём вектор итераторов длины n
        let repeated_iterators = std::iter::repeat(all_conditions.clone())
            .take(n)
            .collect::<Vec<_>>();

        // Переводим его в итератор по ссылкам и применяем multi_cartesian_product
        let combinations: Vec<Vec<ConditionEnum>> = repeated_iterators
            .into_iter()
            .map(|v| v.into_iter())
            .multi_cartesian_product()
            .collect();

        if let Some(target_combination) = target {
            combinations
                .into_iter()
                .filter(|c| *c == target_combination)
                .collect()
        } else {
            combinations
        }
    }
    pub fn check_condition(
        cond: ConditionEnum,
        data: &[f64],             // данные или индикаторы
        threshold: Option<f64>,   // для ABOVE/BELOW
        percent: Option<f64>,     // для процентов
        bar_count: Option<usize>, // для *_BARS
    ) -> bool {
        match cond {
            ConditionEnum::ABOVE => {
                if let Some(t) = threshold {
                    data.last().map_or(false, |&v| v > t)
                } else {
                    false
                }
            }

            ConditionEnum::BELOW => {
                if let Some(t) = threshold {
                    data.last().map_or(false, |&v| v < t)
                } else {
                    false
                }
            }

            ConditionEnum::CROSSESABOVE => {
                if data.len() < 2 || threshold.is_none() {
                    return false;
                }
                let t = threshold.unwrap();
                let prev = data[data.len() - 2];
                let curr = data[data.len() - 1];
                prev <= t && curr > t
            }

            ConditionEnum::CROSSESBELOW => {
                if data.len() < 2 || threshold.is_none() {
                    return false;
                }
                let t = threshold.unwrap();
                let prev = data[data.len() - 2];
                let curr = data[data.len() - 1];
                prev >= t && curr < t
            }

            ConditionEnum::RISINGTOFALLINGDATA => {
                if data.len() < 3 {
                    return false;
                }
                let (a, b, c) = (
                    data[data.len() - 3],
                    data[data.len() - 2],
                    data[data.len() - 1],
                );
                a < b && b > c
            }

            ConditionEnum::FALLINGTORISINGDATA => {
                if data.len() < 3 {
                    return false;
                }
                let (a, b, c) = (
                    data[data.len() - 3],
                    data[data.len() - 2],
                    data[data.len() - 1],
                );
                a > b && b < c
            }

            ConditionEnum::RISINGDATABARS => {
                if let Some(n) = bar_count {
                    if data.len() < n {
                        return false;
                    }
                    let recent = &data[data.len() - n..];
                    recent.windows(2).all(|w| w[0] < w[1])
                } else {
                    false
                }
            }

            ConditionEnum::FAILINGDATABARS => {
                if let Some(n) = bar_count {
                    if data.len() < n {
                        return false;
                    }
                    let recent = &data[data.len() - n..];
                    recent.windows(2).all(|w| w[0] > w[1])
                } else {
                    false
                }
            }

            ConditionEnum::LOWERPERCENTBARS => {
                if let (Some(n), Some(p)) = (bar_count, percent) {
                    if data.len() < n + 1 {
                        return false;
                    }
                    let base = data[data.len() - n - 1];
                    let target = base * (1.0 - p / 100.0);
                    data[data.len() - 1] < target
                } else {
                    false
                }
            }

            ConditionEnum::GREATERPERCENTBARS => {
                if let (Some(n), Some(p)) = (bar_count, percent) {
                    if data.len() < n + 1 {
                        return false;
                    }
                    let base = data[data.len() - n - 1];
                    let target = base * (1.0 + p / 100.0);
                    data[data.len() - 1] > target
                } else {
                    false
                }
            }
        }
    }
}
