// Ported from the upstream delivery engine. Indicator windows and next-open
// fills are the causality contract in docs/CAUSALITY.md — do not use future bars.
use crate::data::Bar;
use crate::indicators::{self, IndicatorRow};
use crate::policy::Policy;
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Trade {
    pub entry_date: NaiveDate,
    pub exit_date: NaiveDate,
    pub entry_price: f64,
    pub exit_price: f64,
    pub shares: i64,
    pub side: String,
    pub exit_reason: String,
    pub pnl: f64,
    pub return_pct: f64,
    pub hold_days: i64,
}

#[derive(Debug, Clone)]
pub struct EquityPoint {
    pub date: NaiveDate,
    pub equity: f64,
    pub cash: f64,
    pub shares: i64,
    pub close: f64,
}

#[derive(Debug)]
pub struct BacktestResult {
    pub bars: Vec<Bar>,
    pub indicators: Vec<IndicatorRow>,
    pub equity_curve: Vec<EquityPoint>,
    pub trades: Vec<Trade>,
    pub starting_cash: f64,
    pub ending_equity: f64,
}

struct Pending {
    action: &'static str, // "buy" | "sell"
    reason: String,
}

fn position_shares(cash: f64, price: f64, policy: &Policy) -> i64 {
    if price <= 0.0 {
        return 0;
    }
    let cap = &policy.capital;
    match cap.position_sizing.as_str() {
        "fraction" => {
            let budget = cash * cap.fraction;
            (budget / price).floor() as i64
        }
        "fixed_shares" => {
            let n = cap.fixed_shares;
            if (n as f64) * price <= cash {
                n
            } else {
                (cash / price).floor() as i64
            }
        }
        "fixed_amount" => {
            let budget = cap.fixed_amount.min(cash);
            (budget / price).floor() as i64
        }
        _ => 0,
    }
}

fn want_buy(row: &IndicatorRow, policy: &Policy, close: f64) -> bool {
    let sig = &policy.signals;
    let ind = &policy.indicators;
    let mut want = false;
    if sig.buy_on_sma_cross_up && row.sma_cross_up {
        want = true;
    }
    if sig.buy_on_ema_cross_up && row.ema_cross_up {
        want = true;
    }
    if sig.buy_on_rsi_oversold_exit && row.rsi_exit_oversold {
        want = true;
    }
    // ADX confirmation: ADX crosses above adx_min while EMA trend is already up
    if sig.buy_on_adx_confirm && row.adx_cross_up {
        let ema_up = match (row.ema_fast, row.ema_slow) {
            (Some(f), Some(s)) => f > s,
            _ => false,
        };
        let sma_up = match (row.sma_fast, row.sma_slow) {
            (Some(f), Some(s)) => f > s,
            _ => false,
        };
        if ema_up || sma_up {
            want = true;
        }
    }
    if !want {
        return false;
    }
    if sig.require_rsi_not_overbought && ind.rsi_period.is_some() {
        if let Some(rsi) = row.rsi {
            if rsi >= ind.rsi_overbought {
                return false;
            }
        }
    }
    if sig.require_adx_trend && ind.adx_period.is_some() {
        if let Some(min) = ind.adx_min {
            match row.adx {
                Some(adx) if adx >= min => {}
                _ => return false,
            }
        }
    }
    // Long-term SMA trend filter: only buy when close > SMA(sma_trend).
    // Uses signal-bar close (known at decision time). If SMA not yet ready, skip buy.
    if sig.require_price_above_sma_trend {
        match row.sma_trend {
            Some(sma) if close > sma => {}
            _ => return false,
        }
    }
    true
}

fn want_sell_signal(row: &IndicatorRow, policy: &Policy) -> bool {
    let sig = &policy.signals;
    if sig.sell_on_sma_cross_down && row.sma_cross_down {
        return true;
    }
    if sig.sell_on_ema_cross_down && row.ema_cross_down {
        return true;
    }
    if sig.sell_on_rsi_overbought_exit && row.rsi_exit_overbought {
        return true;
    }
    false
}

/// Run a long-only delivery backtest with next-open fills.
///
/// Causality: signals use indicators at bar `i` only; fills occur at `i+1` open.
/// Same-day high/low may arm stops/TP after entry; ATR levels use prior-bar ATR.
/// See docs/CAUSALITY.md.
pub fn run_backtest(bars: Vec<Bar>, policy: &Policy) -> BacktestResult {
    let indicators = indicators::compute(&bars, policy);
    let n = bars.len();
    let starting_cash = policy.capital.starting_cash;
    let mut cash = starting_cash;
    let mut shares: i64 = 0;
    let mut entry_price: Option<f64> = None;
    let mut entry_date: Option<NaiveDate> = None;
    let mut pending: Option<Pending> = None;
    let mut trades: Vec<Trade> = Vec::new();
    let mut equity_curve: Vec<EquityPoint> = Vec::new();

    let stop_pct = policy.risk.stop_loss_pct;
    let tp_pct = policy.risk.take_profit_pct;
    let atr_stop_mult = policy.risk.atr_stop_mult;
    let trail_pct = policy.risk.trailing_stop_pct;
    let trail_atr_mult = policy.risk.trailing_atr_mult;

    let mut hard_stop: Option<f64> = None;
    let mut peak_high: Option<f64> = None;
    let mut trail_stop: Option<f64> = None;

    for i in 0..n {
        let bar = &bars[i];
        let row = &indicators[i];
        let o = bar.open;
        let h = bar.high;
        let l = bar.low;
        let c = bar.close;
        // ATR known at the open / for intraday stop levels = prior bar only.
        // indicators[i].atr includes bar i's true range (not knowable at open).
        let atr_prior = if i > 0 { indicators[i - 1].atr } else { None };

        if let Some(p) = pending.take() {
            if p.action == "buy" && shares == 0 {
                let qty = position_shares(cash, o, policy);
                if qty > 0 {
                    cash -= qty as f64 * o;
                    shares = qty;
                    entry_price = Some(o);
                    entry_date = Some(bar.date);
                    // Peak starts at entry; day's high is applied later in this bar.
                    peak_high = Some(o);
                    let mut stops = Vec::new();
                    if let Some(pct) = stop_pct {
                        stops.push(o * (1.0 - pct / 100.0));
                    }
                    if let (Some(mult), Some(atr)) = (atr_stop_mult, atr_prior) {
                        stops.push(o - mult * atr);
                    }
                    hard_stop = stops.into_iter().fold(None, |acc, v| {
                        Some(match acc {
                            Some(a) => a.max(v),
                            None => v,
                        })
                    });
                    trail_stop = None;
                }
            } else if p.action == "sell" {
                if let (true, Some(ep), Some(ed)) = (shares > 0, entry_price, entry_date) {
                    let proceeds = shares as f64 * o;
                    let pnl = proceeds - shares as f64 * ep;
                    let ret = (o / ep - 1.0) * 100.0;
                    let hold = (bar.date - ed).num_days();
                    trades.push(Trade {
                        entry_date: ed,
                        exit_date: bar.date,
                        entry_price: ep,
                        exit_price: o,
                        shares,
                        side: "long".into(),
                        exit_reason: p.reason,
                        pnl,
                        return_pct: ret,
                        hold_days: hold,
                    });
                    cash += proceeds;
                    shares = 0;
                    entry_price = None;
                    entry_date = None;
                    hard_stop = None;
                    peak_high = None;
                    trail_stop = None;
                }
            }
        }

        let equity = cash + shares as f64 * c;
        equity_curve.push(EquityPoint {
            date: bar.date,
            equity,
            cash,
            shares,
            close: c,
        });

        // Same-day high/low may trigger stops/TP after entry (see docs/CAUSALITY.md).
        // Trail distance uses prior-bar ATR so levels checked vs today's low are causal.
        if shares > 0 && entry_price.is_some() && pending.is_none() {
            peak_high = Some(peak_high.map(|p| p.max(h)).unwrap_or(h));

            let mut candidates = Vec::new();
            if let (Some(pct), Some(peak)) = (trail_pct, peak_high) {
                candidates.push(peak * (1.0 - pct / 100.0));
            }
            if let (Some(mult), Some(atr), Some(peak)) = (trail_atr_mult, atr_prior, peak_high) {
                candidates.push(peak - mult * atr);
            }
            if let Some(new_trail) = candidates.into_iter().fold(None, |acc: Option<f64>, v| {
                Some(match acc {
                    Some(a) => a.max(v),
                    None => v,
                })
            }) {
                trail_stop = Some(match trail_stop {
                    Some(t) if t > new_trail => t,
                    _ => new_trail,
                });
            }

            let active_stop = match (hard_stop, trail_stop) {
                (Some(h), Some(t)) => Some(h.max(t)),
                (Some(h), None) => Some(h),
                (None, Some(t)) => Some(t),
                (None, None) => None,
            };

            let stop_hit = active_stop.map(|s| l <= s).unwrap_or(false);
            let tp_hit = if let (Some(pct), Some(ep)) = (tp_pct, entry_price) {
                h >= ep * (1.0 + pct / 100.0)
            } else {
                false
            };

            if stop_hit {
                let reason = if let Some(ts) = trail_stop {
                    let use_trail = hard_stop.map(|hs| ts >= hs).unwrap_or(true) && l <= ts;
                    if use_trail {
                        "trailing_stop"
                    } else {
                        "stop_loss"
                    }
                } else {
                    "stop_loss"
                };
                pending = Some(Pending {
                    action: "sell",
                    reason: reason.into(),
                });
            } else if tp_hit {
                pending = Some(Pending {
                    action: "sell",
                    reason: "take_profit".into(),
                });
            }
        }

        if pending.is_none() {
            if shares > 0 && want_sell_signal(row, policy) {
                pending = Some(Pending {
                    action: "sell",
                    reason: "signal".into(),
                });
            } else if shares == 0 && want_buy(row, policy, c) && i + 1 < n {
                pending = Some(Pending {
                    action: "buy",
                    reason: "signal".into(),
                });
            }
        }
    }

    // Force-close at last close
    if shares > 0 {
        if let (Some(ep), Some(ed)) = (entry_price, entry_date) {
            let last = &bars[n - 1];
            let proceeds = shares as f64 * last.close;
            let pnl = proceeds - shares as f64 * ep;
            let ret = (last.close / ep - 1.0) * 100.0;
            let hold = (last.date - ed).num_days();
            trades.push(Trade {
                entry_date: ed,
                exit_date: last.date,
                entry_price: ep,
                exit_price: last.close,
                shares,
                side: "long".into(),
                exit_reason: "end_of_data".into(),
                pnl,
                return_pct: ret,
                hold_days: hold,
            });
            cash += proceeds;
            if let Some(last_eq) = equity_curve.last_mut() {
                last_eq.equity = cash;
                last_eq.cash = cash;
                last_eq.shares = 0;
            }
        }
    }

    let ending_equity = equity_curve.last().map(|e| e.equity).unwrap_or(cash);

    BacktestResult {
        bars,
        indicators,
        equity_curve,
        trades,
        starting_cash,
        ending_equity,
    }
}
