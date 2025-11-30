use crate::indicators::parameters::ParameterPresets;
use crate::indicators::types::ParameterRange;

pub struct StopParameterPresets;

impl StopParameterPresets {
    pub fn stop_loss_percentage() -> ParameterRange {
        ParameterRange::new(3.5, 15.0, 1.0)
    }

    pub fn take_profit_percentage() -> ParameterRange {
        ParameterRange::new(4.0, 20.0, 1.0)
    }

    pub fn trailing_period() -> ParameterRange {
        ParameterRange::new(10.0, 150.0, 10.0)
    }

    pub fn atr_coefficient() -> ParameterRange {
        ParameterRange::new(2.0, 8.0, 0.5)
    }

    pub fn offset_percent() -> ParameterRange {
        ParameterRange::new(-0.05, 0.05, 0.005)
    }

    pub fn get_range(handler_name: &str, param_name: &str) -> Option<ParameterRange> {
        let handler = handler_name.to_uppercase();
        let param = param_name.to_lowercase();

        match handler.as_str() {
            "STOPLOSSPCT" | "STOP_LOSS_PCT" | "STOPLOSS_PCT" => {
                Self::match_percentage_param(&param)
            }
            "TAKEPROFITPCT" | "TAKE_PROFIT_PCT" => Self::match_take_profit_param(&param),
            "ATRTRAILSTOP" | "ATR_TRAIL_STOP" | "ATR_TRAIL" => Self::match_atr_trail_param(&param),
            "HILOTRAILSTOP" | "HILOTRAILINGSTOP" | "HILO_TRAIL_STOP" | "HILO_TRAIL" => {
                Self::match_hilo_param(&param)
            }
            "PERCENTTRAILSTOP" | "PERCENTTRAILINGSTOP" | "PERCENT_TRAIL_STOP" | "PERCENT_TRAIL" => {
                Self::match_percentage_param(&param)
            }
            "INDICATORSTOP" | "INDICATOR_STOP" | "IND_STOP" => {
                Self::match_indicator_stop_param(&param)
            }
            _ => None,
        }
    }

    fn match_percentage_param(param: &str) -> Option<ParameterRange> {
        match param {
            "percentage" | "stop_loss" | "stop" | "value" | "pct" => {
                Some(Self::stop_loss_percentage())
            }
            _ => None,
        }
    }

    fn match_take_profit_param(param: &str) -> Option<ParameterRange> {
        match param {
            "percentage" | "take_profit" | "take" | "value" | "pct" => {
                Some(Self::take_profit_percentage())
            }
            _ => None,
        }
    }

    fn match_atr_trail_param(param: &str) -> Option<ParameterRange> {
        match param {
            "period" => Some(Self::trailing_period()),
            "coeff_atr" | "coeff" | "atr_coeff" => Some(Self::atr_coefficient()),
            _ => None,
        }
    }

    fn match_hilo_param(param: &str) -> Option<ParameterRange> {
        match param {
            "period" => Some(Self::trailing_period()),
            _ => None,
        }
    }

    fn match_indicator_stop_param(param: &str) -> Option<ParameterRange> {
        match param {
            "period" => Some(ParameterPresets::standard_period()),
            "coeff_atr" | "coeff" | "multiplier" => {
                Some(ParameterPresets::get_multiplier_range(param))
            }
            "offset_percent" | "offset" | "offset_pct" => Some(Self::offset_percent()),
            _ => None,
        }
    }
}
