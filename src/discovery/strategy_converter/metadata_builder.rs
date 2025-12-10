use crate::discovery::engine::StrategyCandidate;
use crate::strategy::types::{StrategyCategory, StrategyMetadata};
use chrono::Utc;

pub struct MetadataBuilder;

impl MetadataBuilder {
    pub fn create_metadata(candidate: &StrategyCandidate) -> StrategyMetadata {
        let indicator_names: Vec<String> = candidate
            .indicators
            .iter()
            .map(|ind| ind.name.clone())
            .collect();
        let nested_names: Vec<String> = candidate
            .nested_indicators
            .iter()
            .map(|nested| nested.indicator.name.clone())
            .collect();
        let all_names = [indicator_names, nested_names].concat();
        let name = format!("Auto Strategy: {}", all_names.join(" + "));

        let condition_names: Vec<String> = candidate
            .conditions
            .iter()
            .map(|cond| cond.name.clone())
            .collect();
        let description = Some(format!(
            "Автоматически сгенерированная стратегия. Индикаторы: {}. Условия: {}.",
            all_names.join(", "),
            condition_names.join(", ")
        ));

        StrategyMetadata {
            id: format!("auto_strategy_{}", Utc::now().timestamp()),
            name,
            description,
            version: Some("1.0.0".to_string()),
            author: Some("Strategy Discovery Engine".to_string()),
            categories: vec![StrategyCategory::Custom("Auto Generated".to_string())],
            tags: vec!["auto-generated".to_string(), "discovery".to_string()],
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }
}
