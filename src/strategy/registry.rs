use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::base::Strategy;
use super::builder::{DynamicStrategy, StrategyBuilder};
use super::types::{StrategyDefinition, StrategyError, StrategyId, StrategyParameterMap};

pub struct StrategyRegistry {
    definitions: RwLock<HashMap<StrategyId, StrategyDefinition>>,
    instances: RwLock<HashMap<StrategyId, Arc<dyn Strategy>>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            definitions: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_definition(&self, definition: StrategyDefinition) {
        let id = definition.metadata.id.clone();
        self.definitions.write().unwrap().insert(id, definition);
    }

    pub fn upsert_definition(&self, definition: StrategyDefinition) {
        self.register_definition(definition);
    }

    pub fn remove_definition(&self, id: &str) {
        self.definitions.write().unwrap().remove(id);
        self.instances.write().unwrap().remove(id);
    }

    pub fn definition(&self, id: &str) -> Option<StrategyDefinition> {
        self.definitions.read().unwrap().get(id).cloned()
    }

    pub fn list_definitions(&self) -> Vec<StrategyDefinition> {
        self.definitions.read().unwrap().values().cloned().collect()
    }

    pub fn list_ids(&self) -> Vec<StrategyId> {
        self.definitions.read().unwrap().keys().cloned().collect()
    }

    pub fn strategy(&self, id: &str) -> Option<Arc<dyn Strategy>> {
        self.instances.read().unwrap().get(id).cloned()
    }

    pub fn register_strategy(&self, strategy: Arc<dyn Strategy>) {
        let id = strategy.id().to_string();
        self.instances.write().unwrap().insert(id, strategy);
    }

    pub fn build_strategy(
        &self,
        id: &str,
        overrides: Option<StrategyParameterMap>,
    ) -> Result<Arc<dyn Strategy>, StrategyError> {
        let definition = self
            .definition(id)
            .ok_or_else(|| StrategyError::DefinitionError(format!("strategy {} not found", id)))?;
        let builder = StrategyBuilder::new(definition.clone());
        let builder = if let Some(parameters) = overrides {
            builder.with_parameters(parameters)
        } else {
            builder
        };
        let strategy = builder.build()?;
        self.store_instance(strategy)
    }

    pub fn store_instance(
        &self,
        strategy: DynamicStrategy,
    ) -> Result<Arc<dyn Strategy>, StrategyError> {
        let id = strategy.metadata().id.clone();
        let definition = strategy.definition().clone();
        self.register_definition(definition);
        let arc: Arc<dyn Strategy> = Arc::new(strategy);
        self.instances
            .write()
            .unwrap()
            .insert(id.clone(), Arc::clone(&arc));
        Ok(arc)
    }

    pub fn clear_instances(&self) {
        self.instances.write().unwrap().clear();
    }
}
