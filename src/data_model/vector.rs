use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;

use thiserror::Error;

use super::quote::Quote;
use super::vector_ops::unsafe_ops;

#[derive(Debug, Error)]
pub enum ValueVectorError {
    #[error("length mismatch: left {left}, right {right}")]
    LengthMismatch { left: usize, right: usize },
    #[error("window must be greater than zero")]
    InvalidWindow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValueVector {
    data: Arc<[f32]>,
}

impl ValueVector {
    pub fn new(data: Vec<f32>) -> Self {
        Self { data: data.into() }
    }

    pub fn from_quotes<F>(quotes: &[Quote], extractor: F) -> Self
    where
        F: Fn(&Quote) -> f32,
    {
        let data: Vec<f32> = quotes.iter().map(extractor).collect();
        Self::new(data)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.data.iter().copied()
    }

    pub fn sum(&self) -> f32 {
        unsafe_ops::sum_f32_fast(&self.data)
    }

    pub fn mean(&self) -> Option<f32> {
        if self.is_empty() {
            None
        } else {
            Some(self.sum() / self.len() as f32)
        }
    }

    pub fn map<F>(&self, func: F) -> Self
    where
        F: Fn(f32) -> f32,
    {
        let data: Vec<f32> = self.iter().map(func).collect();
        Self::new(data)
    }

    pub fn zip_with<F>(&self, other: &Self, func: F) -> Result<Self, ValueVectorError>
    where
        F: Fn(f32, f32) -> f32,
    {
        if self.len() != other.len() {
            return Err(ValueVectorError::LengthMismatch {
                left: self.len(),
                right: other.len(),
            });
        }
        let mut data = Vec::with_capacity(self.len());
        unsafe {
            let self_ptr: *const f32 = self.data.as_ptr();
            let other_ptr: *const f32 = other.data.as_ptr();
            let data_ptr: *mut f32 = data.as_mut_ptr();
            data.set_len(self.len());
            
            for i in 0..self.len() {
                *data_ptr.add(i) = func(*self_ptr.add(i), *other_ptr.add(i));
            }
        }
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
        let mut current = unsafe_ops::sum_f32_fast(&self.data[0..window]);
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
        Ok(sum.map(|value| value / window as f32))
    }

    pub fn diff(&self, period: usize) -> Self {
        if period == 0 || period >= self.len() {
            return Self::new(Vec::new());
        }
        let data: Vec<f32> = self.data[period..]
            .iter()
            .enumerate()
            .map(|(idx, value)| value - self.data[idx])
            .collect();
        Self::new(data)
    }

    pub fn scale(&self, factor: f32) -> Self {
        self.map(|value| value * factor)
    }

    pub fn normalize(&self) -> Self {
        if self.len() <= 1 {
            return self.clone();
        }
        let mean = self.mean().unwrap_or(0.0);
        let variance = unsafe_ops::sum_sq_diff_f32_fast(&self.data, mean) / (self.len() as f32);
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
    type Item = f32;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, f32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter().copied()
    }
}
