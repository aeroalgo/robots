# Архитектурные проблемы модуля индикаторов

## 1. Нарушение Single Responsibility Principle (SRP)

### 1.1 Файл implementations.rs (~2800 строк)
Содержит 30+ индикаторов в одном файле. Каждый индикатор - отдельная сущность со своей логикой.

**Проблемные места:**
- MAXFOR, MINFOR, ATR, SuperTrend, Stochastic, TrueRange, WATR, VTRAND
- SMA, EMA, RSI, WMA, AMA, ZLEMA, GEOMEAN, AMMA, SQWMA, SINEWMA, TPBF
- BBMiddle, BBUpper, BBLower, KCMiddle, KCUpper, KCLower

### 1.2 Файл base.rs (~756 строк)
Смешивает несколько ответственностей:
- Trait `Indicator` (основной интерфейс)
- Traits `TrendIndicator`, `OscillatorIndicator`, `VolatilityIndicator`
- Traits `SimpleIndicator`, `OHLCIndicator`
- Struct `OverboughtOversoldZones`
- Enum `TrendDirection`
- Trait `ParameterOptimizer` с реализацией
- `IndicatorBuildRules` и связанные конфиги (PriceCompareConfig, IndicatorCompareConfig, NestingConfig, ThresholdType)

### 1.3 Файл registry.rs (~827 строк)
Смешивает:
- `IndicatorRegistry` (реестр)
- `IndicatorFactory` (фабрика)
- `IndicatorInfo` (информация)
- `IndicatorConfig` (конфигурация)
- `RegistryStats` (статистика)
- `EmptyIndicator` (заглушка)
- `CloneBox` trait
- `registry_utils` module

---

## 2. Нарушение Open/Closed Principle (OCP)

### 2.1 Hardcoded список индикаторов в фабрике

```rust
// registry.rs:331-453
impl IndicatorFactory {
    pub fn create_indicator(name: &str, parameters: HashMap<String, f32>) 
        -> Result<Box<dyn Indicator + Send + Sync>, IndicatorError> {
        match name.to_uppercase().as_str() {
            "ATR" => { ... }
            "SMA" => { ... }
            // ... 20+ вариантов
            _ => Err(...)
        }
    }
}
```

**Проблема:** Добавление нового индикатора требует:
1. Изменения `create_indicator()`
2. Изменения `get_indicator_info()`
3. Изменения `get_available_indicators()`
4. Изменения `register_all_indicators()`

### 2.2 Дублирование информации об индикаторах

```rust
// registry.rs:499-700 - get_indicator_info() 
// 200 строк match с дублированием
match name.to_uppercase().as_str() {
    "ATR" => Some(IndicatorInfo { ... }),
    "SMA" => Some(IndicatorInfo { ... }),
    // ...
}
```

---

## 3. Нарушение Interface Segregation Principle (ISP)

### 3.1 Слишком большой trait Indicator

```rust
// base.rs:8-178
pub trait Indicator: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> IndicatorCategory;
    fn indicator_type(&self) -> IndicatorType;
    fn parameters(&self) -> &ParameterSet;
    fn min_data_points(&self) -> usize;
    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError>;
    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;
    fn calculate(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError>;
    fn calculate_with_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;
    fn metadata(&self) -> IndicatorMetadata;
    fn build_rules(&self) -> IndicatorBuildRules;
    fn get_required_input_type(&self) -> InputDataType;
    fn validate_parameters(&self) -> Result<(), IndicatorError>;
    fn validate_input_data(&self, data: &[f32]) -> Result<(), IndicatorError>;
    fn validate_ohlc_data(&self, data: &OHLCData) -> Result<(), IndicatorError>;
    fn calculate_with_metadata(&self, data: &[f32]) -> Result<IndicatorResultData, IndicatorError>;
    fn calculate_ohlc_with_metadata(&self, data: &OHLCData) -> Result<IndicatorResultData, IndicatorError>;
    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync>;
}
```

### 3.2 Принудительная реализация ненужных методов

```rust
// implementations.rs - OHLC индикаторы обязаны реализовывать calculate_simple
impl Indicator for Stochastic {
    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)  // Бесполезная заглушка
    }
}

// Simple индикаторы реализуют calculate_ohlc через calculate_simple
impl Indicator for SMA {
    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)  // Дублирование логики
    }
}
```

---

## 4. Нарушение Dependency Inversion Principle (DIP)

### 4.1 Жесткие зависимости между индикаторами

```rust
// implementations.rs:509-518 - SuperTrend создает WATR напрямую
fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
    let watr_indicator = WATR::new_unchecked(period as f32);  // Жесткая зависимость
    let atr_values = watr_indicator.calculate_ohlc(data)?;
    // ...
}

// implementations.rs:902-920 - VTRAND создает MAXFOR и MINFOR
fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
    let max_indicator = MAXFOR::new_unchecked(period as f32);  // Жесткая зависимость
    let min_indicator = MINFOR::new_unchecked(period as f32);  // Жесткая зависимость
    // ...
}

// implementations.rs:2658-2674 - KCUpper создает KCMiddle и ATR
fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
    let middle_indicator = KCMiddle::new(ema_period as f32).unwrap();  // Жесткая зависимость
    let atr_indicator = ATR::new(atr_period as f32).unwrap();  // Жесткая зависимость
    // ...
}
```

---

## 5. Дублирование кода

### 5.1 Идентичная реализация get_trend_direction

```rust
// Эта реализация повторяется 12+ раз для разных индикаторов:
impl TrendIndicator for SMA { ... }  // implementations.rs:1043-1062
impl TrendIndicator for EMA { ... }  // implementations.rs:1159-1178
impl TrendIndicator for WMA { ... }  // implementations.rs:1415-1434
impl TrendIndicator for AMA { ... }  // implementations.rs:1536-1555
impl TrendIndicator for ZLEMA { ... } // implementations.rs:1643-1662
impl TrendIndicator for GEOMEAN { ... } // implementations.rs:1746-1765
impl TrendIndicator for AMMA { ... } // implementations.rs:1853-1872
impl TrendIndicator for SQWMA { ... } // implementations.rs:1969-1988
impl TrendIndicator for SINEWMA { ... } // implementations.rs:2079-2098
impl TrendIndicator for TPBF { ... } // implementations.rs:2213-2232
impl TrendIndicator for SuperTrend { ... } // implementations.rs:573-592
```

**Все реализации идентичны:**
```rust
fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
    let values = self.calculate_simple(data)?;
    if let Some(last_value) = values.last() {
        if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
            if last_value > prev_value { Ok(TrendDirection::Up) }
            else if last_value < prev_value { Ok(TrendDirection::Down) }
            else { Ok(TrendDirection::Sideways) }
        } else { Ok(TrendDirection::Unknown) }
    } else { Ok(TrendDirection::Unknown) }
}
```

### 5.2 Однотипные конструкторы индикаторов с одним параметром

```rust
// Этот паттерн повторяется 15+ раз:
impl SMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params.add_parameter(create_period_parameter("period", period, "Период для расчета SMA"))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        Ok(Self { parameters: params })
    }
}
```

### 5.3 Дублирование вычисления SMA в BBMiddle

```rust
// implementations.rs:2285-2307 - BBMiddle
fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
    // ... вручную считает SMA вместо использования SMA индикатора
    let sum: f32 = data[start..=i].iter().sum();
    let sma = sum / current_window as f32;
}

// implementations.rs:983-1005 - SMA
fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
    // ... та же логика
    let sum: f32 = unsafe_ops::sum_f32_fast(&data[start..=i]);
    sma_values.push(sum / current_window as f32);
}
```

### 5.4 Дублирование функций создания параметров

```rust
// parameters.rs - практически идентичные функции:
pub fn create_sma_parameters(period: f32) -> HashMap<String, IndicatorParameter> { ... }
pub fn create_ema_parameters(period: f32) -> HashMap<String, IndicatorParameter> { ... }
pub fn create_rsi_parameters(period: f32) -> HashMap<String, IndicatorParameter> { ... }
pub fn create_atr_parameters(period: f32) -> HashMap<String, IndicatorParameter> { ... }
pub fn create_wma_parameters(period: f32) -> HashMap<String, IndicatorParameter> { ... }
pub fn create_ama_parameters(period: f32) -> HashMap<String, IndicatorParameter> { ... }
pub fn create_zlema_parameters(period: f32) -> HashMap<String, IndicatorParameter> { ... }
// ... и т.д.
```

### 5.5 Дублирование структур данных

```rust
// types.rs:219-232
pub struct IndicatorMetadata {
    pub name: String,
    pub category: IndicatorCategory,
    pub indicator_type: IndicatorType,
    // ...
}

// registry.rs:703-710
pub struct IndicatorInfo {
    pub name: String,
    pub category: IndicatorCategory,
    pub indicator_type: IndicatorType,
    // ...
}
```

---

## 6. Другие архитектурные проблемы

### 6.1 Magic numbers без пояснений

```rust
// implementations.rs:1514-1517 - AMA
let x = efficiency * 0.60215 + 0.06452;  // Что это за числа?
let smoothing = x * x;
```

### 6.2 Смешение синхронного и асинхронного кода

```rust
// examples.rs - использует .await с синхронными методами
let middle_values = bb_middle.calculate_ohlc(&ohlc_data).await.map_err(|e| e.to_string())?;
// Но calculate_ohlc - синхронный метод!
```

### 6.3 EmptyIndicator как antipattern

```rust
// registry.rs:793-826
struct EmptyIndicator;
impl Indicator for EmptyIndicator {
    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Ok(vec![0.0])  // Возвращает невалидные данные
    }
}
```

### 6.4 Избыточные проверки длины

```rust
// Паттерн повторяется в каждом индикаторе:
let len = data.len();
let Some(period) = adjust_period(period, len) else {
    return Ok(Vec::new());
};
// ... потом снова проверки len
```

### 6.5 Неконсистентное использование unwrap

```rust
// В одних местах:
let period = self.parameters.get_value("period").unwrap() as usize;

// В других:
let period = parameters.get("period").copied().unwrap_or(14.0);
```

---

## 7. Нарушение DRY (Don't Repeat Yourself)

### 7.1 Повторяющийся цикл инициализации нулями

```rust
// Этот паттерн встречается 15+ раз:
for _ in 0..period.saturating_sub(1) {
    values.push(0.0);
}
```

### 7.2 Повторяющаяся логика calculate_ohlc для Simple индикаторов

```rust
// Каждый Simple индикатор реализует:
fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
    self.calculate_simple(&data.close)
}
```

---

## 8. Проблемы с тестируемостью

### 8.1 Жесткие зависимости мешают unit-тестированию

Невозможно тестировать KCUpper без реального ATR и KCMiddle.

### 8.2 Отсутствие моков для зависимостей

Нет интерфейсов для подмены зависимых индикаторов в тестах.

---

## 9. Проблемы с расширяемостью

### 9.1 Добавление нового типа индикатора требует изменения многих файлов

1. Создать struct в implementations.rs
2. Добавить в match в IndicatorFactory::create_indicator
3. Добавить в IndicatorFactory::get_indicator_info
4. Добавить в IndicatorFactory::get_available_indicators
5. Добавить в IndicatorRegistry::register_all_indicators
6. Создать функцию create_X_parameters в parameters.rs

### 9.2 Невозможность плагинной архитектуры

Нет механизма для регистрации пользовательских индикаторов без изменения исходного кода.

---

## 10. Проблемы с производительностью

### 10.1 Излишние аллокации

```rust
// Создание нового индикатора на каждый вызов:
let middle_indicator = KCMiddle::new(ema_period as f32).unwrap();
let atr_indicator = ATR::new(atr_period as f32).unwrap();
```

### 10.2 Отсутствие кеширования промежуточных результатов

Bollinger Bands Upper и Lower пересчитывают SMA и std_dev независимо, хотя могли бы переиспользовать результаты.

