use chrono::{DateTime, Utc};
use std::collections::VecDeque;

use crate::data_model::bar_types::BarType;
use crate::data_model::quote::Quote;
use crate::data_model::quote_frame::{QuoteFrame, QuoteFrameError};
use crate::data_model::types::{Price, Symbol, TimeFrame, Volume};

pub trait BarBuilder {
    fn build_from_quotes(
        &mut self,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, BarBuilderError>;
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
        let mut result = QuoteFrame::new(self.symbol.clone(), self.target_timeframe.clone());
        let range_size = self.range_size as Price;

        let mut current_bar: Option<Quote> = None;

        for quote in source_frame.iter() {
            if current_bar.is_none() {
                current_bar = Some(quote.clone());
                continue;
            }

            let bar = current_bar.as_mut().unwrap();
            let high = bar.high().max(quote.high());
            let low = bar.low().min(quote.low());
            let range = high - low;

            if range >= range_size {
                let new_bar = Quote::from_parts(
                    self.symbol.clone(),
                    self.target_timeframe.clone(),
                    quote.timestamp(),
                    bar.open(),
                    high,
                    low,
                    if quote.close() > bar.open() {
                        bar.open() + range_size
                    } else {
                        bar.open() - range_size
                    },
                    bar.volume() + quote.volume(),
                );
                result.push(new_bar)?;
                current_bar = Some(quote.clone());
            } else {
                *bar = Quote::from_parts(
                    self.symbol.clone(),
                    self.target_timeframe.clone(),
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
        let mut result = QuoteFrame::new(self.symbol.clone(), self.target_timeframe.clone());
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

            let high = bar.high().max(quote.high());
            let low = bar.low().min(quote.low());

            if accumulated_volume >= volume_size {
                let new_bar = Quote::from_parts(
                    self.symbol.clone(),
                    self.target_timeframe.clone(),
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
                *bar = Quote::from_parts(
                    self.symbol.clone(),
                    self.target_timeframe.clone(),
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
        let mut result = QuoteFrame::new(self.symbol.clone(), self.target_timeframe.clone());
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
            let high = bar.high().max(quote.high());
            let low = bar.low().min(quote.low());

            if true_range >= threshold {
                let new_bar = Quote::from_parts(
                    self.symbol.clone(),
                    self.target_timeframe.clone(),
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
                *bar = Quote::from_parts(
                    self.symbol.clone(),
                    self.target_timeframe.clone(),
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
        let mut result = QuoteFrame::new(self.symbol.clone(), self.target_timeframe.clone());
        let brick_size = self.brick_size as Price;

        let mut last_brick_close: Option<Price> = None;

        for quote in source_frame.iter() {
            if last_brick_close.is_none() {
                let rounded_close = (quote.close() / brick_size).round() * brick_size;
                let new_bar = Quote::from_parts(
                    self.symbol.clone(),
                    self.target_timeframe.clone(),
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
                        self.symbol.clone(),
                        self.target_timeframe.clone(),
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
        let mut result = QuoteFrame::new(self.symbol.clone(), self.target_timeframe.clone());
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
                self.symbol.clone(),
                self.target_timeframe.clone(),
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
