// Ported from the upstream delivery engine. Indicator windows and next-open
// fills are the causality contract in docs/CAUSALITY.md — do not use future bars.
use crate::data::Bar;
use crate::patterns::{self, PatternHit};
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
    /// What armed the buy (e.g. "Signals + Hammer").
    pub entry_reason: String,
    /// Score breakdown, pattern/signal details and fill.
    pub entry_explanation: String,
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
    explanation: String,
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

/// One contribution to the buy score.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreComponent {
    /// "Signals" or the pattern name.
    pub label: String,
    /// Weight (signal_weight or the pattern weight).
    pub weight: f64,
    /// 1.0 for signals; pattern strength (0.5..=1) otherwise.
    pub strength: f64,
    pub contribution: f64,
    pub detail: String,
}

/// Buy score for one candle, before filters.
#[derive(Debug, Clone, PartialEq)]
pub struct BuyScore {
    pub components: Vec<ScoreComponent>,
    pub total: f64,
    pub threshold: f64,
}

impl BuyScore {
    pub fn passes(&self) -> bool {
        // Small epsilon so that e.g. 0.3 + 0.2 meets a 0.5 threshold.
        !self.components.is_empty() && self.total + 1e-9 >= self.threshold
    }

    pub fn breakdown(&self) -> String {
        let parts: Vec<String> = self
            .components
            .iter()
            .map(|c| {
                if c.label == "Signals" {
                    format!("signals {:.2}×1 = {:.2}", c.weight, c.contribution)
                } else {
                    format!("{} {:.2}×{:.2} = {:.2}", c.label, c.weight, c.strength, c.contribution)
                }
            })
            .collect();
        format!(
            "Buy score {:.2} ≥ threshold {:.2} [{}]",
            self.total,
            self.threshold,
            parts.join(" + ")
        )
    }

    pub fn short_reason(&self) -> String {
        self.components
            .iter()
            .map(|c| c.label.clone())
            .collect::<Vec<_>>()
            .join(" + ")
    }
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.2}")).unwrap_or_else(|| "n/a".into())
}

/// Which indicator buy signals fired on candle i (descriptions), ignoring filters.
pub fn indicator_signals(bar: &Bar, row: &IndicatorRow, policy: &Policy) -> Vec<String> {
    let sig = &policy.signals;
    let ind = &policy.indicators;
    let mut out = Vec::new();
    if sig.buy_on_sma_cross_up && row.sma_cross_up {
        out.push(format!(
            "SMA {} crossed above SMA {} on {} ({} vs {})",
            ind.sma_fast.unwrap_or(0),
            ind.sma_slow.unwrap_or(0),
            bar.date,
            fmt_opt(row.sma_fast),
            fmt_opt(row.sma_slow)
        ));
    }
    if sig.buy_on_ema_cross_up && row.ema_cross_up {
        out.push(format!(
            "EMA {} crossed above EMA {} on {} ({} vs {})",
            ind.ema_fast.unwrap_or(0),
            ind.ema_slow.unwrap_or(0),
            bar.date,
            fmt_opt(row.ema_fast),
            fmt_opt(row.ema_slow)
        ));
    }
    if sig.buy_on_rsi_oversold_exit && row.rsi_exit_oversold {
        out.push(format!(
            "RSI rose back above {} on {} (RSI {})",
            ind.rsi_oversold,
            bar.date,
            fmt_opt(row.rsi)
        ));
    }
    // ADX confirmation: ADX crosses above adx_min while EMA trend is already up
    if sig.buy_on_adx_confirm && row.adx_cross_up {
        let ema_up = matches!((row.ema_fast, row.ema_slow), (Some(f), Some(s)) if f > s);
        let sma_up = matches!((row.sma_fast, row.sma_slow), (Some(f), Some(s)) if f > s);
        if ema_up || sma_up {
            out.push(format!(
                "ADX crossed up through {} on {} (ADX {}) with the fast average above the slow one",
                ind.adx_min.unwrap_or(0.0),
                bar.date,
                fmt_opt(row.adx)
            ));
        }
    }
    out
}

/// Enabled patterns found on candle i.
pub fn pattern_hits(bars: &[Bar], i: usize, policy: &Policy) -> Vec<(PatternHit, f64)> {
    let p = &policy.patterns;
    let lb = p.downtrend_lookback;
    let mut out = Vec::new();
    if p.hammer {
        if let Some(h) = patterns::hammer(bars, i, lb) {
            out.push((h, p.hammer_weight));
        }
    }
    if p.three_white_soldiers {
        if let Some(h) = patterns::three_white_soldiers(bars, i) {
            out.push((h, p.three_white_soldiers_weight));
        }
    }
    if p.bullish_engulfing {
        if let Some(h) = patterns::bullish_engulfing(bars, i) {
            out.push((h, p.bullish_engulfing_weight));
        }
    }
    if p.doji {
        if let Some(h) = patterns::doji(bars, i, lb) {
            out.push((h, p.doji_weight));
        }
    }
    out
}

/// Deterministic weighted buy score:
/// signal_weight × [any indicator signal] + Σ pattern weight × strength.
pub fn score(signals: &[String], hits: &[(PatternHit, f64)], policy: &Policy) -> BuyScore {
    let mut components = Vec::new();
    if !signals.is_empty() && policy.patterns.signal_weight > 0.0 {
        components.push(ScoreComponent {
            label: "Signals".into(),
            weight: policy.patterns.signal_weight,
            strength: 1.0,
            contribution: policy.patterns.signal_weight,
            detail: signals.join("; "),
        });
    }
    for (hit, w) in hits {
        if *w <= 0.0 {
            continue;
        }
        components.push(ScoreComponent {
            label: hit.pattern.name().into(),
            weight: *w,
            strength: hit.strength,
            contribution: w * hit.strength,
            detail: hit.detail.clone(),
        });
    }
    let total = components.iter().map(|c| c.contribution).sum();
    BuyScore {
        components,
        total,
        threshold: policy.patterns.buy_score_threshold,
    }
}

/// Filters applied to every buy. Ok(description) when all pass.
fn buy_filters(row: &IndicatorRow, policy: &Policy, close: f64) -> Option<Vec<String>> {
    let sig = &policy.signals;
    let ind = &policy.indicators;
    let mut notes = Vec::new();
    if sig.require_rsi_not_overbought && ind.rsi_period.is_some() {
        if let Some(rsi) = row.rsi {
            if rsi >= ind.rsi_overbought {
                return None;
            }
            notes.push(format!("RSI {rsi:.1} < {}", ind.rsi_overbought));
        }
    }
    if sig.require_adx_trend && ind.adx_period.is_some() {
        if let Some(min) = ind.adx_min {
            match row.adx {
                Some(adx) if adx >= min => notes.push(format!("ADX {adx:.1} ≥ {min}")),
                _ => return None,
            }
        }
    }
    // Long-term SMA trend filter: only buy when close > SMA(sma_trend).
    // Uses signal-bar close (known at decision time). If SMA not yet ready, skip buy.
    if sig.require_price_above_sma_trend {
        match row.sma_trend {
            Some(sma) if close > sma => notes.push(format!(
                "close {close:.2} > SMA{} {sma:.2}",
                ind.sma_trend.unwrap_or(0)
            )),
            _ => return None,
        }
    }
    Some(notes)
}

/// Buy decision on candle i: Some((short reason, explanation)) when a buy is armed.
pub fn buy_decision(bars: &[Bar], i: usize, row: &IndicatorRow, policy: &Policy) -> Option<(String, String)> {
    let bar = &bars[i];
    let signals = indicator_signals(bar, row, policy);
    let hits = pattern_hits(bars, i, policy);
    let sc = score(&signals, &hits, policy);
    if !sc.passes() {
        return None;
    }
    let filters = buy_filters(row, policy, bar.close)?;
    let mut text = sc.breakdown();
    text.push_str(". ");
    for c in &sc.components {
        text.push_str(&c.detail);
        text.push_str(". ");
    }
    if !filters.is_empty() {
        text.push_str(&format!("Filters passed: {}. ", filters.join("; ")));
    }
    Some((sc.short_reason(), text))
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
    let mut entry_reason = String::new();
    let mut entry_explanation = String::new();
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
                    entry_reason = p.reason.clone();
                    entry_explanation = format!(
                        "{}Filled at the next candle's open {:.2} (candle dated {}).",
                        p.explanation, o, bar.date
                    );
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
                        entry_reason: entry_reason.clone(),
                        entry_explanation: entry_explanation.clone(),
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
                    explanation: String::new(),
                });
            } else if tp_hit {
                pending = Some(Pending {
                    action: "sell",
                    reason: "take_profit".into(),
                    explanation: String::new(),
                });
            }
        }

        if pending.is_none() {
            if shares > 0 && want_sell_signal(row, policy) {
                pending = Some(Pending {
                    action: "sell",
                    reason: "signal".into(),
                    explanation: String::new(),
                });
            } else if shares == 0 && i + 1 < n {
                if let Some((reason, explanation)) = buy_decision(&bars, i, row, policy) {
                    pending = Some(Pending {
                        action: "buy",
                        reason,
                        explanation,
                    });
                }
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
                entry_reason: entry_reason.clone(),
                entry_explanation: entry_explanation.clone(),
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

#[cfg(test)]
mod scoring_tests {
    use super::*;
    use chrono::{Duration, NaiveDate};

    fn mk(rows: &[(f64, f64, f64, f64)]) -> Vec<Bar> {
        let d0 = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        rows.iter()
            .enumerate()
            .map(|(k, &(o, h, l, c))| Bar { date: d0 + Duration::days(k as i64), open: o, high: h, low: l, close: c, volume: 1.0 })
            .collect()
    }

    /// Six falling candles then a hammer (strength ~0.99).
    fn hammer_series() -> Vec<Bar> {
        let mut rows: Vec<(f64, f64, f64, f64)> =
            (0..6).map(|k| { let c = 110.0 - 2.0 * k as f64; (c + 1.0, c + 1.5, c - 0.5, c) }).collect();
        rows.push((99.0, 100.2, 95.0, 100.0));
        mk(&rows)
    }

    fn policy() -> Policy {
        let mut p = crate::policy::parse_policy_text("{}").unwrap();
        p.signals.require_rsi_not_overbought = false;
        p.patterns.hammer = true;
        p
    }

    #[test]
    fn signal_only_scores_signal_weight() {
        let p = policy();
        let sc = score(&["EMA cross".into()], &[], &p);
        assert_eq!(sc.total, 1.0);
        assert!(sc.passes());
        assert_eq!(sc.short_reason(), "Signals");
    }

    #[test]
    fn nothing_present_never_buys() {
        let p = policy();
        let sc = score(&[], &[], &p);
        assert_eq!(sc.total, 0.0);
        assert!(!sc.passes());
    }

    #[test]
    fn pattern_contribution_is_weight_times_strength() {
        let p = policy();
        let bars = hammer_series();
        let hits = pattern_hits(&bars, 6, &p);
        assert_eq!(hits.len(), 1);
        let strength = hits[0].0.strength;
        let sc = score(&[], &hits, &p);
        assert!((sc.total - 0.6 * strength).abs() < 1e-12);
        assert!(sc.passes()); // ~0.59 ≥ 0.5
        assert!(sc.breakdown().contains("Hammer 0.60×"));
        assert!(sc.breakdown().contains("threshold 0.50"));
    }

    #[test]
    fn weak_pattern_alone_below_threshold_but_adds_to_signal() {
        let mut p = policy();
        p.patterns.hammer_weight = 0.3;
        p.patterns.signal_weight = 0.4;
        p.patterns.buy_score_threshold = 0.6;
        let bars = hammer_series();
        let hits = pattern_hits(&bars, 6, &p);
        assert!(!score(&[], &hits, &p).passes()); // ~0.30
        assert!(!score(&["EMA".into()], &[], &p).passes()); // 0.40
        let both = score(&["EMA".into()], &hits, &p); // ~0.70
        assert!(both.passes());
        assert_eq!(both.components.len(), 2);
        assert_eq!(both.short_reason(), "Signals + Hammer");
    }

    #[test]
    fn disabled_or_zero_weight_patterns_do_not_count() {
        let mut p = policy();
        p.patterns.hammer = false;
        assert!(pattern_hits(&hammer_series(), 6, &p).is_empty());
        p.patterns.hammer = true;
        p.patterns.hammer_weight = 0.0;
        let hits = pattern_hits(&hammer_series(), 6, &p);
        assert!(score(&[], &hits, &p).components.is_empty());
    }

    #[test]
    fn scoring_is_deterministic() {
        let p = policy();
        let bars = hammer_series();
        let a = score(&[], &pattern_hits(&bars, 6, &p), &p);
        let b = score(&[], &pattern_hits(&bars, 6, &p), &p);
        assert_eq!(a, b);
    }

    #[test]
    fn backtest_buy_explanation_contains_breakdown() {
        let mut p = policy();
        p.indicators.ema_fast = None;
        p.indicators.ema_slow = None;
        p.signals.buy_on_adx_confirm = false;
        p.risk.atr_stop_mult = None;
        p.risk.trailing_atr_mult = None;
        let mut bars = hammer_series();
        let d = bars.last().unwrap().date;
        bars.push(Bar { date: d + Duration::days(1), open: 100.5, high: 102.0, low: 100.0, close: 101.0, volume: 1.0 });
        let r = run_backtest(bars, &p);
        assert_eq!(r.trades.len(), 1);
        let t = &r.trades[0];
        assert_eq!(t.entry_reason, "Hammer");
        assert!(t.entry_explanation.contains("Hammer on 2024-03-07"), "{}", t.entry_explanation);
        assert!(t.entry_explanation.contains("Buy score"));
        assert!(t.entry_explanation.contains("Filled at the next candle's open 100.50 (candle dated 2024-03-08)"));
    }
}
