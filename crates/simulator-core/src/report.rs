use crate::engine::BacktestResult;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub starting_capital: f64,
    pub ending_equity: f64,
    pub total_return_pct: f64,
    pub total_pnl: f64,
    pub num_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: f64,
    pub max_drawdown_pct: f64,
    pub avg_hold_days: f64,
    pub num_bars: usize,
    pub first_date: String,
    pub last_date: String,
}

pub fn compute_stats(result: &BacktestResult) -> Stats {
    let closed = &result.trades;
    let wins = closed.iter().filter(|t| t.pnl > 0.0).count();
    let total_return = if result.starting_cash > 0.0 {
        (result.ending_equity / result.starting_cash - 1.0) * 100.0
    } else {
        0.0
    };
    let avg_hold = if closed.is_empty() {
        0.0
    } else {
        closed.iter().map(|t| t.hold_days as f64).sum::<f64>() / closed.len() as f64
    };
    let win_rate = if closed.is_empty() {
        0.0
    } else {
        wins as f64 / closed.len() as f64 * 100.0
    };
    let mdd = max_drawdown(
        &result
            .equity_curve
            .iter()
            .map(|e| e.equity)
            .collect::<Vec<_>>(),
    );
    let total_pnl: f64 = closed.iter().map(|t| t.pnl).sum();
    let first_date = result
        .bars
        .first()
        .map(|b| b.date.to_string())
        .unwrap_or_default();
    let last_date = result
        .bars
        .last()
        .map(|b| b.date.to_string())
        .unwrap_or_default();

    Stats {
        starting_capital: result.starting_cash,
        ending_equity: result.ending_equity,
        total_return_pct: total_return,
        total_pnl,
        num_trades: closed.len(),
        winning_trades: wins,
        losing_trades: closed.len() - wins,
        win_rate_pct: win_rate,
        max_drawdown_pct: mdd,
        avg_hold_days: avg_hold,
        num_bars: result.bars.len(),
        first_date,
        last_date,
    }
}

fn max_drawdown(equity: &[f64]) -> f64 {
    if equity.is_empty() {
        return 0.0;
    }
    let mut peak = equity[0];
    let mut max_dd = 0.0;
    for &e in equity {
        if e > peak {
            peak = e;
        }
        if peak > 0.0 {
            let dd = (peak - e) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}
