use std::fmt::{Display, Formatter};
use std::sync::Arc;

use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub type Price = f64;
pub type Volume = f64;
pub type TimestampMillis = i64;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Symbol {
    code: Arc<str>,
    exchange: Option<Arc<str>>,
}

impl Symbol {
    pub fn new<S: Into<Arc<str>>>(code: S) -> Self {
        Self {
            code: code.into(),
            exchange: None,
        }
    }

    pub fn with_exchange<S: Into<Arc<str>>, E: Into<Arc<str>>>(code: S, exchange: E) -> Self {
        Self {
            code: code.into(),
            exchange: Some(exchange.into()),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn exchange(&self) -> Option<&str> {
        self.exchange.as_deref()
    }

    pub fn from_descriptor(descriptor: &str) -> Self {
        if let Some((exchange, code)) = descriptor.split_once(':') {
            return Self::with_exchange(code.to_string(), exchange.to_string());
        }
        Self::new(descriptor.to_string())
    }

    pub fn with_optional_exchange<S: Into<Arc<str>>, E: Into<Option<Arc<str>>>>(
        code: S,
        exchange: E,
    ) -> Self {
        Self {
            code: code.into(),
            exchange: exchange.into(),
        }
    }

    pub fn descriptor(&self) -> String {
        match self.exchange() {
            Some(exchange) => format!("{}:{}", exchange, self.code()),
            None => self.code().to_string(),
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.exchange() {
            Some(exchange) => write!(f, "{}:{}", exchange, self.code()),
            None => write!(f, "{}", self.code()),
        }
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.descriptor())
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Symbol::from_descriptor(&value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeFrame {
    Minutes(u32),
    Hours(u32),
    Days(u32),
    Weeks(u32),
    Months(u32),
    Custom(String),
}

impl TimeFrame {
    pub fn minutes(value: u32) -> Self {
        Self::Minutes(value)
    }

    pub fn hours(value: u32) -> Self {
        Self::Hours(value)
    }

    pub fn days(value: u32) -> Self {
        Self::Days(value)
    }

    pub fn duration(&self) -> Option<Duration> {
        match self {
            Self::Minutes(v) => Some(Duration::minutes(*v as i64)),
            Self::Hours(v) => Some(Duration::hours(*v as i64)),
            Self::Days(v) => Some(Duration::days(*v as i64)),
            Self::Weeks(v) => Some(Duration::weeks(*v as i64)),
            Self::Months(_) => None,
            Self::Custom(_) => None,
        }
    }

    pub fn identifier(&self) -> String {
        match self {
            Self::Minutes(v) => v.to_string(),
            Self::Hours(v) => format!("{}h", v),
            Self::Days(v) => format!("{}d", v),
            Self::Weeks(v) => format!("{}w", v),
            Self::Months(v) => format!("{}mo", v),
            Self::Custom(v) => v.clone(),
        }
    }

    pub fn total_minutes(&self) -> Option<u64> {
        match self {
            Self::Minutes(v) => Some(*v as u64),
            Self::Hours(v) => Some((*v as u64) * 60),
            Self::Days(v) => Some((*v as u64) * 60 * 24),
            Self::Weeks(v) => Some((*v as u64) * 60 * 24 * 7),
            Self::Months(_) => None,
            Self::Custom(_) => None,
        }
    }

    pub fn total_seconds(&self) -> Option<u64> {
        self.total_minutes().map(|minutes| minutes * 60)
    }

    pub fn from_identifier(value: &str) -> Self {
        let lower = value.to_ascii_lowercase();
        if let Ok(minutes) = lower.parse::<u32>() {
            return Self::Minutes(minutes);
        }
        if let Some(stripped) = lower.strip_suffix('h') {
            if let Ok(hours) = stripped.parse::<u32>() {
                return Self::Hours(hours);
            }
        }
        if let Some(stripped) = lower.strip_suffix('d') {
            if let Ok(days) = stripped.parse::<u32>() {
                return Self::Days(days);
            }
        }
        if let Some(stripped) = lower.strip_suffix('w') {
            if let Ok(weeks) = stripped.parse::<u32>() {
                return Self::Weeks(weeks);
            }
        }
        if let Some(stripped) = lower.strip_suffix("mo") {
            if let Ok(months) = stripped.parse::<u32>() {
                return Self::Months(months);
            }
        }
        Self::Custom(value.to_string())
    }

    pub fn align_timestamp(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        match self.duration() {
            Some(duration) => {
                let window = duration.num_milliseconds().max(1);
                let aligned = (timestamp.timestamp_millis() / window) * window;
                timestamp_from_millis(aligned).unwrap_or(timestamp)
            }
            None => timestamp,
        }
    }
}

impl Display for TimeFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuoteId {
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub timestamp: DateTime<Utc>,
}

impl QuoteId {
    pub fn new(symbol: Symbol, timeframe: TimeFrame, timestamp: DateTime<Utc>) -> Self {
        Self {
            symbol,
            timeframe,
            timestamp,
        }
    }
}

pub fn timestamp_from_millis(value: TimestampMillis) -> Option<DateTime<Utc>> {
    let seconds = value / 1000;
    let nanos = ((value % 1000) * 1_000_000) as u32;
    Utc.timestamp_opt(seconds, nanos).single()
}

pub fn timestamp_from_seconds(value: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(value, 0).single()
}

pub fn timestamp_to_millis(value: DateTime<Utc>) -> TimestampMillis {
    value.timestamp_millis()
}

pub fn timestamp_to_seconds(value: DateTime<Utc>) -> i64 {
    value.timestamp()
}
