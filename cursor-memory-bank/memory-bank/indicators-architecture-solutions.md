# Архитектурные решения для модуля индикаторов

## 1. Решение проблем SRP

### 1.1 Разделить implementations.rs на отдельные файлы

**Структура:**
```
src/indicators/
├── mod.rs
├── base.rs                    # Только базовый trait Indicator
├── traits/
│   ├── mod.rs
│   ├── trend.rs               # TrendIndicator trait
│   ├── oscillator.rs          # OscillatorIndicator trait  
│   ├── volatility.rs          # VolatilityIndicator trait
│   └── optimizer.rs           # ParameterOptimizer trait
├── rules/
│   ├── mod.rs
│   ├── build_rules.rs         # IndicatorBuildRules
│   ├── price_compare.rs       # PriceCompareConfig
│   ├── indicator_compare.rs   # IndicatorCompareConfig
│   └── nesting.rs             # NestingConfig
├── impl/
│   ├── mod.rs
│   ├── trend/
│   │   ├── mod.rs
│   │   ├── sma.rs
│   │   ├── ema.rs
│   │   ├── wma.rs
│   │   ├── ama.rs
│   │   ├── zlema.rs
│   │   ├── geomean.rs
│   │   ├── amma.rs
│   │   ├── sqwma.rs
│   │   ├── sinewma.rs
│   │   ├── tpbf.rs
│   │   └── supertrend.rs
│   ├── oscillator/
│   │   ├── mod.rs
│   │   ├── rsi.rs
│   │   └── stochastic.rs
│   ├── volatility/
│   │   ├── mod.rs
│   │   ├── atr.rs
│   │   ├── watr.rs
│   │   ├── true_range.rs
│   │   └── vtrand.rs
│   ├── channel/
│   │   ├── mod.rs
│   │   ├── bollinger/
│   │   │   ├── mod.rs
│   │   │   ├── middle.rs
│   │   │   ├── upper.rs
│   │   │   └── lower.rs
│   │   └── keltner/
│   │       ├── mod.rs
│   │       ├── middle.rs
│   │       ├── upper.rs
│   │       └── lower.rs
│   └── auxiliary/
│       ├── mod.rs
│       ├── maxfor.rs
│       └── minfor.rs
├── registry/
│   ├── mod.rs
│   ├── registry.rs            # IndicatorRegistry
│   ├── factory.rs             # IndicatorFactory
│   └── info.rs                # IndicatorInfo
├── types.rs
├── parameters.rs
├── formula.rs
└── runtime.rs
```

### 1.2 Упростить base.rs

**До:**
```rust
pub trait Indicator: Send + Sync {
    // 19 методов
}
```

**После:**
```rust
pub trait Indicator: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> IndicatorCategory;
    fn indicator_type(&self) -> IndicatorType;
    fn parameters(&self) -> &ParameterSet;
    fn min_data_points(&self) -> usize;
    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync>;
}

pub trait SimpleCalculation: Indicator {
    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError>;
}

pub trait OHLCCalculation: Indicator {
    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;
}

pub trait IndicatorMetadataProvider: Indicator {
    fn description(&self) -> &str;
    fn metadata(&self) -> IndicatorMetadata { ... }
    fn build_rules(&self) -> IndicatorBuildRules { ... }
}
```

---

## 2. Решение проблем OCP

### 2.1 Автоматическая регистрация через inventory

**Использовать crate `inventory` или макросы для автоматической регистрации:**

```rust
use inventory;

pub trait IndicatorRegistration: Indicator {
    fn register_info() -> IndicatorInfo;
    fn create(params: HashMap<String, f32>) -> Result<Box<dyn Indicator + Send + Sync>, IndicatorError>;
}

inventory::collect!(IndicatorRegistration);

impl IndicatorFactory {
    pub fn create_indicator(name: &str, params: HashMap<String, f32>) 
        -> Result<Box<dyn Indicator + Send + Sync>, IndicatorError> 
    {
        for registration in inventory::iter::<dyn IndicatorRegistration> {
            if registration.register_info().name.eq_ignore_ascii_case(name) {
                return registration.create(params);
            }
        }
        Err(IndicatorError::InvalidParameter(format!("Unknown indicator: {}", name)))
    }
}
```

### 2.2 Макрос для регистрации индикаторов

```rust
#[macro_export]
macro_rules! register_indicator {
    ($type:ty, $name:expr, $category:expr, $ind_type:expr, $params:expr) => {
        inventory::submit! {
            IndicatorRegistrationEntry {
                name: $name,
                category: $category,
                indicator_type: $ind_type,
                parameters: $params,
                create: |params| Box::new(<$type>::from_params(params)?),
            }
        }
    };
}

register_indicator!(SMA, "SMA", IndicatorCategory::Trend, IndicatorType::Simple, &["period"]);
```

### 2.3 Использование derive macro

```rust
#[derive(Indicator)]
#[indicator(name = "SMA", category = "Trend", indicator_type = "Simple")]
pub struct SMA {
    #[parameter(range = "5..200", step = "1", default = "20")]
    period: f32,
}
```

---

## 3. Решение проблем ISP

### 3.1 Разделить Indicator trait

```rust
pub trait IndicatorCore: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> IndicatorCategory;
    fn indicator_type(&self) -> IndicatorType;
    fn parameters(&self) -> &ParameterSet;
    fn min_data_points(&self) -> usize;
}

pub trait SimpleIndicator: IndicatorCore {
    fn calculate(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError>;
}

pub trait OHLCIndicator: IndicatorCore {
    fn calculate(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;
}

pub trait UniversalIndicator: SimpleIndicator + OHLCIndicator {}
```

### 3.2 Убрать заглушки

**Вместо:**
```rust
impl Indicator for Stochastic {
    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }
}
```

**Использовать:**
```rust
impl OHLCIndicator for Stochastic {
    fn calculate(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // ...
    }
}
// Не реализовывать SimpleIndicator вообще
```

---

## 4. Решение проблем DIP

### 4.1 Внедрение зависимостей через конструктор

```rust
pub struct KCUpper<M: OHLCIndicator, A: OHLCIndicator> {
    parameters: ParameterSet,
    middle_indicator: M,
    atr_indicator: A,
}

impl KCUpper<KCMiddle, ATR> {
    pub fn new(period: f32, atr_period: f32, multiplier: f32) -> Result<Self, IndicatorError> {
        Ok(Self {
            parameters: create_kc_params(period, atr_period, multiplier)?,
            middle_indicator: KCMiddle::new(period)?,
            atr_indicator: ATR::new(atr_period)?,
        })
    }
}

impl<M: OHLCIndicator, A: OHLCIndicator> KCUpper<M, A> {
    pub fn with_dependencies(
        params: ParameterSet,
        middle: M,
        atr: A,
    ) -> Self {
        Self { parameters: params, middle_indicator: middle, atr_indicator: atr }
    }
}
```

### 4.2 Использование trait objects для композитных индикаторов

```rust
pub struct CompositeIndicator {
    dependencies: HashMap<String, Box<dyn OHLCIndicator>>,
    compute: Box<dyn Fn(&HashMap<String, Vec<f32>>) -> Vec<f32>>,
}
```

---

## 5. Устранение дублирования кода

### 5.1 Дефолтная реализация get_trend_direction

```rust
pub trait TrendIndicator: SimpleIndicator {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        default_trend_direction(self.calculate(data)?)
    }
}

fn default_trend_direction(values: Vec<f32>) -> Result<TrendDirection, IndicatorError> {
    match (values.last(), values.get(values.len().saturating_sub(2))) {
        (Some(last), Some(prev)) => Ok(match last.partial_cmp(prev) {
            Some(std::cmp::Ordering::Greater) => TrendDirection::Up,
            Some(std::cmp::Ordering::Less) => TrendDirection::Down,
            _ => TrendDirection::Sideways,
        }),
        _ => Ok(TrendDirection::Unknown),
    }
}
```

### 5.2 Генерик конструктор для однопараметровых индикаторов

```rust
pub trait SinglePeriodIndicator: Sized {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    
    fn from_params(params: ParameterSet) -> Self;
    
    fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params.add_parameter(create_period_parameter(
            "period", 
            period, 
            &format!("Период для расчета {}", Self::NAME)
        )).map_err(|e| IndicatorError::InvalidParameter(e))?;
        Ok(Self::from_params(params))
    }
}

macro_rules! impl_single_period {
    ($type:ident) => {
        impl SinglePeriodIndicator for $type {
            const NAME: &'static str = stringify!($type);
            const DESCRIPTION: &'static str = concat!(stringify!($type), " indicator");
            
            fn from_params(params: ParameterSet) -> Self {
                Self { parameters: params }
            }
        }
    };
}

impl_single_period!(SMA);
impl_single_period!(EMA);
impl_single_period!(WMA);
// ...
```

### 5.3 Trait для автоматической реализации calculate_ohlc

```rust
pub trait SimpleToOHLC: SimpleIndicator {
    fn ohlc_field(&self) -> OHLCField { OHLCField::Close }
    
    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let simple_data = match self.ohlc_field() {
            OHLCField::Open => &data.open,
            OHLCField::High => &data.high,
            OHLCField::Low => &data.low,
            OHLCField::Close => &data.close,
        };
        self.calculate(simple_data)
    }
}

pub enum OHLCField { Open, High, Low, Close }
```

### 5.4 Убрать дублирование структур IndicatorMetadata и IndicatorInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorInfo {
    pub name: String,
    pub category: IndicatorCategory,
    pub indicator_type: IndicatorType,
    pub description: String,
    pub parameters: Vec<ParameterInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

pub type IndicatorMetadata = IndicatorInfo;
```

### 5.5 Унифицированная функция создания параметров

```rust
pub fn create_single_period_parameters(
    indicator_name: &str,
    period: f32,
) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, &format!("Период для расчета {}", indicator_name)),
    );
    params
}
```

---

## 6. Решение других архитектурных проблем

### 6.1 Документировать magic numbers

```rust
impl AMA {
    const FAST_SMOOTHING: f32 = 2.0 / (2.0 + 1.0);   // 0.6667
    const SLOW_SMOOTHING: f32 = 2.0 / (30.0 + 1.0);  // 0.0645
    const FAST_FACTOR: f32 = Self::FAST_SMOOTHING - Self::SLOW_SMOOTHING; // ~0.60215
    const SLOW_FACTOR: f32 = Self::SLOW_SMOOTHING;                         // ~0.06452
    
    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        // ...
        let x = efficiency * Self::FAST_FACTOR + Self::SLOW_FACTOR;
        // ...
    }
}
```

### 6.2 Убрать async из примеров (или сделать методы async)

```rust
pub fn bollinger_bands_example() -> Result<(), String> {
    let bb_middle = BBMiddle::new(20.0, 2.0).map_err(|e| e.to_string())?;
    let middle_values = bb_middle.calculate_ohlc(&ohlc_data).map_err(|e| e.to_string())?;
    // ...
}
```

### 6.3 Заменить EmptyIndicator на Result

```rust
impl CloneBox for Box<dyn Indicator + Send + Sync> {
    fn clone_box(&self) -> Result<Box<dyn Indicator + Send + Sync>, IndicatorError> {
        let name = self.name();
        let parameters = self.parameters().get_current_values();
        IndicatorFactory::create_indicator(name, parameters)
    }
}
```

### 6.4 Вынести общую логику инициализации

```rust
fn initialize_with_zeros(len: usize, period: usize) -> Vec<f32> {
    let warmup = period.saturating_sub(1);
    let mut result = vec![0.0; warmup];
    result.reserve(len - warmup);
    result
}
```

---

## 7. Улучшение тестируемости

### 7.1 Trait для зависимостей

```rust
pub trait ATRProvider {
    fn calculate(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;
}

impl ATRProvider for ATR {
    fn calculate(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_ohlc(data)
    }
}

pub struct KCUpper<A: ATRProvider = ATR> {
    atr_provider: A,
    // ...
}
```

### 7.2 Builder pattern для тестов

```rust
#[cfg(test)]
pub struct KCUpperBuilder {
    period: f32,
    atr_period: f32,
    multiplier: f32,
    atr_provider: Option<Box<dyn ATRProvider>>,
}

#[cfg(test)]
impl KCUpperBuilder {
    pub fn with_mock_atr(mut self, mock: impl ATRProvider + 'static) -> Self {
        self.atr_provider = Some(Box::new(mock));
        self
    }
}
```

---

## 8. Улучшение расширяемости

### 8.1 Plugin система

```rust
pub trait IndicatorPlugin {
    fn name(&self) -> &str;
    fn info(&self) -> IndicatorInfo;
    fn create(&self, params: HashMap<String, f32>) -> Result<Box<dyn Indicator>, IndicatorError>;
}

impl IndicatorRegistry {
    pub fn register_plugin(&mut self, plugin: Box<dyn IndicatorPlugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }
}
```

### 8.2 Декларативное определение индикаторов

```rust
indicator! {
    name: "SMA",
    category: Trend,
    params: {
        period: f32 = 20.0 [5.0..200.0, step: 1.0]
    },
    calculate_simple(data, period) {
        let mut result = Vec::with_capacity(data.len());
        // ...
        result
    }
}
```

---

## 9. Оптимизация производительности

### 9.1 Кеширование вложенных индикаторов

```rust
pub struct BollingerBands {
    sma: SMA,
    cached_sma: Option<Vec<f32>>,
    cached_std_dev: Option<Vec<f32>>,
}

impl BollingerBands {
    pub fn middle(&mut self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        if self.cached_sma.is_none() {
            self.compute_base(data)?;
        }
        Ok(self.cached_sma.clone().unwrap())
    }
    
    pub fn upper(&mut self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        if self.cached_sma.is_none() {
            self.compute_base(data)?;
        }
        let deviation = self.parameters.get_value("deviation").unwrap();
        Ok(self.cached_sma.as_ref().unwrap().iter()
            .zip(self.cached_std_dev.as_ref().unwrap())
            .map(|(sma, std)| sma + deviation * std)
            .collect())
    }
}
```

### 9.2 Ленивые вычисления

```rust
pub struct LazyIndicator<I: Indicator> {
    indicator: I,
    cache: RefCell<Option<Vec<f32>>>,
    data_hash: RefCell<Option<u64>>,
}

impl<I: Indicator> LazyIndicator<I> {
    pub fn get(&self, data: &[f32]) -> Result<&Vec<f32>, IndicatorError> {
        let hash = calculate_hash(data);
        if self.data_hash.borrow().map_or(true, |h| h != hash) {
            let result = self.indicator.calculate_simple(data)?;
            *self.cache.borrow_mut() = Some(result);
            *self.data_hash.borrow_mut() = Some(hash);
        }
        Ok(self.cache.borrow().as_ref().unwrap())
    }
}
```

---

## 10. Приоритеты внедрения

### Высокий приоритет (устранить сразу):
1. Разделить implementations.rs на файлы по категориям
2. Вынести дублирующуюся реализацию get_trend_direction в default метод
3. Создать макрос для однопараметровых индикаторов
4. Объединить IndicatorMetadata и IndicatorInfo

### Средний приоритет:
5. Разделить trait Indicator на ISP-совместимые части
6. Внедрить DI для композитных индикаторов
7. Добавить автоматическую регистрацию индикаторов
8. Документировать magic numbers

### Низкий приоритет:
9. Plugin система
10. Декларативные макросы для определения индикаторов
11. Кеширование вложенных вычислений

