use chrono::{DateTime, Utc};
use std::collections::VecDeque;

use crate::candles::bar_types::BarType;
use crate::data_model::quote::Quote;
use crate::data_model::quote_frame::{QuoteFrame, QuoteFrameError};
use crate::data_model::types::{Price, Symbol, TimeFrame, Volume};

pub trait BarBuilder {
    fn build_from_quotes(
        &mut self,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, BarBuilderError>;
}

struct BarBuilderHelpers;

impl BarBuilderHelpers {
    fn create_quote(
        symbol: &Symbol,
        timeframe: &TimeFrame,
        timestamp: chrono::DateTime<chrono::Utc>,
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: Volume,
    ) -> Quote {
        Quote::from_parts(
            symbol.clone(),
            timeframe.clone(),
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        )
    }

    fn update_high_low(
        bar_high: Price,
        bar_low: Price,
        quote_high: Price,
        quote_low: Price,
    ) -> (Price, Price) {
        (bar_high.max(quote_high), bar_low.min(quote_low))
    }
}

pub struct RangeBarBuilder {
    range_size: u32,
    symbol: Symbol,
    target_timeframe: TimeFrame,
}

impl RangeBarBuilder {
    pub fn new(symbol: Symbol, target_timeframe: TimeFrame, range_size: u32) -> Self {
        Self {
            range_size,
            symbol,
            target_timeframe,
        }
    }
}

impl BarBuilder for RangeBarBuilder {
    fn build_from_quotes(
        &mut self,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, BarBuilderError> {
        if source_frame.is_empty() {
            return Ok(QuoteFrame::new(
                self.symbol.clone(),
                self.target_timeframe.clone(),
            ));
        }

        if self.range_size == 0 {
            return Err(BarBuilderError::UnsupportedBarType(
                "Range size must be greater than 0".to_string(),
            ));
        }

        let symbol = &self.symbol;
        let timeframe = &self.target_timeframe;
        let mut result = QuoteFrame::new(symbol.clone(), timeframe.clone());
        let range_size = self.range_size as Price;

        let mut current_bar: Option<Quote> = None;

        for quote in source_frame.iter() {
            if current_bar.is_none() {
                current_bar = Some(quote.clone());
                continue;
            }

            let bar = current_bar.as_mut().unwrap();
            let (high, low) = BarBuilderHelpers::update_high_low(
                bar.high(),
                bar.low(),
                quote.high(),
                quote.low(),
            );
            let range = high - low;

            if range >= range_size {
                let close_price = if quote.close() > bar.open() {
                    bar.open() + range_size
                } else {
                    bar.open() - range_size
                };
                let new_bar = BarBuilderHelpers::create_quote(
                    symbol,
                    timeframe,
                    quote.timestamp(),
                    bar.open(),
                    high,
                    low,
                    close_price,
                    bar.volume() + quote.volume(),
                );
                result.push(new_bar)?;
                current_bar = Some(quote.clone());
            } else {
                *bar = BarBuilderHelpers::create_quote(
                    symbol,
                    timeframe,
                    bar.timestamp(),
                    bar.open(),
                    high,
                    low,
                    quote.close(),
                    bar.volume() + quote.volume(),
                );
            }
        }

        if let Some(bar) = current_bar {
            result.push(bar)?;
        }

        Ok(result)
    }
}

pub struct VolumeBarBuilder {
    volume_size: u64,
    symbol: Symbol,
    target_timeframe: TimeFrame,
}

impl VolumeBarBuilder {
    pub fn new(symbol: Symbol, target_timeframe: TimeFrame, volume_size: u64) -> Self {
        Self {
            volume_size,
            symbol,
            target_timeframe,
        }
    }
}

impl BarBuilder for VolumeBarBuilder {
    fn build_from_quotes(
        &mut self,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, BarBuilderError> {
        if source_frame.is_empty() {
            return Ok(QuoteFrame::new(
                self.symbol.clone(),
                self.target_timeframe.clone(),
            ));
        }

        if self.volume_size == 0 {
            return Err(BarBuilderError::UnsupportedBarType(
                "Volume size must be greater than 0".to_string(),
            ));
        }

        let symbol = &self.symbol;
        let timeframe = &self.target_timeframe;
        let mut result = QuoteFrame::new(symbol.clone(), timeframe.clone());
        let volume_size = self.volume_size as Volume;

        let mut current_bar: Option<Quote> = None;
        let mut accumulated_volume = 0.0;

        for quote in source_frame.iter() {
            if current_bar.is_none() {
                current_bar = Some(quote.clone());
                accumulated_volume = quote.volume();
                continue;
            }

            let bar = current_bar.as_mut().unwrap();
            accumulated_volume += quote.volume();

            let (high, low) = BarBuilderHelpers::update_high_low(
                bar.high(),
                bar.low(),
                quote.high(),
                quote.low(),
            );

            if accumulated_volume >= volume_size {
                let new_bar = BarBuilderHelpers::create_quote(
                    symbol,
                    timeframe,
                    quote.timestamp(),
                    bar.open(),
                    high,
                    low,
                    quote.close(),
                    accumulated_volume,
                );
                result.push(new_bar)?;
                current_bar = Some(quote.clone());
                accumulated_volume = quote.volume();
            } else {
                *bar = BarBuilderHelpers::create_quote(
                    symbol,
                    timeframe,
                    bar.timestamp(),
                    bar.open(),
                    high,
                    low,
                    quote.close(),
                    accumulated_volume,
                );
            }
        }

        if let Some(bar) = current_bar {
            result.push(bar)?;
        }

        Ok(result)
    }
}

pub struct VolatilityBarBuilder {
    volatility_threshold: u32,
    symbol: Symbol,
    target_timeframe: TimeFrame,
}

impl VolatilityBarBuilder {
    pub fn new(symbol: Symbol, target_timeframe: TimeFrame, volatility_threshold: u32) -> Self {
        Self {
            volatility_threshold,
            symbol,
            target_timeframe,
        }
    }
}

impl BarBuilder for VolatilityBarBuilder {
    fn build_from_quotes(
        &mut self,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, BarBuilderError> {
        if source_frame.is_empty() {
            return Ok(QuoteFrame::new(
                self.symbol.clone(),
                self.target_timeframe.clone(),
            ));
        }

        if self.volatility_threshold == 0 {
            return Err(BarBuilderError::UnsupportedBarType(
                "Volatility threshold must be greater than 0".to_string(),
            ));
        }

        let symbol = &self.symbol;
        let timeframe = &self.target_timeframe;
        let mut result = QuoteFrame::new(symbol.clone(), timeframe.clone());
        let threshold = self.volatility_threshold as Price;

        let mut current_bar: Option<Quote> = None;
        let mut previous_close: Option<Price> = None;

        for quote in source_frame.iter() {
            if current_bar.is_none() {
                current_bar = Some(quote.clone());
                previous_close = Some(quote.close());
                continue;
            }

            let bar = current_bar.as_mut().unwrap();
            let true_range = quote.true_range(previous_close);
            let (high, low) = BarBuilderHelpers::update_high_low(
                bar.high(),
                bar.low(),
                quote.high(),
                quote.low(),
            );

            if true_range >= threshold {
                let new_bar = BarBuilderHelpers::create_quote(
                    symbol,
                    timeframe,
                    quote.timestamp(),
                    bar.open(),
                    high,
                    low,
                    quote.close(),
                    bar.volume() + quote.volume(),
                );
                result.push(new_bar)?;
                current_bar = Some(quote.clone());
                previous_close = Some(quote.close());
            } else {
                *bar = BarBuilderHelpers::create_quote(
                    symbol,
                    timeframe,
                    bar.timestamp(),
                    bar.open(),
                    high,
                    low,
                    quote.close(),
                    bar.volume() + quote.volume(),
                );
                previous_close = Some(quote.close());
            }
        }

        if let Some(bar) = current_bar {
            result.push(bar)?;
        }

        Ok(result)
    }
}

pub struct RenkoBarBuilder {
    brick_size: u32,
    symbol: Symbol,
    target_timeframe: TimeFrame,
}

impl RenkoBarBuilder {
    pub fn new(symbol: Symbol, target_timeframe: TimeFrame, brick_size: u32) -> Self {
        Self {
            brick_size,
            symbol,
            target_timeframe,
        }
    }
}

impl BarBuilder for RenkoBarBuilder {
    fn build_from_quotes(
        &mut self,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, BarBuilderError> {
        if source_frame.is_empty() {
            return Ok(QuoteFrame::new(
                self.symbol.clone(),
                self.target_timeframe.clone(),
            ));
        }

        if self.brick_size == 0 {
            return Err(BarBuilderError::UnsupportedBarType(
                "Brick size must be greater than 0".to_string(),
            ));
        }

        let symbol = &self.symbol;
        let timeframe = &self.target_timeframe;
        let mut result = QuoteFrame::new(symbol.clone(), timeframe.clone());
        let brick_size = self.brick_size as Price;

        let mut last_brick_close: Option<Price> = None;

        for quote in source_frame.iter() {
            if last_brick_close.is_none() {
                let rounded_close = (quote.close() / brick_size).round() * brick_size;
                let new_bar = Quote::from_parts(
                    symbol.clone(),
                    timeframe.clone(),
                    quote.timestamp(),
                    rounded_close,
                    rounded_close,
                    rounded_close,
                    rounded_close,
                    quote.volume(),
                );
                result.push(new_bar.clone())?;
                last_brick_close = Some(rounded_close);
                continue;
            }

            let last_close = last_brick_close.unwrap();
            let current_close = quote.close();
            let diff = current_close - last_close;
            let bricks = (diff / brick_size).abs().floor() as i32;

            if bricks > 0 {
                let direction = if diff > 0.0 { 1.0 } else { -1.0 };
                for i in 1..=bricks {
                    let brick_close = last_close + (direction * brick_size * i as Price);
                    let new_bar = Quote::from_parts(
                        symbol.clone(),
                        timeframe.clone(),
                        quote.timestamp(),
                        brick_close - (direction * brick_size),
                        brick_close.max(brick_close - (direction * brick_size)),
                        brick_close.min(brick_close - (direction * brick_size)),
                        brick_close,
                        quote.volume() / bricks as Volume,
                    );
                    result.push(new_bar)?;
                    last_brick_close = Some(brick_close);
                }
            }
        }

        Ok(result)
    }
}

pub struct HeikinAshiBarBuilder {
    symbol: Symbol,
    target_timeframe: TimeFrame,
}

impl HeikinAshiBarBuilder {
    pub fn new(symbol: Symbol, target_timeframe: TimeFrame) -> Self {
        Self {
            symbol,
            target_timeframe,
        }
    }
}

impl BarBuilder for HeikinAshiBarBuilder {
    fn build_from_quotes(
        &mut self,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, BarBuilderError> {
        if source_frame.is_empty() {
            return Ok(QuoteFrame::new(
                self.symbol.clone(),
                self.target_timeframe.clone(),
            ));
        }

        let symbol = &self.symbol;
        let timeframe = &self.target_timeframe;
        let mut result = QuoteFrame::new(symbol.clone(), timeframe.clone());
        let mut previous_ha: Option<Quote> = None;

        for quote in source_frame.iter() {
            let (ha_open, ha_close, ha_high, ha_low) = if let Some(prev_ha) = &previous_ha {
                let ha_close = (quote.open() + quote.high() + quote.low() + quote.close()) / 4.0;
                let ha_open = (prev_ha.open() + prev_ha.close()) / 2.0;
                let ha_high = quote.high().max(ha_open).max(ha_close);
                let ha_low = quote.low().min(ha_open).min(ha_close);

                (ha_open, ha_close, ha_high, ha_low)
            } else {
                let ha_close = (quote.open() + quote.high() + quote.low() + quote.close()) / 4.0;
                let ha_open = (quote.open() + quote.close()) / 2.0;
                let ha_high = quote.high();
                let ha_low = quote.low();

                (ha_open, ha_close, ha_high, ha_low)
            };

            let ha_bar = Quote::from_parts(
                symbol.clone(),
                timeframe.clone(),
                quote.timestamp(),
                ha_open,
                ha_high,
                ha_low,
                ha_close,
                quote.volume(),
            );

            result.push(ha_bar.clone())?;
            previous_ha = Some(ha_bar);
        }

        Ok(result)
    }
}

pub struct BarBuilderFactory;

impl BarBuilderFactory {
    pub fn create_builder(
        bar_type: &BarType,
        symbol: Symbol,
        target_timeframe: TimeFrame,
    ) -> Result<Box<dyn BarBuilder>, BarBuilderError> {
        match bar_type {
            BarType::Range { range_size } => Ok(Box::new(RangeBarBuilder::new(
                symbol,
                target_timeframe,
                *range_size,
            ))),
            BarType::Volume { volume_size } => Ok(Box::new(VolumeBarBuilder::new(
                symbol,
                target_timeframe,
                *volume_size,
            ))),
            BarType::Volatility {
                volatility_threshold,
            } => Ok(Box::new(VolatilityBarBuilder::new(
                symbol,
                target_timeframe,
                *volatility_threshold,
            ))),
            BarType::Renko { brick_size } => Ok(Box::new(RenkoBarBuilder::new(
                symbol,
                target_timeframe,
                *brick_size,
            ))),
            BarType::HeikinAshi => Ok(Box::new(HeikinAshiBarBuilder::new(
                symbol,
                target_timeframe,
            ))),
            BarType::Time => Err(BarBuilderError::UnsupportedBarType(
                "Time bars are built by timeframe aggregator".to_string(),
            )),
            BarType::Custom { .. } => Err(BarBuilderError::UnsupportedBarType(
                "Custom bar types require custom builder implementation".to_string(),
            )),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BarBuilderError {
    #[error("QuoteFrame error: {0}")]
    QuoteFrameError(#[from] QuoteFrameError),
    #[error("Unsupported bar type: {0}")]
    UnsupportedBarType(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::types::{Symbol, TimeFrame};
    use chrono::{DateTime, Utc};

    fn create_test_quote_frame(symbol: Symbol, timeframe: TimeFrame, count: usize) -> QuoteFrame {
        let mut frame = QuoteFrame::new(symbol, timeframe);
        let base_time = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        for i in 0..count {
            let timestamp = base_time + chrono::Duration::minutes(i as i64);
            let base_price = 100.0 + (i as f32 * 0.5);
            let quote = Quote::from_parts(
                frame.symbol().clone(),
                frame.timeframe().clone(),
                timestamp,
                base_price,
                base_price + 1.0,
                base_price - 1.0,
                base_price + 0.5,
                1000.0,
            );
            frame.push(quote).unwrap();
        }
        frame
    }

    #[test]
    fn test_range_bar_builder() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = RangeBarBuilder::new(symbol, timeframe, 5);
        let result = builder.build_from_quotes(&source_frame).unwrap();

        assert!(!result.is_empty());
        assert!(result.len() <= source_frame.len());
    }

    #[test]
    fn test_range_bar_builder_empty_frame() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = QuoteFrame::new(symbol.clone(), timeframe.clone());

        let mut builder = RangeBarBuilder::new(symbol, timeframe, 5);
        let result = builder.build_from_quotes(&source_frame).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_volume_bar_builder() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = VolumeBarBuilder::new(symbol, timeframe, 5000);
        let result = builder.build_from_quotes(&source_frame).unwrap();

        assert!(!result.is_empty());
    }

    #[test]
    fn test_volatility_bar_builder() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = VolatilityBarBuilder::new(symbol, timeframe, 2);
        let result = builder.build_from_quotes(&source_frame).unwrap();

        assert!(!result.is_empty());
    }

    #[test]
    fn test_renko_bar_builder() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = RenkoBarBuilder::new(symbol, timeframe, 5);
        let result = builder.build_from_quotes(&source_frame).unwrap();

        assert!(!result.is_empty());
    }

    #[test]
    fn test_heikin_ashi_bar_builder() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = HeikinAshiBarBuilder::new(symbol, timeframe);
        let result = builder.build_from_quotes(&source_frame).unwrap();

        assert_eq!(result.len(), source_frame.len());
    }

    #[test]
    fn test_bar_builder_factory_range() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let bar_type = BarType::Range { range_size: 100 };

        let builder = BarBuilderFactory::create_builder(&bar_type, symbol, timeframe);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_bar_builder_factory_volume() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let bar_type = BarType::Volume { volume_size: 1000 };

        let builder = BarBuilderFactory::create_builder(&bar_type, symbol, timeframe);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_bar_builder_factory_time_error() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let bar_type = BarType::Time;

        let builder_result = BarBuilderFactory::create_builder(&bar_type, symbol, timeframe);
        assert!(builder_result.is_err());
        match builder_result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(error_msg.contains("Time bars"));
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_bar_builder_factory_custom_error() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let bar_type = BarType::Custom {
            name: "Custom".to_string(),
            parameters: vec![],
        };

        let builder_result = BarBuilderFactory::create_builder(&bar_type, symbol, timeframe);
        assert!(builder_result.is_err());
        match builder_result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(error_msg.contains("Custom bar types"));
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_range_bar_builder_range_size() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let mut source_frame = QuoteFrame::new(symbol.clone(), timeframe.clone());

        let base_time = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        for i in 0..5 {
            let timestamp = base_time + chrono::Duration::minutes(i as i64);
            let quote = Quote::from_parts(
                symbol.clone(),
                timeframe.clone(),
                timestamp,
                100.0 + (i as f32 * 2.0),
                105.0 + (i as f32 * 2.0),
                95.0 + (i as f32 * 2.0),
                102.0 + (i as f32 * 2.0),
                1000.0,
            );
            source_frame.push(quote).unwrap();
        }

        let mut builder = RangeBarBuilder::new(symbol, timeframe, 10);
        let result = builder.build_from_quotes(&source_frame).unwrap();

        assert!(!result.is_empty());
        let quotes: Vec<_> = result.iter().collect();
        let last_index = quotes.len() - 1;

        for (idx, quote) in quotes.iter().enumerate() {
            let range = quote.high() - quote.low();
            let is_last = idx == last_index;
            if !is_last {
                assert!(
                    range >= 10.0,
                    "Non-last bar should have range >= 10.0, got {}",
                    range
                );
            }
        }
    }

    #[test]
    fn test_range_bar_builder_zero_range_size() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = RangeBarBuilder::new(symbol, timeframe, 0);
        let result = builder.build_from_quotes(&source_frame);
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.to_string().contains("Range size must be greater than 0")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_volume_bar_builder_zero_volume_size() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = VolumeBarBuilder::new(symbol, timeframe, 0);
        let result = builder.build_from_quotes(&source_frame);
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.to_string().contains("Volume size must be greater than 0")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_volatility_bar_builder_zero_threshold() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = VolatilityBarBuilder::new(symbol, timeframe, 0);
        let result = builder.build_from_quotes(&source_frame);
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e
                .to_string()
                .contains("Volatility threshold must be greater than 0")),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_renko_bar_builder_zero_brick_size() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol.clone(), timeframe.clone(), 10);

        let mut builder = RenkoBarBuilder::new(symbol, timeframe, 0);
        let result = builder.build_from_quotes(&source_frame);
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.to_string().contains("Brick size must be greater than 0")),
            Ok(_) => panic!("Expected error"),
        }
    }
}
