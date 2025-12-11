use crate::discovery::types::ConditionInfo;
use crate::optimization::condition_id::ConditionId;
use crate::strategy::types::StrategyParameterSpec;

use super::helpers::get_condition_param_range;

pub fn extract_condition_parameters(
    conditions: &[ConditionInfo],
    prefix: &str,
) -> Vec<StrategyParameterSpec> {
    let estimated_capacity = conditions.len() * 2;
    let mut params = Vec::with_capacity(estimated_capacity);

    for condition in conditions {
        for param in &condition.optimization_params {
            if param.optimizable {
                let param_name = if prefix == "exit" {
                    ConditionId::parameter_name(&format!("exit_{}", condition.id), &param.name)
                } else {
                    ConditionId::parameter_name(&condition.id, &param.name)
                };

                let (default_val, min_val, max_val, step_val) =
                    get_condition_param_range(&param.name);

                params.push(StrategyParameterSpec::new_numeric(
                    param_name,
                    Some(format!(
                        "{} parameter for {} condition {}",
                        param.name, prefix, condition.name
                    )),
                    crate::strategy::types::StrategyParamValue::Number(default_val),
                    min_val,
                    max_val,
                    step_val,
                    param.optimizable,
                    param.mutatable,
                ));
            }
        }
    }

    params
}

