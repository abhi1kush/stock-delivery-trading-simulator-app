//! Causal long-only delivery-trade backtester.
//!
//! Ported from the `stock_simulator` engine (same indicator math, same
//! next-open fills). See `docs/CAUSALITY.md`. This crate has no CLI and no
//! filesystem access so the WASM wrapper can call it directly.

pub mod data;
pub mod engine;
pub mod error;
pub mod indicators;
pub mod policy;
pub mod report;

pub use data::{load_prices_csv, Bar};
pub use engine::run_backtest;
pub use error::{Result, SimError};
pub use policy::parse_policy_text;
pub use report::Stats;

use policy::Policy;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RunOutput {
    pub stats: Stats,
    pub policy: PolicySummary,
    pub equity: Vec<EquityOut>,
    pub prices: Vec<PriceOut>,
    pub trades: Vec<TradeOut>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicySummary {
    pub starting_cash: f64,
    pub position_sizing: String,
    pub fraction: f64,
    pub ema_fast: Option<usize>,
    pub ema_slow: Option<usize>,
    pub adx_min: Option<f64>,
    pub rsi_period: Option<usize>,
    pub rsi_overbought: f64,
    pub sma_trend: Option<usize>,
    pub require_price_above_sma_trend: bool,
    pub stop_loss_pct: Option<f64>,
    pub trailing_stop_pct: Option<f64>,
    pub atr_stop_mult: Option<f64>,
    pub trailing_atr_mult: Option<f64>,
    pub buy_on_ema_cross_up: bool,
    pub buy_on_adx_confirm: bool,
    pub require_rsi_not_overbought: bool,
    pub fill_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquityOut {
    pub date: String,
    pub equity: f64,
    pub cash: f64,
    pub shares: i64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceOut {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub sma_trend: Option<f64>,
    pub ema_fast: Option<f64>,
    pub ema_slow: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeOut {
    pub entry_date: String,
    pub exit_date: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub shares: i64,
    pub side: String,
    pub exit_reason: String,
    pub pnl: f64,
    pub return_pct: f64,
    pub hold_days: i64,
}

/// Run a backtest from an OHLCV CSV string and a YAML or JSON policy string.
pub fn execute(prices_csv: &str, policy_text: &str) -> Result<RunOutput> {
    let bars = load_prices_csv(prices_csv)?;
    let policy = parse_policy_text(policy_text)?;
    let result = run_backtest(bars, &policy);
    Ok(assemble(&result, &policy))
}

fn assemble(result: &engine::BacktestResult, policy: &Policy) -> RunOutput {
    let stats = report::compute_stats(result);
    let equity = result
        .equity_curve
        .iter()
        .map(|e| EquityOut {
            date: e.date.to_string(),
            equity: e.equity,
            cash: e.cash,
            shares: e.shares,
            close: e.close,
        })
        .collect();
    let prices = result
        .bars
        .iter()
        .zip(result.indicators.iter())
        .map(|(bar, ind)| PriceOut {
            date: bar.date.to_string(),
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume,
            sma_trend: ind.sma_trend,
            ema_fast: ind.ema_fast,
            ema_slow: ind.ema_slow,
        })
        .collect();
    let trades = result
        .trades
        .iter()
        .map(|t| TradeOut {
            entry_date: t.entry_date.to_string(),
            exit_date: t.exit_date.to_string(),
            entry_price: t.entry_price,
            exit_price: t.exit_price,
            shares: t.shares,
            side: t.side.clone(),
            exit_reason: t.exit_reason.clone(),
            pnl: t.pnl,
            return_pct: t.return_pct,
            hold_days: t.hold_days,
        })
        .collect();
    RunOutput {
        stats,
        policy: summarize_policy(policy),
        equity,
        prices,
        trades,
    }
}

fn summarize_policy(p: &Policy) -> PolicySummary {
    PolicySummary {
        starting_cash: p.capital.starting_cash,
        position_sizing: p.capital.position_sizing.clone(),
        fraction: p.capital.fraction,
        ema_fast: p.indicators.ema_fast,
        ema_slow: p.indicators.ema_slow,
        adx_min: p.indicators.adx_min,
        rsi_period: p.indicators.rsi_period,
        rsi_overbought: p.indicators.rsi_overbought,
        sma_trend: p.indicators.sma_trend,
        require_price_above_sma_trend: p.signals.require_price_above_sma_trend,
        stop_loss_pct: p.risk.stop_loss_pct,
        trailing_stop_pct: p.risk.trailing_stop_pct,
        atr_stop_mult: p.risk.atr_stop_mult,
        trailing_atr_mult: p.risk.trailing_atr_mult,
        buy_on_ema_cross_up: p.signals.buy_on_ema_cross_up,
        buy_on_adx_confirm: p.signals.buy_on_adx_confirm,
        require_rsi_not_overbought: p.signals.require_rsi_not_overbought,
        fill_at: p.execution.fill_at.clone(),
    }
}
