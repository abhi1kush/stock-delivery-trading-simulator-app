//! Bullish candlestick pattern recognition.
//!
//! All checks are causal: a pattern on candle `i` reads candles `i-L-1..=i`
//! only (L = downtrend lookback). Each detector returns a [`PatternHit`] with a
//! **strength** in `[0.5, 1.0]`: 0.5 when the candle just clears the
//! thresholds, rising toward 1.0 the more cleanly it fits. Strength is
//! multiplied by the user weight to get the pattern's buy-score contribution.
//!
//! Thresholds (body = |close-open|, range = high-low,
//! upper = high-max(open,close), lower = min(open,close)-low):
//!
//! * **Downtrend** (hammer, doji): close of the candle *before* the pattern is
//!   below the close L candles before that (default L = 5).
//!   decline% = 1 - close[i-1]/close[i-1-L].
//! * **Hammer**: range > 0, body <= 35% of range, lower >= 2 x body,
//!   upper <= 10% of range, lower >= 60% of range, after a downtrend.
//!   Strength = 0.5 + 0.25*clamp((lower/body - 2)/2) + 0.25*clamp(decline%/5%).
//! * **Three white soldiers**: candles i-2, i-1, i are all bullish
//!   (close > open), each body >= 50% of its range, each of the last two opens
//!   within the prior body (prior open..=prior close) and closes above the
//!   prior close, every upper wick <= 30% of its body.
//!   Strength = 0.5 + 0.5*(1 - max(upper/body)/0.30).
//! * **Bullish engulfing**: candle i-1 bearish (close < open), candle i bullish,
//!   open_i <= close_{i-1}, close_i >= open_{i-1}, and body_i > body_{i-1}.
//!   Strength = 0.5 + 0.5*clamp(body_i/body_{i-1} - 1).
//! * **Doji**: range > 0, body <= 10% of range, after a downtrend (a doji is
//!   indecision; it is only a buy hint once sellers have been in control).
//!   Strength = 0.5 + 0.25*(1 - (body/range)/0.10) + 0.25*clamp(decline%/5%).
use crate::data::Bar;

pub const HAMMER_MAX_BODY_FRAC: f64 = 0.35;
pub const HAMMER_MIN_LOWER_X_BODY: f64 = 2.0;
pub const HAMMER_MAX_UPPER_FRAC: f64 = 0.10;
pub const HAMMER_MIN_LOWER_FRAC: f64 = 0.60;
pub const TWS_MIN_BODY_FRAC: f64 = 0.50;
pub const TWS_MAX_UPPER_X_BODY: f64 = 0.30;
pub const DOJI_MAX_BODY_FRAC: f64 = 0.10;
/// Decline (fraction) at which the downtrend part of strength saturates.
pub const DECLINE_FULL: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    Hammer,
    ThreeWhiteSoldiers,
    BullishEngulfing,
    Doji,
}

impl Pattern {
    /// Short chart label.
    pub fn code(&self) -> &'static str {
        match self {
            Pattern::Hammer => "H",
            Pattern::ThreeWhiteSoldiers => "3WS",
            Pattern::BullishEngulfing => "BE",
            Pattern::Doji => "D",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Pattern::Hammer => "Hammer",
            Pattern::ThreeWhiteSoldiers => "Three white soldiers",
            Pattern::BullishEngulfing => "Bullish engulfing",
            Pattern::Doji => "Doji",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternHit {
    pub pattern: Pattern,
    /// 0.5..=1.0
    pub strength: f64,
    /// Human description with dates and key numbers.
    pub detail: String,
}

fn clamp01(x: f64) -> f64 {
    if x.is_nan() {
        0.0
    } else {
        x.clamp(0.0, 1.0)
    }
}
fn body(b: &Bar) -> f64 {
    (b.close - b.open).abs()
}
fn range(b: &Bar) -> f64 {
    b.high - b.low
}
fn upper(b: &Bar) -> f64 {
    b.high - b.open.max(b.close)
}
fn lower(b: &Bar) -> f64 {
    b.open.min(b.close) - b.low
}

/// Decline fraction before candle `i` if in a downtrend, else None.
pub fn prior_decline(bars: &[Bar], i: usize, lookback: usize) -> Option<f64> {
    if lookback == 0 || i < lookback + 1 {
        return None;
    }
    let end = bars[i - 1].close;
    let start = bars[i - 1 - lookback].close;
    if start > 0.0 && end < start {
        Some(1.0 - end / start)
    } else {
        None
    }
}

pub fn hammer(bars: &[Bar], i: usize, lookback: usize) -> Option<PatternHit> {
    let b = &bars[i];
    let r = range(b);
    if r <= 0.0 {
        return None;
    }
    let (bd, lo, up) = (body(b), lower(b), upper(b));
    if bd > HAMMER_MAX_BODY_FRAC * r
        || lo < HAMMER_MIN_LOWER_X_BODY * bd
        || up > HAMMER_MAX_UPPER_FRAC * r
        || lo < HAMMER_MIN_LOWER_FRAC * r
    {
        return None;
    }
    let decline = prior_decline(bars, i, lookback)?;
    let ratio = if bd > 0.0 { lo / bd } else { f64::INFINITY };
    let strength = 0.5 + 0.25 * clamp01((ratio - 2.0) / 2.0) + 0.25 * clamp01(decline / DECLINE_FULL);
    let ratio_txt = if ratio.is_finite() { format!("{ratio:.1}x body") } else { "no body".into() };
    Some(PatternHit {
        pattern: Pattern::Hammer,
        strength,
        detail: format!(
            "Hammer on {}: lower shadow {ratio_txt}, upper shadow {:.0}% of range, after {lookback}-candle decline of {:.1}%; close {:.2}",
            b.date,
            up / r * 100.0,
            decline * 100.0,
            b.close
        ),
    })
}

pub fn three_white_soldiers(bars: &[Bar], i: usize) -> Option<PatternHit> {
    if i < 2 {
        return None;
    }
    let c = &bars[i - 2..=i];
    let mut worst_wick = 0.0_f64;
    for (k, b) in c.iter().enumerate() {
        let bd = body(b);
        let r = range(b);
        if b.close <= b.open || r <= 0.0 || bd < TWS_MIN_BODY_FRAC * r || upper(b) > TWS_MAX_UPPER_X_BODY * bd {
            return None;
        }
        worst_wick = worst_wick.max(upper(b) / bd);
        if k > 0 {
            let p = &c[k - 1];
            if b.open < p.open || b.open > p.close || b.close <= p.close {
                return None;
            }
        }
    }
    let strength = 0.5 + 0.5 * clamp01(1.0 - worst_wick / TWS_MAX_UPPER_X_BODY);
    Some(PatternHit {
        pattern: Pattern::ThreeWhiteSoldiers,
        strength,
        detail: format!(
            "Three white soldiers {} → {}: closes {:.2} → {:.2} → {:.2}, largest upper wick {:.0}% of body",
            c[0].date,
            c[2].date,
            c[0].close,
            c[1].close,
            c[2].close,
            worst_wick * 100.0
        ),
    })
}

pub fn bullish_engulfing(bars: &[Bar], i: usize) -> Option<PatternHit> {
    if i < 1 {
        return None;
    }
    let (p, b) = (&bars[i - 1], &bars[i]);
    if !(p.close < p.open && b.close > b.open && b.open <= p.close && b.close >= p.open) {
        return None;
    }
    let (pb, bb) = (body(p), body(b));
    if bb <= pb {
        return None;
    }
    let ratio = bb / pb;
    let strength = 0.5 + 0.5 * clamp01(ratio - 1.0);
    Some(PatternHit {
        pattern: Pattern::BullishEngulfing,
        strength,
        detail: format!(
            "Bullish engulfing on {}: body {:.2} ({:.2}→{:.2}) engulfs {}'s bearish body {:.2} ({:.1}x); close {:.2}",
            b.date, bb, b.open, b.close, p.date, pb, ratio, b.close
        ),
    })
}

pub fn doji(bars: &[Bar], i: usize, lookback: usize) -> Option<PatternHit> {
    let b = &bars[i];
    let r = range(b);
    if r <= 0.0 {
        return None;
    }
    let frac = body(b) / r;
    if frac > DOJI_MAX_BODY_FRAC {
        return None;
    }
    let decline = prior_decline(bars, i, lookback)?;
    let strength = 0.5 + 0.25 * clamp01(1.0 - frac / DOJI_MAX_BODY_FRAC) + 0.25 * clamp01(decline / DECLINE_FULL);
    Some(PatternHit {
        pattern: Pattern::Doji,
        strength,
        detail: format!(
            "Doji on {}: body {:.1}% of range, after {lookback}-candle decline of {:.1}% (indecision, used only after a downtrend); close {:.2}",
            b.date,
            frac * 100.0,
            decline * 100.0,
            b.close
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate};

    /// Build candles from (o,h,l,c), dated consecutively from 2024-03-01.
    fn mk(rows: &[(f64, f64, f64, f64)]) -> Vec<Bar> {
        let d0 = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        rows.iter()
            .enumerate()
            .map(|(k, &(o, h, l, c))| Bar { date: d0 + Duration::days(k as i64), open: o, high: h, low: l, close: c, volume: 1.0 })
            .collect()
    }

    /// Six falling candles (closes 110,108,...,100) then `last`.
    fn after_decline(last: (f64, f64, f64, f64)) -> Vec<Bar> {
        let mut rows: Vec<(f64, f64, f64, f64)> =
            (0..6).map(|k| { let c = 110.0 - 2.0 * k as f64; (c + 1.0, c + 1.5, c - 0.5, c) }).collect();
        rows.push(last);
        mk(&rows)
    }

    /// Six rising candles then `last`.
    fn after_rise(last: (f64, f64, f64, f64)) -> Vec<Bar> {
        let mut rows: Vec<(f64, f64, f64, f64)> =
            (0..6).map(|k| { let c = 100.0 + 2.0 * k as f64; (c - 1.0, c + 0.5, c - 1.5, c) }).collect();
        rows.push(last);
        mk(&rows)
    }

    #[test]
    fn hammer_positive() {
        // body 1, lower 4, upper 0.2, range 5.2
        let b = after_decline((99.0, 100.2, 95.0, 100.0));
        let h = hammer(&b, 6, 5).expect("hammer");
        assert!(h.strength > 0.9 && h.strength <= 1.0, "{}", h.strength);
        assert!(h.detail.contains("Hammer on 2024-03-07"));
        assert!(h.detail.contains("lower shadow 4.0x body"));
        assert!(h.detail.contains("5-candle decline"));
    }

    #[test]
    fn hammer_negatives() {
        // shape ok but no downtrend
        assert!(hammer(&after_rise((99.0, 100.2, 95.0, 100.0)), 6, 5).is_none());
        // lower shadow only 1.5x body
        assert!(hammer(&after_decline((98.0, 100.1, 95.0, 100.0)), 6, 5).is_none());
        // big upper shadow (inverted hammer / spinning top)
        assert!(hammer(&after_decline((99.0, 102.0, 95.0, 100.0)), 6, 5).is_none());
        // not enough history for the downtrend check
        assert!(hammer(&mk(&[(99.0, 100.2, 95.0, 100.0)]), 0, 5).is_none());
    }

    #[test]
    fn hammer_strength_grows_with_shadow() {
        let weak = hammer(&after_decline((99.0, 100.1, 96.5, 100.0)), 6, 5).unwrap(); // lower 2.5x body
        let strong = hammer(&after_decline((99.5, 100.1, 95.0, 100.0)), 6, 5).unwrap();
        assert!(strong.strength > weak.strength);
    }

    #[test]
    fn three_white_soldiers_positive() {
        let b = mk(&[(100.0, 104.2, 99.8, 104.0), (102.0, 108.3, 101.5, 108.0), (106.0, 112.4, 105.5, 112.0)]);
        let h = three_white_soldiers(&b, 2).expect("3ws");
        assert!(h.strength > 0.5);
        assert!(h.detail.contains("2024-03-01 → 2024-03-03"));
    }

    #[test]
    fn three_white_soldiers_negatives() {
        // middle candle bearish
        assert!(three_white_soldiers(&mk(&[(100.0, 104.2, 99.8, 104.0), (108.0, 108.3, 101.5, 102.0), (102.0, 112.4, 101.5, 112.0)]), 2).is_none());
        // third opens above prior body (gap up)
        assert!(three_white_soldiers(&mk(&[(100.0, 104.2, 99.8, 104.0), (102.0, 108.3, 101.5, 108.0), (109.0, 113.4, 108.5, 113.0)]), 2).is_none());
        // long upper wick on the last candle
        assert!(three_white_soldiers(&mk(&[(100.0, 104.2, 99.8, 104.0), (102.0, 108.3, 101.5, 108.0), (106.0, 115.0, 105.5, 110.0)]), 2).is_none());
        // third does not close higher
        assert!(three_white_soldiers(&mk(&[(100.0, 104.2, 99.8, 104.0), (102.0, 108.3, 101.5, 108.0), (104.0, 107.9, 103.8, 107.5)]), 2).is_none());
        assert!(three_white_soldiers(&mk(&[(100.0, 104.2, 99.8, 104.0)]), 0).is_none());
    }

    #[test]
    fn bullish_engulfing_positive() {
        let b = mk(&[(105.0, 105.5, 101.5, 102.0), (101.5, 107.0, 101.0, 106.5)]);
        let h = bullish_engulfing(&b, 1).expect("engulfing");
        // body 5.0 vs 3.0 → 1.67x → strength 0.5 + 0.5 * 0.667
        assert!((h.strength - (0.5 + 0.5 * (5.0 / 3.0 - 1.0))).abs() < 1e-9);
        assert!(h.detail.contains("1.7x"));
    }

    #[test]
    fn bullish_engulfing_negatives() {
        // prior candle bullish
        assert!(bullish_engulfing(&mk(&[(102.0, 105.5, 101.5, 105.0), (101.5, 107.0, 101.0, 106.5)]), 1).is_none());
        // current does not close above prior open
        assert!(bullish_engulfing(&mk(&[(105.0, 105.5, 101.5, 102.0), (101.5, 104.8, 101.0, 104.5)]), 1).is_none());
        // current opens above prior close
        assert!(bullish_engulfing(&mk(&[(105.0, 105.5, 101.5, 102.0), (103.0, 107.0, 102.5, 106.5)]), 1).is_none());
        // current bearish
        assert!(bullish_engulfing(&mk(&[(105.0, 105.5, 101.5, 102.0), (106.5, 107.0, 101.0, 101.5)]), 1).is_none());
    }

    #[test]
    fn doji_positive_after_downtrend() {
        let h = doji(&after_decline((100.0, 102.0, 98.0, 100.2)), 6, 5).expect("doji");
        assert!(h.detail.contains("body 5.0% of range"));
        assert!(h.detail.contains("indecision"));
        assert!(h.strength > 0.5 && h.strength <= 1.0);
    }

    #[test]
    fn doji_negatives() {
        // same doji, but after a rise → not a buy trigger
        assert!(doji(&after_rise((100.0, 102.0, 98.0, 100.2)), 6, 5).is_none());
        // body 25% of range
        assert!(doji(&after_decline((100.0, 102.0, 98.0, 101.0)), 6, 5).is_none());
        // zero range
        assert!(doji(&after_decline((100.0, 100.0, 100.0, 100.0)), 6, 5).is_none());
    }
}
