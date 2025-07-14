use std::collections::HashMap;
use std::mem;

/// Анализатор использования памяти для модуля condition
pub struct MemoryAnalyzer;

impl MemoryAnalyzer {
    /// Анализирует размер структуры StrategyCondition
    pub fn analyze_strategy_condition_memory() -> HashMap<String, usize> {
        let mut analysis = HashMap::new();

        // Размеры типов данных
        analysis.insert("f32_size".to_string(), mem::size_of::<f32>());
        analysis.insert("bool_size".to_string(), mem::size_of::<bool>());
        analysis.insert("String_size".to_string(), mem::size_of::<String>());
        analysis.insert("Vec_f32_size".to_string(), mem::size_of::<Vec<f32>>());
        analysis.insert("Vec_bool_size".to_string(), mem::size_of::<Vec<bool>>());

        // Примерный размер StrategyCondition для 1000 элементов
        let data_size = 1000 * mem::size_of::<f32>();
        let indicator_size = 1000 * mem::size_of::<f32>();
        let constant_size = 1000 * mem::size_of::<f32>();
        let result_size = 1000 * mem::size_of::<bool>();
        let name_size = mem::size_of::<String>();

        let total_size = data_size + indicator_size + constant_size + result_size + name_size;

        analysis.insert("StrategyCondition_1000_elements".to_string(), total_size);
        analysis.insert("data_1000_elements".to_string(), data_size);
        analysis.insert("indicator_1000_elements".to_string(), indicator_size);
        analysis.insert("constant_1000_elements".to_string(), constant_size);
        analysis.insert("result_1000_elements".to_string(), result_size);

        analysis
    }

    /// Анализирует размер структуры OptimizedStrategyCondition
    pub fn analyze_optimized_condition_memory() -> HashMap<String, usize> {
        let mut analysis = HashMap::new();

        // Размеры типов данных
        analysis.insert("f32_size".to_string(), mem::size_of::<f32>());
        analysis.insert("bool_size".to_string(), mem::size_of::<bool>());
        analysis.insert("String_size".to_string(), mem::size_of::<String>());
        analysis.insert("slice_f32_size".to_string(), mem::size_of::<&[f32]>());
        analysis.insert("Vec_bool_size".to_string(), mem::size_of::<Vec<bool>>());

        // Примерный размер OptimizedStrategyCondition для 1000 элементов
        let slice_size = mem::size_of::<&[f32]>() * 2; // data и indicator слайсы
        let constant_size = mem::size_of::<f32>();
        let result_size = 1000 * mem::size_of::<bool>();
        let name_size = mem::size_of::<String>();

        let total_size = slice_size + constant_size + result_size + name_size;

        analysis.insert(
            "OptimizedStrategyCondition_1000_elements".to_string(),
            total_size,
        );
        analysis.insert("slices_size".to_string(), slice_size);
        analysis.insert("constant_size".to_string(), constant_size);
        analysis.insert("result_1000_elements".to_string(), result_size);

        analysis
    }

    /// Сравнивает использование памяти между базовой и оптимизированной версиями
    pub fn compare_memory_usage() -> HashMap<String, f64> {
        let basic_analysis = Self::analyze_strategy_condition_memory();
        let optimized_analysis = Self::analyze_optimized_condition_memory();

        let basic_size = basic_analysis
            .get("StrategyCondition_1000_elements")
            .unwrap_or(&0);
        let optimized_size = optimized_analysis
            .get("OptimizedStrategyCondition_1000_elements")
            .unwrap_or(&0);

        let mut comparison = HashMap::new();
        comparison.insert("basic_size_bytes".to_string(), *basic_size as f64);
        comparison.insert("optimized_size_bytes".to_string(), *optimized_size as f64);
        comparison.insert(
            "memory_saved_bytes".to_string(),
            (*basic_size - *optimized_size) as f64,
        );
        comparison.insert(
            "memory_saved_percent".to_string(),
            ((*basic_size - *optimized_size) as f64 / *basic_size as f64) * 100.0,
        );

        comparison
    }

    /// Анализирует производительность аллокаций
    pub fn analyze_allocation_performance() -> HashMap<String, String> {
        let mut analysis = HashMap::new();

        analysis.insert(
            "basic_cloning".to_string(),
            "Требует клонирования данных при создании".to_string(),
        );
        analysis.insert(
            "optimized_references".to_string(),
            "Использует ссылки на существующие данные".to_string(),
        );
        analysis.insert(
            "basic_memory_ownership".to_string(),
            "Каждый экземпляр владеет копией данных".to_string(),
        );
        analysis.insert(
            "optimized_memory_sharing".to_string(),
            "Данные разделяются между экземплярами".to_string(),
        );
        analysis.insert(
            "basic_allocation_pattern".to_string(),
            "Множественные аллокации при создании".to_string(),
        );
        analysis.insert(
            "optimized_allocation_pattern".to_string(),
            "Минимальные аллокации, предварительное выделение".to_string(),
        );

        analysis
    }

    /// Рекомендации по оптимизации памяти
    pub fn get_memory_optimization_recommendations() -> Vec<String> {
        vec![
            "Используйте OptimizedStrategyCondition вместо StrategyCondition для больших наборов данных".to_string(),
            "Избегайте клонирования данных - используйте слайсы".to_string(),
            "Предварительно выделяйте память для результатов с Vec::with_capacity()".to_string(),
            "Используйте ссылки на данные вместо владения копиями".to_string(),
            "Группируйте связанные данные в структуры для лучшей локальности кэша".to_string(),
            "Используйте итераторы вместо индексов где возможно".to_string(),
            "Рассмотрите использование Box<[T]> вместо Vec<T> для неизменяемых данных".to_string(),
            "Используйте SmallVec для небольших векторов (менее 32 элементов)".to_string(),
        ]
    }

    /// Анализирует использование памяти для конкретного набора данных
    pub fn analyze_data_set_memory_usage(
        data_size: usize,
        indicator_size: usize,
    ) -> HashMap<String, usize> {
        let mut analysis = HashMap::new();

        // Базовая версия
        let basic_data_size = data_size * mem::size_of::<f32>();
        let basic_indicator_size = indicator_size * mem::size_of::<f32>();
        let basic_constant_size = data_size * mem::size_of::<f32>();
        let basic_result_size = std::cmp::min(data_size, indicator_size) * mem::size_of::<bool>();
        let basic_total =
            basic_data_size + basic_indicator_size + basic_constant_size + basic_result_size;

        // Оптимизированная версия
        let optimized_slices_size = 2 * mem::size_of::<&[f32]>(); // data и indicator слайсы
        let optimized_constant_size = mem::size_of::<f32>();
        let optimized_result_size =
            std::cmp::min(data_size, indicator_size) * mem::size_of::<bool>();
        let optimized_total =
            optimized_slices_size + optimized_constant_size + optimized_result_size;

        analysis.insert("data_size_bytes".to_string(), basic_data_size);
        analysis.insert("indicator_size_bytes".to_string(), basic_indicator_size);
        analysis.insert("basic_total_bytes".to_string(), basic_total);
        analysis.insert("optimized_total_bytes".to_string(), optimized_total);
        analysis.insert(
            "memory_saved_bytes".to_string(),
            basic_total - optimized_total,
        );
        analysis.insert(
            "memory_saved_percent".to_string(),
            ((basic_total - optimized_total) as f64 / basic_total as f64 * 100.0) as usize,
        );

        analysis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_analysis() {
        let basic_analysis = MemoryAnalyzer::analyze_strategy_condition_memory();
        let optimized_analysis = MemoryAnalyzer::analyze_optimized_condition_memory();

        assert!(basic_analysis.contains_key("StrategyCondition_1000_elements"));
        assert!(optimized_analysis.contains_key("OptimizedStrategyCondition_1000_elements"));

        let comparison = MemoryAnalyzer::compare_memory_usage();
        assert!(comparison.contains_key("memory_saved_percent"));

        let recommendations = MemoryAnalyzer::get_memory_optimization_recommendations();
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_data_set_analysis() {
        let analysis = MemoryAnalyzer::analyze_data_set_memory_usage(1000, 1000);

        assert!(analysis.contains_key("basic_total_bytes"));
        assert!(analysis.contains_key("optimized_total_bytes"));
        assert!(analysis.contains_key("memory_saved_bytes"));

        let basic_total = analysis.get("basic_total_bytes").unwrap();
        let optimized_total = analysis.get("optimized_total_bytes").unwrap();

        // Оптимизированная версия должна использовать меньше памяти
        assert!(optimized_total < basic_total);
    }
}
