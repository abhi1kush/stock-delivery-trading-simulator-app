use crate::error::{Result, SimError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Policy {
    #[serde(default)]
    pub capital: Capital,
    #[serde(default)]
    pub risk: Risk,
    #[serde(default)]
    pub indicators: Indicators,
    #[serde(default)]
    pub signals: Signals,
    #[serde(default)]
    pub execution: Execution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capital {
    #[serde(default = "default_cash")]
    pub starting_cash: f64,
    #[serde(default = "default_sizing")]
    pub position_sizing: String,
    #[serde(default = "default_fraction")]
    pub fraction: f64,
    #[serde(default = "default_fixed_shares")]
    pub fixed_shares: i64,
    #[serde(default = "default_fixed_amount")]
    pub fixed_amount: f64,
}

fn default_cash() -> f64 {
    100_000.0
}
fn default_sizing() -> String {
    "fraction".into()
}
fn default_fraction() -> f64 {
    0.95
}
fn default_fixed_shares() -> i64 {
    100
}
fn default_fixed_amount() -> f64 {
    10_000.0
}

impl Default for Capital {
    fn default() -> Self {
        Self {
            starting_cash: default_cash(),
            position_sizing: default_sizing(),
            fraction: default_fraction(),
            fixed_shares: default_fixed_shares(),
            fixed_amount: default_fixed_amount(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Risk {
    #[serde(default)]
    pub stop_loss_pct: Option<f64>,
    #[serde(default)]
    pub take_profit_pct: Option<f64>,
    #[serde(default = "default_atr_stop")]
    pub atr_stop_mult: Option<f64>,
    #[serde(default)]
    pub trailing_stop_pct: Option<f64>,
    #[serde(default = "default_trail_atr")]
    pub trailing_atr_mult: Option<f64>,
    #[serde(default)]
    pub allow_short: bool,
}

fn default_atr_stop() -> Option<f64> {
    Some(2.0)
}
fn default_trail_atr() -> Option<f64> {
    Some(2.5)
}

impl Default for Risk {
    fn default() -> Self {
        Self {
            stop_loss_pct: None,
            take_profit_pct: None,
            atr_stop_mult: default_atr_stop(),
            trailing_stop_pct: None,
            trailing_atr_mult: default_trail_atr(),
            allow_short: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Indicators {
    #[serde(default)]
    pub sma_fast: Option<usize>,
    #[serde(default)]
    pub sma_slow: Option<usize>,
    /// Optional long-term SMA period (e.g. 100) for trend filter.
    #[serde(default)]
    pub sma_trend: Option<usize>,
    #[serde(default = "default_ema_fast")]
    pub ema_fast: Option<usize>,
    #[serde(default = "default_ema_slow")]
    pub ema_slow: Option<usize>,
    #[serde(default = "default_rsi_period")]
    pub rsi_period: Option<usize>,
    #[serde(default = "default_rsi_os")]
    pub rsi_oversold: f64,
    #[serde(default = "default_rsi_ob")]
    pub rsi_overbought: f64,
    #[serde(default = "default_atr_period")]
    pub atr_period: Option<usize>,
    #[serde(default = "default_adx_period")]
    pub adx_period: Option<usize>,
    #[serde(default = "default_adx_min")]
    pub adx_min: Option<f64>,
}

fn default_ema_fast() -> Option<usize> {
    Some(12)
}
fn default_ema_slow() -> Option<usize> {
    Some(26)
}
fn default_rsi_period() -> Option<usize> {
    Some(14)
}
fn default_rsi_os() -> f64 {
    30.0
}
fn default_rsi_ob() -> f64 {
    70.0
}
fn default_atr_period() -> Option<usize> {
    Some(14)
}
fn default_adx_period() -> Option<usize> {
    Some(14)
}
fn default_adx_min() -> Option<f64> {
    Some(20.0)
}

impl Default for Indicators {
    fn default() -> Self {
        Self {
            sma_fast: None,
            sma_slow: None,
            sma_trend: None,
            ema_fast: default_ema_fast(),
            ema_slow: default_ema_slow(),
            rsi_period: default_rsi_period(),
            rsi_oversold: default_rsi_os(),
            rsi_overbought: default_rsi_ob(),
            atr_period: default_atr_period(),
            adx_period: default_adx_period(),
            adx_min: default_adx_min(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signals {
    #[serde(default)]
    pub buy_on_sma_cross_up: bool,
    #[serde(default)]
    pub sell_on_sma_cross_down: bool,
    #[serde(default = "default_true")]
    pub buy_on_ema_cross_up: bool,
    #[serde(default)]
    pub sell_on_ema_cross_down: bool,
    #[serde(default)]
    pub buy_on_rsi_oversold_exit: bool,
    #[serde(default)]
    pub sell_on_rsi_overbought_exit: bool,
    #[serde(default = "default_true")]
    pub require_rsi_not_overbought: bool,
    #[serde(default)]
    pub require_adx_trend: bool,
    /// Buy when ADX crosses up through adx_min while fast EMA > slow EMA
    #[serde(default = "default_true")]
    pub buy_on_adx_confirm: bool,
    /// Block buys when close <= SMA(sma_trend). Off by default; requires indicators.sma_trend.
    #[serde(default)]
    pub require_price_above_sma_trend: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Signals {
    fn default() -> Self {
        Self {
            buy_on_sma_cross_up: false,
            sell_on_sma_cross_down: false,
            buy_on_ema_cross_up: true,
            sell_on_ema_cross_down: false,
            buy_on_rsi_oversold_exit: false,
            sell_on_rsi_overbought_exit: false,
            require_rsi_not_overbought: true,
            require_adx_trend: false,
            buy_on_adx_confirm: true,
            require_price_above_sma_trend: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Execution {
    #[serde(default = "default_fill")]
    pub fill_at: String,
}

fn default_fill() -> String {
    "next_open".into()
}

impl Default for Execution {
    fn default() -> Self {
        Self {
            fill_at: default_fill(),
        }
    }
}

/// Parse a policy from YAML (as in `policy/example_policy.yaml`) or JSON.
///
/// JSON is used when the trimmed text starts with `{`. Anything else is YAML,
/// including documents that begin with comments. Missing fields keep the same
/// serde defaults as the original CLI loader — pass explicit nulls when a
/// default must stay off (notably `atr_stop_mult` and `trailing_atr_mult`).
pub fn parse_policy_text(text: &str) -> Result<Policy> {
    let trimmed = text.trim().trim_start_matches('\u{feff}');
    if trimmed.is_empty() {
        return Err(SimError::Validation("policy is empty".into()));
    }
    let policy: Policy = if trimmed.starts_with('{') {
        serde_json::from_str(trimmed)?
    } else {
        serde_yaml::from_str(trimmed)?
    };
    validate(&policy)?;
    Ok(policy)
}

fn validate(p: &Policy) -> Result<()> {
    if p.capital.starting_cash <= 0.0 {
        return Err(SimError::Validation("starting_cash must be > 0".into()));
    }
    match p.capital.position_sizing.as_str() {
        "fraction" | "fixed_shares" | "fixed_amount" => {}
        other => {
            return Err(SimError::Validation(format!(
                "invalid position_sizing: {other}"
            )));
        }
    }
    if p.risk.allow_short {
        return Err(SimError::Validation(
            "shorting is not supported (set allow_short: false)".into(),
        ));
    }
    if p.execution.fill_at != "next_open" {
        return Err(SimError::Validation(
            "only fill_at: next_open is supported".into(),
        ));
    }
    let pair_ok = |a: Option<usize>, b: Option<usize>, name: &str| -> Result<()> {
        match (a, b) {
            (None, None) => Ok(()),
            (Some(f), Some(s)) if f < s => Ok(()),
            (Some(_), Some(_)) => Err(SimError::Validation(format!(
                "{name}_fast must be < {name}_slow"
            ))),
            _ => Err(SimError::Validation(format!(
                "{name}_fast and {name}_slow must both be set or both null"
            ))),
        }
    };
    pair_ok(p.indicators.sma_fast, p.indicators.sma_slow, "sma")?;
    pair_ok(p.indicators.ema_fast, p.indicators.ema_slow, "ema")?;

    if (p.risk.atr_stop_mult.is_some() || p.risk.trailing_atr_mult.is_some())
        && p.indicators.atr_period.is_none()
    {
        return Err(SimError::Validation(
            "atr_period required when using ATR-based stops".into(),
        ));
    }
    for (name, v) in [
        ("stop_loss_pct", p.risk.stop_loss_pct),
        ("take_profit_pct", p.risk.take_profit_pct),
        ("trailing_stop_pct", p.risk.trailing_stop_pct),
        ("atr_stop_mult", p.risk.atr_stop_mult),
        ("trailing_atr_mult", p.risk.trailing_atr_mult),
    ] {
        if let Some(x) = v {
            if x <= 0.0 {
                return Err(SimError::Validation(format!("{name} must be > 0 when set")));
            }
        }
    }
    if let Some(period) = p.indicators.sma_trend {
        if period == 0 {
            return Err(SimError::Validation(
                "sma_trend must be > 0 when set".into(),
            ));
        }
    }
    if p.signals.require_price_above_sma_trend && p.indicators.sma_trend.is_none() {
        return Err(SimError::Validation(
            "indicators.sma_trend required when signals.require_price_above_sma_trend is true"
                .into(),
        ));
    }
    Ok(())
}
