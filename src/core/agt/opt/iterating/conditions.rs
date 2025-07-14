use enum_iterator::all;
use itertools::Itertools; // itertools = "0.12"
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, enum_iterator::Sequence)]
pub enum ConditionEnum {
    ABOVE,
    BELOW,
    CROSSESABOVE,
    CROSSESBELOW,
    LOWERPERCENTBARS,
    GREATERPERCENTBARS,
    LOWERPERCENTBARSINDICATORS,
    GREATERPERCENTBARSINDICATORS,
    FAILINGDATABARS,
    FAILINGINDICATORSBARS, // Добавлено
    FALLINGTORISINGDATA,
    FALLINGTORISINGINDICATORS, // Добавлено
    RISINGDATABARS,
    RISINGINDICATORSBARS, // Добавлено
    RISINGTOFALLINGDATA,
    RISINGTOFALLINGINDICATORS, // Добавлено
}

pub struct SourceCombinationCondition {}

impl SourceCombinationCondition {
    /// Метод для генерации комбинаций условий
    pub fn execute(quantity: usize) -> Vec<Vec<ConditionEnum>> {
        let all_conditions: Vec<ConditionEnum> = all::<ConditionEnum>().collect();

        // Создаём вектор итераторов длины quantity
        let repeated_iterators = std::iter::repeat(all_conditions.clone())
            .take(quantity)
            .collect::<Vec<_>>();

        // Переводим его в итератор по ссылкам и применяем multi_cartesian_product
        let combinations: Vec<Vec<ConditionEnum>> = repeated_iterators
            .into_iter()
            .map(|v| v.into_iter())
            .multi_cartesian_product()
            .collect();

        combinations
    }

    /// n — длина комбинации
    /// target — необязательная фильтрация по конкретной комбинации
    pub async fn combination_condition(
        n: usize,
        target: Option<Vec<ConditionEnum>>,
    ) -> Vec<Vec<ConditionEnum>> {
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
}
