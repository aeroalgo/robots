use crate::discovery::StrategyCandidate;
use crate::optimization::per_structure_optimizer::OptimizedStrategyResult;
use crate::strategy::types::StrategyDefinition;
use crate::discovery::strategy_converter::StrategyConverter;
use serde_json;

pub struct StrategySaver;

impl StrategySaver {
    pub fn new() -> Self {
        Self
    }

    pub fn convert_to_definition(
        &self,
        result: &OptimizedStrategyResult,
        base_timeframe: crate::data_model::types::TimeFrame,
    ) -> Result<StrategyDefinition, anyhow::Error> {
        let mut strategy_def = StrategyConverter::candidate_to_definition(
            &result.candidate,
            base_timeframe,
        )?;
        
        for (param_name, param_value) in &result.parameters {
            strategy_def.parameters.iter_mut()
                .find(|p| p.name == *param_name)
                .map(|p| {
                    p.default_value = match param_value {
                        crate::strategy::types::StrategyParamValue::Number(n) => {
                            crate::strategy::types::StrategyParamValue::Number(*n)
                        }
                        crate::strategy::types::StrategyParamValue::Integer(i) => {
                            crate::strategy::types::StrategyParamValue::Integer(*i)
                        }
                        crate::strategy::types::StrategyParamValue::Flag(b) => {
                            crate::strategy::types::StrategyParamValue::Flag(*b)
                        }
                        _ => param_value.clone(),
                    };
                });
        }
        
        Ok(strategy_def)
    }

    pub fn format_for_storage(
        &self,
        result: &OptimizedStrategyResult,
    ) -> String {
        let mut parts = Vec::new();

        parts.push(format!("Fitness: {:.4}", result.fitness));

        if let Some(pf) = result.backtest_report.metrics.profit_factor {
            parts.push(format!("Profit Factor: {:.2}", pf));
        }
        if let Some(sharpe) = result.backtest_report.metrics.sharpe_ratio {
            parts.push(format!("Sharpe Ratio: {:.2}", sharpe));
        }
        parts.push(format!(
            "Total Profit: {:.2}",
            result.backtest_report.metrics.total_profit
        ));
        parts.push(format!(
            "Win Rate: {:.1}%",
            result.backtest_report.metrics.winning_percentage * 100.0
        ));
        parts.push(format!("Trades: {}", result.backtest_report.trades.len()));

        parts.join(", ")
    }

    pub fn serialize_for_db(
        &self,
        result: &OptimizedStrategyResult,
        base_timeframe: crate::data_model::types::TimeFrame,
    ) -> Result<String, anyhow::Error> {
        let strategy_def = self.convert_to_definition(result, base_timeframe)?;
        
        let db_record = serde_json::json!({
            "strategy": {
                "metadata": {
                    "name": strategy_def.metadata.name,
                    "description": strategy_def.metadata.description,
                },
                "parameters": strategy_def.parameters.iter().map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "value": match &p.default_value {
                            crate::strategy::types::StrategyParamValue::Number(n) => serde_json::Value::Number(serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0))),
                            crate::strategy::types::StrategyParamValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
                            crate::strategy::types::StrategyParamValue::Flag(b) => serde_json::Value::Bool(*b),
                            _ => serde_json::Value::String(format!("{:?}", p.default_value)),
                        }
                    })
                }).collect::<Vec<_>>(),
            },
            "metrics": {
                "fitness": result.fitness,
                "total_profit": result.backtest_report.metrics.total_profit,
                "profit_factor": result.backtest_report.metrics.profit_factor,
                "sharpe_ratio": result.backtest_report.metrics.sharpe_ratio,
                "win_rate": result.backtest_report.metrics.winning_percentage,
                "trades_count": result.backtest_report.trades.len(),
            }
        });
        
        Ok(serde_json::to_string_pretty(&db_record)?)
    }
}

impl Default for StrategySaver {
    fn default() -> Self {
        Self::new()
    }
}

