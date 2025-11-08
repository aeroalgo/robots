use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;

use thiserror::Error;

use super::quote::Quote;

#[derive(Debug, Error)]
pub enum ValueVectorError {
    #[error("length mismatch: left {left}, right {right}")]
    LengthMismatch { left: usize, right: usize },
    #[error("window must be greater than zero")]
    InvalidWindow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValueVector {
    data: Arc<[f64]>,
}

impl ValueVector {
    pub fn new(data: Vec<f64>) -> Self {
        Self { data: data.into() }
    }

    pub fn from_quotes<F>(quotes: &[Quote], extractor: F) -> Self
    where
        F: Fn(&Quote) -> f64,
    {
        let data: Vec<f64> = quotes.iter().map(extractor).collect();
        Self::new(data)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }

    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.data.iter().copied()
    }

    pub fn sum(&self) -> f64 {
        self.iter().sum()
    }

    pub fn mean(&self) -> Option<f64> {
        if self.is_empty() {
            None
        } else {
            Some(self.sum() / self.len() as f64)
        }
    }

    pub fn map<F>(&self, func: F) -> Self
    where
        F: Fn(f64) -> f64,
    {
        let data: Vec<f64> = self.iter().map(func).collect();
        Self::new(data)
    }

    pub fn zip_with<F>(&self, other: &Self, func: F) -> Result<Self, ValueVectorError>
    where
        F: Fn(f64, f64) -> f64,
    {
        if self.len() != other.len() {
            return Err(ValueVectorError::LengthMismatch {
                left: self.len(),
                right: other.len(),
            });
        }
        let data: Vec<f64> = self
            .iter()
            .zip(other.iter())
            .map(|(l, r)| func(l, r))
            .collect();
        Ok(Self::new(data))
    }

    pub fn rolling_sum(&self, window: usize) -> Result<Self, ValueVectorError> {
        if window == 0 {
            return Err(ValueVectorError::InvalidWindow);
        }
        if window > self.len() {
            return Ok(Self::new(Vec::new()));
        }
        let mut result = Vec::with_capacity(self.len() - window + 1);
        let mut current = self.data[0..window].iter().sum::<f64>();
        result.push(current);
        for i in window..self.len() {
            current += self.data[i];
            current -= self.data[i - window];
            result.push(current);
        }
        Ok(Self::new(result))
    }

    pub fn rolling_mean(&self, window: usize) -> Result<Self, ValueVectorError> {
        let sum = self.rolling_sum(window)?;
        Ok(sum.map(|value| value / window as f64))
    }

    pub fn diff(&self, period: usize) -> Self {
        if period == 0 || period >= self.len() {
            return Self::new(Vec::new());
        }
        let data: Vec<f64> = self.data[period..]
            .iter()
            .enumerate()
            .map(|(idx, value)| value - self.data[idx])
            .collect();
        Self::new(data)
    }

    pub fn scale(&self, factor: f64) -> Self {
        self.map(|value| value * factor)
    }

    pub fn normalize(&self) -> Self {
        if self.len() <= 1 {
            return self.clone();
        }
        let mean = self.mean().unwrap_or(0.0);
        let variance = self
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / (self.len() as f64);
        let std_dev = variance.sqrt();
        if std_dev == 0.0 {
            return self.clone();
        }
        self.map(|value| (value - mean) / std_dev)
    }
}

impl Add for ValueVector {
    type Output = Result<Self, ValueVectorError>;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip_with(&rhs, |l, r| l + r)
    }
}

impl Sub for ValueVector {
    type Output = Result<Self, ValueVectorError>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.zip_with(&rhs, |l, r| l - r)
    }
}

impl Mul for ValueVector {
    type Output = Result<Self, ValueVectorError>;

    fn mul(self, rhs: Self) -> Self::Output {
        self.zip_with(&rhs, |l, r| l * r)
    }
}

impl Div for ValueVector {
    type Output = Result<Self, ValueVectorError>;

    fn div(self, rhs: Self) -> Self::Output {
        self.zip_with(&rhs, |l, r| l / r)
    }
}

impl<'a> IntoIterator for &'a ValueVector {
    type Item = f64;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, f64>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter().copied()
    }
}
