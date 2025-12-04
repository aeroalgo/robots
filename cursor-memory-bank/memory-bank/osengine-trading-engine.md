# Анализ торгового движка OsEngine

## Обзор

Документация описывает архитектуру и работу торгового движка OsEngine, путь данных от получения свечей до отправки приказов на биржу, а также анализ возможностей переиспользования кода из нашего проекта.

## Путь данных: от свечи до приказа

### 1. Получение данных от поставщика

#### 1.1. Серверы данных (Market/Servers)
OsEngine использует абстракцию `IServer` для подключения к различным биржам и источникам данных.

**Основные компоненты:**
- `AServer` - базовый класс для всех серверов
- Конкретные реализации: `BinanceServer`, `TinkoffServer`, `AlorServer` и т.д.
- Каждый сервер реализует методы:
  - `Connect()` - подключение к бирже
  - `GetSecurities()` - получение списка инструментов
  - `SubscribeMarketData()` - подписка на данные
  - `SendOrder()` - отправка приказа
  - `CancelOrder()` - отмена приказа

**События сервера:**
- `NewCandleIncomeEvent` - новая свеча
- `NewTradeEvent` - новый тик
- `NewMarketDepthEvent` - обновление стакана
- `TimeServerChangeEvent` - изменение времени сервера

#### 1.2. CandleManager
`CandleManager` управляет получением и обработкой свечей от сервера.

**Процесс:**
1. Сервер получает данные от биржи (WebSocket/REST)
2. Сервер вызывает `NewCandleIncomeEvent` или `NewTradeEvent`
3. `CandleManager` подписывается на эти события
4. `CandleManager` передает данные в `CandleSeries`

**Код:**
```csharp
// Market/Servers/AServer.cs
private void LowPriorityDataThreadArea()
{
    if (!_candleSeriesToSend.IsEmpty)
    {
        CandleSeries series;
        while (_candleSeriesToSend.TryDequeue(out series))
        {
            if (NewCandleIncomeEvent != null)
            {
                NewCandleIncomeEvent(series);
            }
        }
    }
}
```

### 2. Обработка свечей

#### 2.1. CandleSeries
`CandleSeries` - объект, который собирает свечи для конкретного инструмента и таймфрейма.

**Основные функции:**
- Хранение всех свечей (`CandlesAll`)
- Хранение завершенных свечей (`CandlesFinishedOnly`)
- Создание свечей из тиков или стакана
- Уведомление о завершении свечи

**События:**
- `CandleFinishedEvent` - свеча завершена
- `CandleUpdateEvent` - свеча обновлена (текущая формирующаяся)

**Код:**
```csharp
// Candles/CandleSeries.cs
private void TimeFrameBuilder_CandleFinishedEvent(CandleSeries candleSeries)
{
    // Вызывается когда свеча завершена
    // Уведомляет подписчиков
}
```

#### 2.2. ConnectorCandles
`ConnectorCandles` - универсальный интерфейс для подключения к серверам для ботов.

**Основные функции:**
- Подписка на инструмент
- Получение свечей от сервера
- Управление подписками
- Эмулятор исполнения приказов

**События:**
- `NewCandlesChangeEvent` - новые завершенные свечи
- `LastCandlesChangeEvent` - обновление текущей свечи
- `OrderChangeEvent` - изменение приказа
- `MyTradeEvent` - исполнение сделки

**Код:**
```csharp
// Market/Connectors/ConnectorCandles.cs
private void MySeries_CandleFinishedEvent(CandleSeries candleSeries)
{
    List<Candle> candles = Candles(true);
    if (NewCandlesChangeEvent != null)
    {
        NewCandlesChangeEvent(candles);
    }
}
```

### 3. Передача в стратегию

#### 3.1. BotTabSimple
`BotTabSimple` - основная панель для торговли, которая связывает данные с стратегией.

**Процесс:**
1. `BotTabSimple` создает `ConnectorCandles`
2. Подписывается на события `ConnectorCandles`:
   - `NewCandlesChangeEvent` → `LogicToEndCandle`
   - `LastCandlesChangeEvent` → `LogicToUpdateLastCandle`
3. При получении свечей вызывает событие `CandleFinishedEvent`
4. Стратегия подписывается на `CandleFinishedEvent`

**Код:**
```csharp
// OsTrader/Panels/Tab/BotTabSimple.cs
public BotTabSimple(string name, StartProgram startProgram)
{
    _connector = new ConnectorCandles(TabName, startProgram, true);
    _connector.NewCandlesChangeEvent += LogicToEndCandle;
    _connector.LastCandlesChangeEvent += LogicToUpdateLastCandle;
}

private void LogicToEndCandle(List<Candle> candles)
{
    // Обновление графика
    if (_chartMaster != null)
    {
        _chartMaster.SetCandles(candles);
    }
    
    // Вызов события для стратегии
    CandleFinishedEvent?.Invoke(candles);
}
```

#### 3.2. Стратегия
Стратегия подписывается на событие `CandleFinishedEvent` и получает список свечей.

**Пример стратегии:**
```csharp
// Robots/BotsFromStartLessons/Lesson5Bot1.cs
private void _tabToTrade_CandleFinishedEvent(List<Candle> candles)
{
    if (_mode.ValueString == "Off")
    {
        return;
    }
    
    if (candles.Count < 10)
    {
        return;
    }
    
    // Логика стратегии
    List<Position> positions = _tabToTrade.PositionsOpenAll;
    if (positions.Count == 0)
    {
        LogicOpenPosition(candles, positions);
    }
}
```

### 4. Расчет индикаторов

#### 4.1. Система индикаторов
Индикаторы в OsEngine реализуют интерфейс `IIndicator` и наследуются от `Aindicator`.

**Основные методы:**
- `Process(List<Candle> candles)` - обработка свечей
- `ProcessAll()` - полный пересчет
- `ProcessNew()` - обработка новой свечи
- `ProcessLast()` - обновление последней свечи

#### 4.2. Инкрементальное обновление
OsEngine предоставляет механизм инкрементального обновления индикаторов для оптимизации производительности, но **не все индикаторы его поддерживают**.

**Важно:** Базовый класс `Aindicator` всегда вызывает `OnProcess(candles, index)` с конкретным индексом, но каждый индикатор сам решает, как использовать этот индекс. Некоторые индикаторы могут игнорировать индекс и пересчитывать все значения, если их алгоритм требует этого.

**Три режима обновления:**

1. **ProcessAll** - полный пересчет:
   - Когда изменяется первая свеча
   - Когда количество свечей уменьшается
   - При первом запуске
   - Всегда вызывает `OnProcess` для всех индексов

2. **ProcessNew** - обработка новой свечи:
   - Когда добавляется одна новая свеча
   - Вызывает `OnProcess(candles, index)` только для нового индекса
   - Индикатор может использовать индекс для инкрементального расчета или пересчитать все значения

3. **ProcessLast** - обновление последней свечи:
   - Когда обновляется текущая формирующаяся свеча
   - Вызывает `OnProcess(candles, candles.Count - 1)` для последнего индекса
   - Индикатор может обновить только последнее значение или пересчитать все

**Какие индикаторы поддерживают инкрементальное обновление:**

✅ **Поддерживают (могут обновлять только последнее значение):**
- SMA, EMA, WMA - скользящие средние (могут пересчитать только последнее значение)
- RSI - может обновить только последнее значение, если хранит предыдущие значения
- Простые индикаторы с фиксированным окном

❌ **Не поддерживают (требуют полного пересчета):**
- Индикаторы, зависящие от всех предыдущих значений (например, накопительные)
- Индикаторы с динамическим окном
- Сложные индикаторы с множественными зависимостями
- Индикаторы, использующие оптимизацию или поиск паттернов

**Реализация в OsEngine:**
```csharp
// Базовый класс всегда вызывает OnProcess с индексом
// ВАЖНО: Передается ВЕСЬ массив свечей, а не срез!
private void ProcessNew(List<Candle> candles, int index)
{
    // ... подготовка данных ...
    OnProcess(candles, index);  // Передается весь массив + индекс
}

// Индикатор использует индекс для доступа к нужным данным
public override void OnProcess(List<Candle> candles, int index)
{
    // Для SMA с периодом 20 нужно взять срез от index-19 до index
    if (index >= period - 1)
    {
        // Используем срез candles[index - period + 1..=index]
        decimal sum = 0;
        for (int i = index - period + 1; i <= index; i++)
        {
            sum += candles[i].Close;
        }
        _series.Values[index] = sum / period;
    }
}
```

**Важные детали:**

1. **Передается весь массив свечей**, а не срез:
   - `OnProcess(candles, index)` получает весь массив `candles`
   - Индикатор сам выбирает нужный диапазон через индекс

2. **Для инкрементального обновления нужен срез равный периоду:**
   - SMA(20): нужны свечи от `index - 19` до `index` (20 свечей)
   - EMA(20): можно использовать только предыдущее значение EMA + новая цена
   - RSI(14): нужны свечи от `index - 13` до `index` (14 свечей) + предыдущие значения RSI

3. **Пример для SMA:**
   ```rust
   // Для пересчета последнего значения SMA(20) при index=100
   // Нужен срез данных: candles[81..=100] (20 свечей)
   let start = index.saturating_sub(period - 1);
   let window = &data[start..=index];
   let sum: f32 = window.iter().sum();
   let sma_value = sum / window.len() as f32;
   ```

4. **Пример для EMA (более эффективный):**
   ```rust
   // EMA можно пересчитать только с предыдущим значением
   // Не нужно передавать весь период!
   let prev_ema = cached_ema_values[index - 1];
   let new_price = data[index];
   let multiplier = 2.0 / (period + 1.0);
   let new_ema = prev_ema + multiplier * (new_price - prev_ema);
   ```

**Вывод:** 
- Для большинства индикаторов (SMA, RSI и т.д.) нужно передавать срез данных равный периоду
- Для EMA и подобных индикаторов можно использовать только предыдущее значение + новая цена
- В OsEngine всегда передается весь массив свечей, индикатор сам выбирает нужный диапазон

**В нашем проекте:**
- Текущие индикаторы всегда выполняют полный пересчет через `calculate_simple()` или `calculate_ohlc()`
- Для поддержки инкрементального обновления нужно:
  - Добавить методы `update_incremental()` в трейт `Indicator`
  - Реализовать инкрементальную логику для каждого индикатора
  - Добавить флаг `supports_incremental` для указания поддержки

**Код:**
```csharp
// Indicators/Aindicator.cs
public void Process(List<Candle> candles)
{
    if (_myCandles == null ||
        candles.Count < _myCandles.Count ||
        candles.Count > _myCandles.Count + 1 ||
        (_lastFirstCandle != null && _lastFirstCandle.TimeStart != candles[0].TimeStart))
    {
        ProcessAll(candles);  // Полный пересчет
    }
    else if (_myCandles.Count == candles.Count)
    {
        ProcessLast(candles);  // Обновление последней
    }
    else if (_myCandles.Count + 1 == candles.Count)
    {
        ProcessNew(candles, candles.Count - 1);  // Новая свеча
    }
}
```

#### 4.3. Интервал обновления
В режиме реальной торговли (`IsOsTrader`) индикаторы могут обновляться с заданным интервалом.

**Код:**
```csharp
if (StartProgram == StartProgram.IsOsTrader
   && UpdateIntervalInSeconds != 0)
{
    if (_nextUpdateIndicatorsTime > DateTime.Now
        && _lastUpdateCandleTime == candles[^1].TimeStart)
    {
        return;  // Пропускаем обновление
    }
    _nextUpdateIndicatorsTime = DateTime.Now.AddSeconds(UpdateIntervalInSeconds);
    _lastUpdateCandleTime = candles[^1].TimeStart;
}
```

### 5. Проверка условий

#### 5.1. Условия в стратегии
Условия проверяются в методе стратегии, который вызывается при событии `CandleFinishedEvent`.

**Типы проверок:**
- **Полная проверка** - все условия проверяются на каждой свече
- **Инкрементальная проверка** - проверяются только изменившиеся условия

**Пример:**
```csharp
private void LogicOpenPosition(List<Candle> candles, List<Position> position)
{
    decimal lastSma = _sma.DataSeries[0].Last;
    Candle candle = candles[candles.Count - 1];
    
    // Проверка условий
    if (lowCandle < lastSma && closeCandle > lastSma)
    {
        // Условие выполнено - открываем позицию
        _tabToTrade.BuyAtMarket(volume);
    }
}
```

#### 5.2. Проверка срезами
OsEngine не использует явную систему "срезов" для условий. Вместо этого:
- Условия проверяются на каждой завершенной свече
- Индикаторы обновляются инкрементально
- Стратегия получает полный список свечей и значения индикаторов

### 6. Создание и отправка приказов

#### 6.1. Методы создания приказов
`BotTabSimple` предоставляет методы для создания приказов:

- `BuyAtMarket(volume)` - покупка по рынку
- `SellAtMarket(volume)` - продажа по рынку
- `BuyAtLimit(volume, price)` - покупка по лимиту
- `SellAtLimit(volume, price)` - продажа по лимиту

**Код:**
```csharp
// OsTrader/Panels/Tab/BotTabSimple.cs
public Position BuyAtMarket(decimal volume)
{
    // Проверка подключения
    if (_connector.IsConnected == false || _connector.IsReadyToTrade == false)
    {
        return null;
    }
    
    // Создание позиции
    Position newDeal = _dealCreator.CreatePosition(...);
    
    // Отправка приказа
    _connector.OrderExecute(newDeal.OpenOrders[0]);
    
    return newDeal;
}
```

#### 6.2. ConnectorCandles.OrderExecute
`ConnectorCandles` передает приказ серверу через метод `OrderExecute`.

**Код:**
```csharp
// Market/Connectors/ConnectorCandles.cs
public void OrderExecute(Order order)
{
    if (_myServer == null)
    {
        return;
    }
    
    // Отправка на сервер
    _myServer.SendOrder(order);
}
```

#### 6.3. Отправка на биржу
Сервер преобразует приказ в формат API биржи и отправляет его.

**Код:**
```csharp
// Market/Servers/TraderNet/TraderNetServer.cs
public void SendOrder(Order order)
{
    _rateGateSendOrder.WaitToProceed();  // Rate limiting
    
    Dictionary<string, dynamic> paramsDict = new Dictionary<string, dynamic>();
    paramsDict.Add("instr_name", order.SecurityNameCode);
    paramsDict.Add("action_id", order.Side == Side.Buy ? "1" : "3");
    paramsDict.Add("order_type_id", order.TypeOrder == OrderPriceType.Market ? "1" : "2");
    paramsDict.Add("qty", order.Volume.ToString());
    paramsDict.Add("limit_price", order.Price.ToString());
    
    // Отправка HTTP запроса
    HttpResponseMessage responseMessage = CreateAuthQuery("/api/v2/cmd/putTradeOrder", "POST", null, data);
}
```

## Архитектура торгового движка

### Компоненты системы

```
┌─────────────────┐
│   Биржа/API     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Server (AServer)│
│  - Connect()     │
│  - SendOrder()   │
│  - Events        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  CandleManager   │
│  - NewCandles    │
│  - NewTrades     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  CandleSeries    │
│  - CandlesAll    │
│  - FinishedOnly  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ ConnectorCandles │
│  - Subscribe     │
│  - Events        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  BotTabSimple    │
│  - CandleEvents  │
│  - OrderMethods  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Strategy       │
│  - CandleFinished│
│  - Logic         │
└─────────────────┘
```

### Поток данных

1. **Получение данных:**
   - Биржа → Server → CandleManager → CandleSeries

2. **Обработка свечей:**
   - CandleSeries → ConnectorCandles → BotTabSimple → Strategy

3. **Расчет индикаторов:**
   - Strategy → Indicators.Process() → DataSeries

4. **Проверка условий:**
   - Strategy проверяет условия на основе индикаторов

5. **Создание приказов:**
   - Strategy → BotTabSimple.BuyAtMarket() → ConnectorCandles.OrderExecute()

6. **Отправка приказов:**
   - ConnectorCandles → Server.SendOrder() → Биржа

## Особенности реализации

### 1. Событийная архитектура
OsEngine использует событийную модель для связи компонентов:
- События позволяют слабую связанность
- Легко добавлять новые подписчики
- Асинхронная обработка

### 2. Инкрементальные вычисления
- Индикаторы обновляются инкрементально
- Полный пересчет только при необходимости
- Оптимизация производительности

### 3. Эмулятор исполнения
- `OrderExecutionEmulator` для тестирования
- Имитация исполнения приказов
- Полезно для разработки стратегий

### 4. Rate Limiting
- Ограничение частоты запросов к бирже
- Использование `RateGate`
- Защита от блокировки API

## Анализ нашего кода

### Что можно переиспользовать

#### 1. Система индикаторов
**Файлы:**
- `src/indicators/base.rs` - базовый трейт для индикаторов
- `src/indicators/registry.rs` - реестр индикаторов
- `src/indicators/implementations.rs` - реализации индикаторов

**Что переиспользовать:**
- ✅ Трейт `Indicator` - можно использовать как есть
- ✅ Реализации индикаторов (SMA, EMA, RSI и т.д.)
- ✅ Система параметров индикаторов
- ✅ Типы данных (OHLCData, IndicatorResultData)

**Что нужно адаптировать:**
- ⚠️ Добавить инкрементальное обновление (ProcessNew, ProcessLast)
- ⚠️ Добавить поддержку интервала обновления
- ⚠️ Добавить кеширование результатов

#### 2. Система условий
**Файлы:**
- `src/condition/base.rs` - базовый трейт для условий
- `src/condition/conditions.rs` - реализации условий
- `src/condition/types.rs` - типы данных

**Что переиспользовать:**
- ✅ Трейт `Condition` - можно использовать
- ✅ Реализации условий (Above, Below, CrossesAbove и т.д.)
- ✅ Типы данных (ConditionResult, ConditionInputData)

**Что нужно адаптировать:**
- ⚠️ Добавить поддержку проверки на каждой свече
- ⚠️ Оптимизировать для реального времени

#### 3. Модель данных
**Файлы:**
- `src/data_model/quote_frame.rs` - структура данных
- `src/data_model/quote.rs` - свеча
- `src/data_model/types.rs` - типы

**Что переиспользовать:**
- ✅ Структура `QuoteFrame` (аналог CandleSeries)
- ✅ Структура `Quote` (аналог Candle)
- ✅ Типы данных (TimeFrame, Symbol)

**Что нужно адаптировать:**
- ⚠️ Добавить разделение на завершенные и текущие свечи
- ⚠️ Добавить события для обновления

#### 4. Система стратегий
**Файлы:**
- `src/strategy/types.rs` - типы стратегий
- `src/strategy/base.rs` - базовый трейт
- `src/discovery/strategy_converter.rs` - конвертер стратегий

**Что переиспользовать:**
- ✅ Структура `StrategyDefinition`
- ✅ Типы правил (EntryRule, ExitRule)
- ✅ Система биндингов индикаторов и условий

**Что нужно адаптировать:**
- ⚠️ Добавить поддержку событий (CandleFinishedEvent)
- ⚠️ Добавить методы для создания приказов
- ⚠️ Адаптировать для реального времени

### Что нужно писать заново

#### 1. Торговый движок (Trading Engine)
**Причины:**
- Бэктест движок работает с историческими данными
- Торговый движок должен работать в реальном времени
- Разные требования к производительности
- Разные модели данных (поток vs массив)

**Что нужно:**
- Новый модуль `src/trading/`
- Компоненты:
  - `TradingEngine` - основной движок
  - `MarketDataProvider` - поставщик данных
  - `OrderManager` - управление приказами
  - `PositionManager` - управление позициями
  - `RiskManager` - управление рисками

#### 2. Подключение к биржам
**Причины:**
- Нет реализации подключений к биржам
- Нужны WebSocket и REST клиенты
- Разные форматы API

**Что нужно:**
- Новый модуль `src/exchange/`
- Компоненты:
  - `ExchangeClient` - базовый клиент
  - Конкретные реализации (Binance, Bybit и т.д.)
  - WebSocket менеджер
  - REST клиент
  - Обработка ошибок и переподключений

#### 3. Система событий
**Причины:**
- Бэктест не использует события
- Нужна асинхронная обработка
- Нужна подписка/отписка

**Что нужно:**
- Использовать `tokio::sync::broadcast` или `async-channel`
- События:
  - `CandleFinished`
  - `CandleUpdate`
  - `OrderUpdate`
  - `TradeUpdate`
  - `PositionUpdate`

#### 4. Управление приказами
**Причины:**
- Бэктест не отправляет реальные приказы
- Нужна обработка статусов приказов
- Нужна система retry и error handling

**Что нужно:**
- `OrderManager` для управления приказами
- Очередь приказов
- Отслеживание статусов
- Обработка ошибок
- Rate limiting

#### 5. Управление позициями
**Причины:**
- Бэктест использует упрощенную модель позиций
- Нужно отслеживание в реальном времени
- Нужна синхронизация с биржей

**Что нужно:**
- `PositionManager` для управления позициями
- Синхронизация с биржей
- Расчет PnL в реальном времени
- Управление стоп-лоссами и тейк-профитами

## Рекомендации по реализации

### 1. Архитектура торгового движка

```rust
// src/trading/mod.rs
pub mod engine;
pub mod market_data;
pub mod order_manager;
pub mod position_manager;
pub mod risk_manager;
pub mod events;

// src/trading/engine.rs
pub struct TradingEngine {
    market_data_provider: Box<dyn MarketDataProvider>,
    order_manager: OrderManager,
    position_manager: PositionManager,
    risk_manager: RiskManager,
    strategies: Vec<Box<dyn Strategy>>,
    event_bus: EventBus,
}

impl TradingEngine {
    pub async fn run(&mut self) -> Result<()> {
        // Подписка на данные
        let mut candle_stream = self.market_data_provider.subscribe_candles().await?;
        
        while let Some(candles) = candle_stream.next().await {
            // Обновление индикаторов
            self.update_indicators(&candles).await?;
            
            // Проверка условий стратегий
            for strategy in &mut self.strategies {
                if let Some(signal) = strategy.check_conditions(&candles).await? {
                    // Создание приказа
                    let order = self.create_order(signal)?;
                    
                    // Проверка рисков
                    if self.risk_manager.check_order(&order).await? {
                        // Отправка приказа
                        self.order_manager.send_order(order).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### 2. Инкрементальное обновление индикаторов

```rust
// src/indicators/runtime.rs
pub struct IndicatorRuntime {
    indicators: HashMap<String, Box<dyn Indicator>>,
    cached_values: HashMap<String, Vec<f32>>,
    last_candle_count: usize,
}

impl IndicatorRuntime {
    pub fn update(&mut self, candles: &[Quote]) -> Result<()> {
        let current_count = candles.len();
        
        if current_count > self.last_candle_count + 1 {
            // Полный пересчет
            self.process_all(candles)?;
        } else if current_count == self.last_candle_count + 1 {
            // Новая свеча
            self.process_new(candles)?;
        } else if current_count == self.last_candle_count {
            // Обновление последней
            self.process_last(candles)?;
        }
        
        self.last_candle_count = current_count;
        Ok(())
    }
}
```

### 3. Система событий

OsEngine использует **встроенные C# события** (делегаты), а не брокеры сообщений. Это простая синхронная модель событий.

**Реализация в OsEngine:**

```csharp
// Market/Connectors/ConnectorCandles.cs

// 1. Объявление событий
public event Action<List<Candle>> NewCandlesChangeEvent;
public event Action<List<Candle>> LastCandlesChangeEvent;
public event Action<Order> OrderChangeEvent;
public event Action<MyTrade> MyTradeEvent;
public event Action<MarketDepth> GlassChangeEvent;

// 2. Вызов события (публикация)
private void MySeries_CandleFinishedEvent(CandleSeries candleSeries)
{
    if (EventsIsOn == false) return;
    
    List<Candle> candles = Candles(true);
    if (candles == null || candles.Count == 0) return;
    
    // Вызов события - синхронный вызов всех подписчиков
    if (NewCandlesChangeEvent != null)
    {
        NewCandlesChangeEvent(candles);
    }
}

// 3. Подписка на события (в BotTabSimple)
public BotTabSimple(string name, StartProgram startProgram)
{
    _connector = new ConnectorCandles(TabName, startProgram, true);
    
    // Подписка на события через +=
    _connector.NewCandlesChangeEvent += LogicToEndCandle;
    _connector.LastCandlesChangeEvent += LogicToUpdateLastCandle;
    _connector.OrderChangeEvent += _connector_OrderChangeEvent;
    _connector.MyTradeEvent += _connector_MyTradeEvent;
}

// 4. Обработчик события
private void LogicToEndCandle(List<Candle> candles)
{
    // Обновление графика
    if (_chartMaster != null)
    {
        _chartMaster.SetCandles(candles);
    }
    
    // Вызов события для стратегии
    CandleFinishedEvent?.Invoke(candles);
}

// 5. Стратегия подписывается на событие
public override void GetNameStrategy()
{
    Name = "Lesson5Bot1";
    _tabToTrade = TabsSimple[0];
    _tabToTrade.CandleFinishedEvent += _tabToTrade_CandleFinishedEvent;
}

private void _tabToTrade_CandleFinishedEvent(List<Candle> candles)
{
    // Логика стратегии
    if (candles.Count < 10) return;
    
    List<Position> positions = _tabToTrade.PositionsOpenAll;
    if (positions.Count == 0)
    {
        LogicOpenPosition(candles, positions);
    }
}
```

**Особенности реализации в C#:**
- События выполняются **синхронно** в том же потоке
- Если подписчик выбрасывает исключение, оно прерывает выполнение других подписчиков
- Нет очереди сообщений - события теряются, если нет подписчиков
- Простая модель - нет необходимости в брокерах сообщений

**Реализация в Rust (варианты):**

**Вариант 1: Tokio Broadcast (аналог C# событий, но асинхронный)**
```rust
// src/trading/events.rs
use tokio::sync::broadcast;

pub struct EventBus {
    candle_finished: broadcast::Sender<CandleFinishedEvent>,
    candle_update: broadcast::Sender<CandleUpdateEvent>,
    order_update: broadcast::Sender<OrderUpdateEvent>,
}

#[derive(Clone)]
pub struct CandleFinishedEvent {
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub candles: Vec<Quote>,
}

impl EventBus {
    pub fn new() -> Self {
        let (candle_finished, _) = broadcast::channel(1000);
        let (candle_update, _) = broadcast::channel(1000);
        let (order_update, _) = broadcast::channel(1000);
        
        Self {
            candle_finished,
            candle_update,
            order_update,
        }
    }
    
    pub fn publish_candle_finished(&self, event: CandleFinishedEvent) {
        let _ = self.candle_finished.send(event);
    }
    
    pub fn subscribe_candles(&self) -> broadcast::Receiver<CandleFinishedEvent> {
        self.candle_finished.subscribe()
    }
}

// Использование
pub struct TradingEngine {
    event_bus: EventBus,
}

impl TradingEngine {
    pub async fn run(&mut self) -> Result<()> {
        let mut candle_receiver = self.event_bus.subscribe_candles();
        
        // Публикация события
        self.event_bus.publish_candle_finished(CandleFinishedEvent {
            symbol: Symbol::new("BTCUSDT"),
            timeframe: TimeFrame::M1,
            candles: vec![],
        });
        
        // Подписка на события
        tokio::spawn(async move {
            while let Ok(event) = candle_receiver.recv().await {
                // Обработка события
            }
        });
        
        Ok(())
    }
}
```

**Вариант 2: Callback функции (проще, синхронный)**
```rust
// src/trading/connector.rs
pub type CandleCallback = Box<dyn Fn(&[Quote]) + Send + Sync>;
pub type OrderCallback = Box<dyn Fn(&Order) + Send + Sync>;

pub struct ConnectorCandles {
    candle_finished_callbacks: Vec<CandleCallback>,
    order_change_callbacks: Vec<OrderCallback>,
}

impl ConnectorCandles {
    pub fn new() -> Self {
        Self {
            candle_finished_callbacks: Vec::new(),
            order_change_callbacks: Vec::new(),
        }
    }
    
    pub fn on_candle_finished<F>(&mut self, callback: F)
    where
        F: Fn(&[Quote]) + Send + Sync + 'static,
    {
        self.candle_finished_callbacks.push(Box::new(callback));
    }
    
    pub fn notify_candle_finished(&self, candles: &[Quote]) {
        for callback in &self.candle_finished_callbacks {
            callback(candles);
        }
    }
}

// Использование
let mut connector = ConnectorCandles::new();
connector.on_candle_finished(|candles| {
    println!("New candles: {}", candles.len());
});
```

**Вариант 3: EventEmitter паттерн (как в Node.js)**
```rust
// src/trading/event_emitter.rs
use std::collections::HashMap;
use std::sync::Arc;

pub type EventHandler<T> = Box<dyn Fn(T) + Send + Sync>;

pub struct EventEmitter {
    handlers: HashMap<String, Vec<Arc<dyn Fn(serde_json::Value) + Send + Sync>>>,
}

impl EventEmitter {
    pub fn on<F>(&mut self, event: &str, handler: F)
    where
        F: Fn(serde_json::Value) + Send + Sync + 'static,
    {
        self.handlers
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::new(handler));
    }
    
    pub fn emit(&self, event: &str, data: serde_json::Value) {
        if let Some(handlers) = self.handlers.get(event) {
            for handler in handlers {
                handler(data.clone());
            }
        }
    }
}
```

**Рекомендация для Rust проекта:**

Для торгового движка лучше использовать **Tokio Broadcast**, так как:
- ✅ Асинхронная обработка (не блокирует основной поток)
- ✅ Множественные подписчики (broadcast)
- ✅ Буферизация событий (не теряются при отсутствии подписчиков)
- ✅ Отписка через drop receiver
- ✅ Не требует внешних брокеров сообщений

**Как работает Tokio Broadcast:**

Tokio Broadcast - это **встроенная библиотека** (часть экосистемы Tokio), которая предоставляет каналы для обмена сообщениями между асинхронными задачами. Это не внешний брокер сообщений, а структура данных в памяти.

**Архитектура:**

```
┌─────────────────┐
│  Sender (tx)    │  ← Один отправитель
└────────┬────────┘
         │
         │ broadcast::channel(1000)
         │ (буфер на 1000 сообщений)
         │
         ▼
┌─────────────────┐
│  Внутренний     │
│  буфер (Vec)    │  ← Сообщения хранятся в памяти
└────────┬────────┘
         │
         ├──────────┬──────────┬──────────┐
         ▼          ▼          ▼          ▼
    ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
    │Receiver│ │Receiver│ │Receiver│ │Receiver│  ← Множественные получатели
    │  (rx1) │ │  (rx2) │ │  (rx3) │ │  (rx4) │
    └────────┘ └────────┘ └────────┘ └────────┘
```

**Как это работает:**

1. **Создание канала:**
   ```rust
   use tokio::sync::broadcast;
   
   // Создается один Sender и можно создать множество Receivers
   let (tx, _rx) = broadcast::channel(1000);  // Буфер на 1000 сообщений
   ```

2. **Отправка сообщений (множество задач могут отправлять):**
   ```rust
   // Задача 1: Получение свечей от биржи
   tokio::spawn(async move {
       while let Some(candle) = exchange.receive_candle().await {
           let event = CandleFinishedEvent { candles: vec![candle] };
           tx.send(event).unwrap();  // Отправка в канал
       }
   });
   ```

3. **Подписка на сообщения (каждая задача создает свой Receiver):**
   ```rust
   // Задача 2: Стратегия 1
   let mut rx1 = tx.subscribe();  // Создает новый Receiver
   tokio::spawn(async move {
       while let Ok(event) = rx1.recv().await {
           // Обработка события в стратегии 1
           strategy1.process(event).await;
       }
   });
   
   // Задача 3: Стратегия 2
   let mut rx2 = tx.subscribe();  // Еще один Receiver
   tokio::spawn(async move {
       while let Ok(event) = rx2.recv().await {
           // Обработка события в стратегии 2
           strategy2.process(event).await;
       }
   });
   ```

4. **Внутренняя реализация:**
   - Сообщения хранятся в **круговом буфере** в памяти
   - Каждый Receiver имеет свой **индекс чтения**
   - Когда Receiver читает сообщение, оно **не удаляется** (другие Receivers могут его прочитать)
   - Сообщения удаляются только когда **все Receivers** их прочитали или буфер переполнен

**Преимущества:**

- ✅ **В памяти** - очень быстро (нет сетевых задержек)
- ✅ **Неблокирующий** - async/await, не блокирует потоки
- ✅ **Множественные подписчики** - один Sender, много Receivers
- ✅ **Буферизация** - сообщения не теряются (до заполнения буфера)
- ✅ **Автоматическая очистка** - старые сообщения удаляются автоматически
- ✅ **Легковесный** - не требует внешних зависимостей

**Сравнение с другими подходами:**

| Подход | Тип | Производительность | Сложность |
|--------|-----|-------------------|-----------|
| **Tokio Broadcast** | Встроенная библиотека | Очень высокая | Низкая |
| Redis Pub/Sub | Внешний брокер | Средняя (сеть) | Средняя |
| RabbitMQ | Внешний брокер | Средняя (сеть) | Высокая |
| Kafka | Внешний брокер | Высокая (сеть) | Очень высокая |
| C# Events | Встроенные делегаты | Высокая (синхронная) | Низкая |

**Пример полной реализации:**

```rust
use tokio::sync::broadcast;
use std::sync::Arc;

// События
#[derive(Clone, Debug)]
pub struct CandleFinishedEvent {
    pub symbol: String,
    pub candles: Vec<Candle>,
}

// EventBus - центральная точка для всех событий
pub struct EventBus {
    candle_finished: broadcast::Sender<CandleFinishedEvent>,
    order_update: broadcast::Sender<OrderUpdateEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (candle_finished, _) = broadcast::channel(1000);
        let (order_update, _) = broadcast::channel(1000);
        
        Self {
            candle_finished,
            order_update,
        }
    }
    
    // Публикация события
    pub fn publish_candle_finished(&self, event: CandleFinishedEvent) {
        let _ = self.candle_finished.send(event);
    }
    
    // Подписка на события (создает новый Receiver)
    pub fn subscribe_candles(&self) -> broadcast::Receiver<CandleFinishedEvent> {
        self.candle_finished.subscribe()
    }
}

// Использование
#[tokio::main]
async fn main() {
    let event_bus = Arc::new(EventBus::new());
    
    // Задача 1: Получение данных от биржи
    let event_bus1 = event_bus.clone();
    tokio::spawn(async move {
        loop {
            let candles = exchange.get_candles().await;
            let event = CandleFinishedEvent {
                symbol: "BTCUSDT".to_string(),
                candles,
            };
            event_bus1.publish_candle_finished(event);
        }
    });
    
    // Задача 2: Стратегия 1
    let mut rx1 = event_bus.subscribe_candles();
    tokio::spawn(async move {
        while let Ok(event) = rx1.recv().await {
            strategy1.on_candle_finished(event).await;
        }
    });
    
    // Задача 3: Стратегия 2
    let mut rx2 = event_bus.subscribe_candles();
    tokio::spawn(async move {
        while let Ok(event) = rx2.recv().await {
            strategy2.on_candle_finished(event).await;
        }
    });
    
    // Задача 4: Логирование
    let mut rx3 = event_bus.subscribe_candles();
    tokio::spawn(async move {
        while let Ok(event) = rx3.recv().await {
            println!("New candles: {}", event.candles.len());
        }
    });
}
```

**Важные детали:**

1. **Не pipe в Unix смысле** - это структура данных в памяти процесса, а не межпроцессное взаимодействие
2. **Один процесс** - все задачи работают в одном процессе Rust
3. **Быстро** - нет сериализации/десериализации, копирование только при клонировании
4. **Безопасно** - Rust гарантирует безопасность памяти и потоков
5. **Легковесно** - нет внешних зависимостей, все в стандартной библиотеке Tokio

**Пример полной реализации:**

```rust
// src/trading/mod.rs
pub mod events;
pub mod engine;
pub mod connector;

// src/trading/events.rs
use tokio::sync::broadcast;
use crate::data_model::types::{Symbol, TimeFrame};
use crate::data_model::quote::Quote;
use crate::entity::Order;

#[derive(Clone, Debug)]
pub struct CandleFinishedEvent {
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub candles: Vec<Quote>,
}

#[derive(Clone, Debug)]
pub struct OrderUpdateEvent {
    pub order: Order,
}

pub struct EventBus {
    candle_finished: broadcast::Sender<CandleFinishedEvent>,
    order_update: broadcast::Sender<OrderUpdateEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (candle_finished, _) = broadcast::channel(1000);
        let (order_update, _) = broadcast::channel(1000);
        
        Self {
            candle_finished,
            order_update,
        }
    }
    
    pub fn publish_candle_finished(&self, event: CandleFinishedEvent) -> Result<(), broadcast::error::SendError<CandleFinishedEvent>> {
        self.candle_finished.send(event)
    }
    
    pub fn subscribe_candles(&self) -> broadcast::Receiver<CandleFinishedEvent> {
        self.candle_finished.subscribe()
    }
}

// src/trading/connector.rs
use crate::trading::events::{EventBus, CandleFinishedEvent};
use tokio::sync::broadcast;

pub struct ConnectorCandles {
    event_bus: Arc<EventBus>,
}

impl ConnectorCandles {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }
    
    pub async fn on_candle_finished(&self, candles: Vec<Quote>) {
        let event = CandleFinishedEvent {
            symbol: Symbol::new("BTCUSDT"),
            timeframe: TimeFrame::M1,
            candles,
        };
        
        let _ = self.event_bus.publish_candle_finished(event);
    }
}

// src/trading/engine.rs
use crate::trading::events::EventBus;
use tokio::sync::broadcast;

pub struct TradingEngine {
    event_bus: Arc<EventBus>,
    strategies: Vec<Box<dyn Strategy>>,
}

impl TradingEngine {
    pub async fn run(&mut self) -> Result<()> {
        let mut candle_receiver = self.event_bus.subscribe_candles();
        
        // Обработка событий в отдельной задаче
        tokio::spawn(async move {
            while let Ok(event) = candle_receiver.recv().await {
                // Обновление индикаторов
                // Проверка условий стратегий
                // Создание приказов
            }
        });
        
        Ok(())
    }
}
```

**Сравнение с OsEngine:**

| Аспект | OsEngine (C#) | Rust (Tokio Broadcast) |
|--------|---------------|------------------------|
| Тип | Синхронные события | Асинхронные каналы |
| Подписчики | Множественные | Множественные (broadcast) |
| Буферизация | Нет (теряются) | Да (очередь) |
| Поток | Тот же поток | Асинхронные задачи |
| Брокер | Не нужен | Не нужен |
| Производительность | Высокая | Очень высокая |

### 4. Подключение к биржам

```rust
// src/exchange/mod.rs
pub mod binance;
pub mod bybit;
pub mod client;

// src/exchange/client.rs
pub trait ExchangeClient: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn subscribe_candles(&mut self, symbol: Symbol, timeframe: TimeFrame) -> Result<()>;
    async fn send_order(&mut self, order: Order) -> Result<OrderId>;
    async fn cancel_order(&mut self, order_id: OrderId) -> Result<()>;
    fn subscribe_events(&self) -> Receiver<ExchangeEvent>;
}
```

## Выводы

### Что можно переиспользовать:
1. ✅ Система индикаторов (с адаптацией)
2. ✅ Система условий (с адаптацией)
3. ✅ Модель данных (с адаптацией)
4. ✅ Структура стратегий (с адаптацией)

### Что нужно писать заново:
1. ❌ Торговый движок (полностью новый)
2. ❌ Подключение к биржам (полностью новый)
3. ❌ Система событий (новый подход)
4. ❌ Управление приказами (новый)
5. ❌ Управление позициями (новый)

### Приоритеты разработки:
1. **Высокий приоритет:**
   - Торговый движок
   - Подключение к биржам
   - Система событий

2. **Средний приоритет:**
   - Управление приказами
   - Управление позициями
   - Инкрементальное обновление индикаторов

3. **Низкий приоритет:**
   - Адаптация существующих компонентов
   - Оптимизация производительности

## Заключение

Торговый движок OsEngine использует событийную архитектуру с инкрементальным обновлением индикаторов. Основной поток данных: Биржа → Server → CandleManager → CandleSeries → ConnectorCandles → BotTabSimple → Strategy → Order → Server → Биржа.

Наш бэктест движок не подходит для реальной торговли, так как работает с историческими данными и не имеет подключений к биржам. Необходимо создать новый торговый движок, который будет использовать существующие компоненты (индикаторы, условия, модель данных) с адаптацией для реального времени.

