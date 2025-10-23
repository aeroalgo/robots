# План реализации системы метрик

## Обзор

Детальный план реализации системы метрик оценки торговых стратегий на основе предоставленного списка из 40+ метрик.

## Архитектура системы метрик

### Модульная структура

```rust
src/
├── metrics/
│   ├── mod.rs
│   ├── calculator.rs          // Основной калькулятор
│   ├── report.rs             // Генерация отчетов
│   ├── basic/                // Базовые метрики производительности
│   │   ├── mod.rs
│   │   ├── total_profit.rs
│   │   ├── profit_pips.rs
│   │   ├── yearly_avg_profit.rs
│   │   ├── yearly_avg_return.rs
│   │   └── cagr.rs
│   ├── risk_return/          // Метрики риска и доходности
│   │   ├── mod.rs
│   │   ├── sharpe_ratio.rs
│   │   ├── profit_factor.rs
│   │   ├── return_dd_ratio.rs
│   │   └── winning_percentage.rs
│   ├── drawdown/             // Метрики просадки
│   │   ├── mod.rs
│   │   ├── draw_down.rs
│   │   ├── percent_draw_down.rs
│   │   ├── max_consec_wins.rs
│   │   └── max_consec_losses.rs
│   ├── statistical/          // Статистические метрики
│   │   ├── mod.rs
│   │   ├── r_expectancy.rs
│   │   ├── r_expectancy_score.rs
│   │   ├── sqn.rs
│   │   └── sqn_score.rs
│   ├── advanced/             // Продвинутые метрики
│   │   ├── mod.rs
│   │   ├── z_score.rs
│   │   ├── z_probability.rs
│   │   ├── expectancy.rs
│   │   ├── deviation.rs
│   │   └── exposure.rs
│   ├── symmetry/             // Метрики симметрии и стабильности
│   │   ├── mod.rs
│   │   ├── symmetry.rs
│   │   ├── trades_symmetry.rs
│   │   ├── nsymmetry.rs
│   │   └── stability.rs
│   ├── stagnation/           // Метрики застоя
│   │   ├── mod.rs
│   │   ├── stagnation_days.rs
│   │   ├── stagnation_percent.rs
│   │   ├── gross_profit.rs
│   │   └── gross_loss.rs
│   └── additional/           // Дополнительные метрики
│       ├── mod.rs
│       ├── average_win.rs
│       ├── average_loss.rs
│       ├── payout_ratio.rs
│       ├── ahpr.rs
│       ├── daily_avg_profit.rs
│       ├── monthly_avg_profit.rs
│       ├── average_trade.rs
│       ├── annual_max_dd_ratio.rs
│       └── wins_losses_ratio.rs
```

## Базовые трейты и структуры

### Основной трейт метрики

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: u64,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub side: Side,
    pub commission: f64,
    pub pnl: f64,
    pub return_pct: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Side {
    Long,
    Short,
}

pub trait Metric: Send + Sync {
    fn calculate(&self, trades: &[Trade], initial_capital: f64) -> MetricResult;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn category(&self) -> MetricCategory;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub value: f64,
    pub unit: String,
    pub interpretation: Option<String>,
    pub is_good: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricCategory {
    BasicPerformance,
    RiskReturn,
    Drawdown,
    Statistical,
    Advanced,
    Symmetry,
    Stagnation,
    Additional,
}
```

### Калькулятор метрик

```rust
pub struct MetricCalculator {
    metrics: Vec<Box<dyn Metric>>,
    cache: HashMap<String, MetricResult>,
}

impl MetricCalculator {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            cache: HashMap::new(),
        }
    }
    
    pub fn add_metric(&mut self, metric: Box<dyn Metric>) {
        self.metrics.push(metric);
    }
    
    pub fn calculate_all(&self, trades: &[Trade], initial_capital: f64) -> MetricReport {
        let mut results = HashMap::new();
        
        for metric in &self.metrics {
            let key = format!("{}_{}", metric.category() as u8, metric.name());
            if let Some(cached) = self.cache.get(&key) {
                results.insert(metric.name().to_string(), cached.clone());
            } else {
                let result = metric.calculate(trades, initial_capital);
                results.insert(metric.name().to_string(), result);
            }
        }
        
        MetricReport {
            results,
            generated_at: Utc::now(),
            trade_count: trades.len(),
            initial_capital,
        }
    }
    
    pub fn calculate_category(&self, trades: &[Trade], initial_capital: f64, category: MetricCategory) -> CategoryReport {
        let category_metrics: Vec<_> = self.metrics
            .iter()
            .filter(|m| m.category() == category)
            .collect();
            
        let mut results = HashMap::new();
        for metric in category_metrics {
            let result = metric.calculate(trades, initial_capital);
            results.insert(metric.name().to_string(), result);
        }
        
        CategoryReport {
            category,
            results,
            generated_at: Utc::now(),
        }
    }
}
```

## Фазы реализации

### Фаза 1: Критические метрики (1-2 недели)

#### 1.1 Базовые метрики производительности
- [ ] **Total Profit**
  - Простая сумма PnL всех сделок
  - Приоритет: Критический
  - Сложность: Низкая

- [ ] **CAGR (Compound Annual Growth Rate)**
  - Формула: ((Конечный капитал / Начальный капитал)^(1/лет) - 1) * 100
  - Приоритет: Критический
  - Сложность: Низкая

- [ ] **Yearly AVG Profit**
  - Total Profit / количество лет
  - Приоритет: Высокий
  - Сложность: Низкая

- [ ] **Yearly AVG % Return**
  - (Yearly AVG Profit / Начальный капитал) * 100
  - Приоритет: Высокий
  - Сложность: Низкая

#### 1.2 Метрики риска и доходности
- [ ] **Sharpe Ratio**
  - (Средняя доходность - Безрисковая ставка) / Стандартное отклонение
  - Приоритет: Критический
  - Сложность: Средняя

- [ ] **Profit Factor**
  - Gross Profit / |Gross Loss|
  - Приоритет: Критический
  - Сложность: Низкая

- [ ] **Winning Percentage**
  - (Прибыльные сделки / Все сделки) * 100
  - Приоритет: Высокий
  - Сложность: Низкая

#### 1.3 Метрики просадки
- [ ] **Draw Down**
  - Максимальное падение от пика
  - Приоритет: Критический
  - Сложность: Средняя

- [ ] **% Draw Down**
  - (Draw Down / Максимальный капитал) * 100
  - Приоритет: Критический
  - Сложность: Низкая

### Фаза 2: Важные метрики (2-3 недели)

#### 2.1 Статистические метрики
- [ ] **R Expectancy**
  - (Win Rate * Avg Win) - (Loss Rate * Avg Loss)
  - Приоритет: Высокий
  - Сложность: Средняя

- [ ] **SQN (Strategy Quality Number)**
  - (Expectancy / Std Dev) * sqrt(Количество сделок)
  - Приоритет: Высокий
  - Сложность: Высокая

- [ ] **Z-Score**
  - (Average Trade - 0) / Стандартное отклонение
  - Приоритет: Высокий
  - Сложность: Средняя

#### 2.2 Дополнительные важные метрики
- [ ] **Return/DD ratio**
  - Total Return / Max Drawdown
  - Приоритет: Высокий
  - Сложность: Низкая

- [ ] **Average Win**
  - Gross Profit / Количество прибыльных сделок
  - Приоритет: Высокий
  - Сложность: Низкая

- [ ] **Average Loss**
  - |Gross Loss| / Количество убыточных сделок
  - Приоритет: Высокий
  - Сложность: Низкая

- [ ] **Payout ratio**
  - Average Win / Average Loss
  - Приоритет: Высокий
  - Сложность: Низкая

### Фаза 3: Продвинутые метрики (2-3 недели)

#### 3.1 Статистические метрики
- [ ] **Z-Probability**
  - 1 - P(Z < Z-Score)
  - Приоритет: Средний
  - Сложность: Высокая

- [ ] **Expectancy**
  - Ожидаемая прибыль на сделку
  - Приоритет: Средний
  - Сложность: Низкая

- [ ] **Deviation**
  - Стандартное отклонение доходности
  - Приоритет: Средний
  - Сложность: Средняя

#### 3.2 Метрики симметрии и стабильности
- [ ] **Stability**
  - R² от линейной регрессии кривой капитала
  - Приоритет: Средний
  - Сложность: Высокая

- [ ] **Symmetry**
  - Long Profit / |Short Profit|
  - Приоритет: Средний
  - Сложность: Средняя

- [ ] **NSymmetry**
  - Направленная симметрия прибыльности
  - Приоритет: Средний
  - Сложность: Средняя

### Фаза 4: Дополнительные метрики (1-2 недели)

#### 4.1 Метрики застоя
- [ ] **Stagnation In Days**
  - Максимальные дни без нового максимума
  - Приоритет: Низкий
  - Сложность: Средняя

- [ ] **Stagnation In %**
  - Процент времени без нового максимума
  - Приоритет: Низкий
  - Сложность: Низкая

#### 4.2 Дополнительные метрики
- [ ] **Gross Profit/Loss**
  - Сумма прибыльных/убыточных сделок
  - Приоритет: Низкий
  - Сложность: Низкая

- [ ] **Daily/Monthly AVG Profit**
  - Средняя дневная/месячная прибыль
  - Приоритет: Низкий
  - Сложность: Низкая

- [ ] **Wins/Losses ratio**
  - Отношение количества прибыльных к убыточным
  - Приоритет: Низкий
  - Сложность: Низкая

## Технические детали реализации

### Производительность

#### SIMD оптимизация
```rust
use std::simd::*;

impl SharpeRatio {
    fn calculate_simd(&self, returns: &[f64]) -> f64 {
        let chunk_size = 8; // AVX2 chunk size
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        
        for chunk in returns.chunks(chunk_size) {
            if chunk.len() == chunk_size {
                let simd_chunk = f64x8::from_slice(chunk);
                sum += simd_chunk.reduce_sum();
                sum_sq += (simd_chunk * simd_chunk).reduce_sum();
            } else {
                // Handle remaining elements
                for &val in chunk {
                    sum += val;
                    sum_sq += val * val;
                }
            }
        }
        
        let mean = sum / returns.len() as f64;
        let variance = (sum_sq / returns.len() as f64) - (mean * mean);
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 { 0.0 } else { (mean - self.risk_free_rate) / std_dev }
    }
}
```

#### Параллельные вычисления
```rust
use rayon::prelude::*;

impl MetricCalculator {
    pub fn calculate_all_parallel(&self, trades: &[Trade], initial_capital: f64) -> MetricReport {
        let results: HashMap<String, MetricResult> = self.metrics
            .par_iter()
            .map(|metric| {
                let result = metric.calculate(trades, initial_capital);
                (metric.name().to_string(), result)
            })
            .collect();
            
        MetricReport {
            results,
            generated_at: Utc::now(),
            trade_count: trades.len(),
            initial_capital,
        }
    }
}
```

#### Кэширование
```rust
use lru::LruCache;

pub struct CachedMetricCalculator {
    calculator: MetricCalculator,
    cache: Arc<Mutex<LruCache<String, MetricResult>>>,
}

impl CachedMetricCalculator {
    pub fn calculate_with_cache(&self, trades: &[Trade], initial_capital: f64) -> MetricReport {
        let cache_key = format!("{}_{}", trades.len(), initial_capital);
        
        // Check cache first
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return cached.clone();
        }
        
        // Calculate and cache
        let result = self.calculator.calculate_all(trades, initial_capital);
        self.cache.lock().unwrap().put(cache_key, result.clone());
        result
    }
}
```

### Валидация и тестирование

#### Unit тесты
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sharpe_ratio_calculation() {
        let trades = create_test_trades();
        let sharpe = SharpeRatio::new(0.02);
        let result = sharpe.calculate(&trades, 10000.0);
        
        assert!(result.value > 0.0);
        assert_eq!(result.unit, "ratio");
        assert!(result.is_good.unwrap_or(false));
    }
    
    #[test]
    fn test_profit_factor_calculation() {
        let trades = create_test_trades();
        let pf = ProfitFactor::new();
        let result = pf.calculate(&trades, 10000.0);
        
        assert!(result.value > 1.0);
        assert_eq!(result.unit, "ratio");
    }
}
```

#### Интеграционные тесты
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_full_metrics_calculation() {
        let trades = create_large_test_dataset();
        let mut calculator = MetricCalculator::new();
        
        calculator.add_metric(Box::new(SharpeRatio::new(0.02)));
        calculator.add_metric(Box::new(ProfitFactor::new()));
        calculator.add_metric(Box::new(DrawDown::new()));
        
        let report = calculator.calculate_all(&trades, 10000.0);
        
        assert_eq!(report.results.len(), 3);
        assert!(report.results.contains_key("Sharpe Ratio"));
        assert!(report.results.contains_key("Profit Factor"));
        assert!(report.results.contains_key("Draw Down"));
    }
}
```

### Отчетность

#### Генерация отчетов
```rust
pub struct MetricReport {
    pub results: HashMap<String, MetricResult>,
    pub generated_at: DateTime<Utc>,
    pub trade_count: usize,
    pub initial_capital: f64,
}

impl MetricReport {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("Metric,Value,Unit,Interpretation,Is Good\n");
        
        for (name, result) in &self.results {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                name,
                result.value,
                result.unit,
                result.interpretation.as_deref().unwrap_or(""),
                result.is_good.map(|b| b.to_string()).as_deref().unwrap_or("")
            ));
        }
        
        csv
    }
    
    pub fn to_html(&self) -> String {
        // Generate HTML report with charts and tables
        todo!()
    }
}
```

## Интеграция с системой

### Связь с другими компонентами

#### Strategy Layer
```rust
pub trait Strategy {
    fn execute(&self, data: &MarketData) -> Vec<Trade>;
    fn get_metrics(&self) -> MetricReport;
}

impl Strategy for MyStrategy {
    fn get_metrics(&self) -> MetricReport {
        let trades = self.get_trades();
        let calculator = MetricCalculator::new();
        // Add all metrics
        calculator.calculate_all(&trades, self.initial_capital)
    }
}
```

#### Optimization Layer
```rust
pub trait FitnessFunction {
    fn calculate(&self, strategy: &dyn Strategy) -> f64;
}

pub struct MultiObjectiveFitness {
    weights: HashMap<String, f64>,
    calculator: MetricCalculator,
}

impl FitnessFunction for MultiObjectiveFitness {
    fn calculate(&self, strategy: &dyn Strategy) -> f64 {
        let report = strategy.get_metrics();
        let mut score = 0.0;
        
        for (metric_name, weight) in &self.weights {
            if let Some(result) = report.results.get(metric_name) {
                score += result.value * weight;
            }
        }
        
        score
    }
}
```

## Заключение

Данный план обеспечивает пошаговую реализацию полной системы метрик с учетом производительности, тестируемости и интеграции с остальными компонентами системы. Реализация разбита на 4 фазы с четкими приоритетами и оценками сложности.

*Модель: Claude 3.5 Sonnet*

