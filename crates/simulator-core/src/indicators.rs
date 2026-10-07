use crate::data::Bar;
use crate::policy::Policy;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IndicatorRow {
    pub sma_fast: Option<f64>,
    pub sma_slow: Option<f64>,
    pub sma_trend: Option<f64>,
    pub ema_fast: Option<f64>,
    pub ema_slow: Option<f64>,
    pub rsi: Option<f64>,
    pub atr: Option<f64>,
    pub adx: Option<f64>,
    pub sma_cross_up: bool,
    pub sma_cross_down: bool,
    pub ema_cross_up: bool,
    pub ema_cross_down: bool,
    pub rsi_exit_oversold: bool,
    pub rsi_exit_overbought: bool,
    pub adx_cross_up: bool,
}

fn sma(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let mut sum = 0.0;
    for i in 0..n {
        sum += closes[i];
        if i >= period {
            sum -= closes[i - period];
        }
        if i + 1 >= period {
            out[i] = Some(sum / period as f64);
        }
    }
    out
}

fn ema(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    // Seed with SMA of first `period` closes
    let seed: f64 = closes[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = Some(seed);
    let mut prev = seed;
    for i in period..n {
        let v = alpha * closes[i] + (1.0 - alpha) * prev;
        out[i] = Some(v);
        prev = v;
    }
    out
}

fn rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period + 1 {
        return out;
    }
    let alpha = 1.0 / period as f64;
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for i in 1..=period {
        let d = closes[i] - closes[i - 1];
        if d >= 0.0 {
            avg_gain += d;
        } else {
            avg_loss += -d;
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;
    let rs = if avg_loss == 0.0 {
        if avg_gain > 0.0 {
            f64::INFINITY
        } else {
            1.0
        }
    } else {
        avg_gain / avg_loss
    };
    out[period] = Some(if rs.is_infinite() {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + rs)
    });

    for i in (period + 1)..n {
        let d = closes[i] - closes[i - 1];
        let gain = if d > 0.0 { d } else { 0.0 };
        let loss = if d < 0.0 { -d } else { 0.0 };
        avg_gain = alpha * gain + (1.0 - alpha) * avg_gain;
        avg_loss = alpha * loss + (1.0 - alpha) * avg_loss;
        let val = if avg_loss == 0.0 {
            if avg_gain > 0.0 {
                100.0
            } else {
                50.0
            }
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - 100.0 / (1.0 + rs)
        };
        out[i] = Some(val);
    }
    out
}

fn true_range(bars: &[Bar]) -> Vec<f64> {
    let mut tr = Vec::with_capacity(bars.len());
    for (i, b) in bars.iter().enumerate() {
        if i == 0 {
            tr.push(b.high - b.low);
        } else {
            let prev = bars[i - 1].close;
            let a = b.high - b.low;
            let c = (b.high - prev).abs();
            let d = (b.low - prev).abs();
            tr.push(a.max(c).max(d));
        }
    }
    tr
}

fn wilder_smooth(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let alpha = 1.0 / period as f64;
    let seed: f64 = values[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = Some(seed);
    let mut prev = seed;
    for i in period..n {
        let v = alpha * values[i] + (1.0 - alpha) * prev;
        out[i] = Some(v);
        prev = v;
    }
    out
}

fn atr(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    wilder_smooth(&true_range(bars), period)
}

fn adx(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    if period == 0 || n < period + 1 {
        return vec![None; n];
    }
    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];
    for i in 1..n {
        let up = bars[i].high - bars[i - 1].high;
        let down = bars[i - 1].low - bars[i].low;
        if up > down && up > 0.0 {
            plus_dm[i] = up;
        }
        if down > up && down > 0.0 {
            minus_dm[i] = down;
        }
    }
    let atr_s = atr(bars, period);
    let plus_s = wilder_smooth(&plus_dm, period);
    let minus_s = wilder_smooth(&minus_dm, period);
    let mut dx = vec![0.0; n];
    for i in 0..n {
        if let (Some(a), Some(p), Some(m)) = (atr_s[i], plus_s[i], minus_s[i]) {
            if a > 0.0 {
                let pdi = 100.0 * p / a;
                let mdi = 100.0 * m / a;
                let sum = pdi + mdi;
                dx[i] = if sum > 0.0 {
                    100.0 * (pdi - mdi).abs() / sum
                } else {
                    0.0
                };
            }
        }
    }
    // ADX is Wilder smooth of DX; first valid around 2*period-1
    // Use wilder_smooth but only where dx has been computed (from period-1)
    wilder_smooth(&dx, period)
}

fn cross_up(fast: &[Option<f64>], slow: &[Option<f64>]) -> Vec<bool> {
    let n = fast.len();
    let mut out = vec![false; n];
    for i in 1..n {
        if let (Some(pf), Some(ps), Some(cf), Some(cs)) =
            (fast[i - 1], slow[i - 1], fast[i], slow[i])
        {
            out[i] = pf <= ps && cf > cs;
        }
    }
    out
}

fn cross_down(fast: &[Option<f64>], slow: &[Option<f64>]) -> Vec<bool> {
    let n = fast.len();
    let mut out = vec![false; n];
    for i in 1..n {
        if let (Some(pf), Some(ps), Some(cf), Some(cs)) =
            (fast[i - 1], slow[i - 1], fast[i], slow[i])
        {
            out[i] = pf >= ps && cf < cs;
        }
    }
    out
}

/// Compute indicator rows aligned 1:1 with `bars`.
///
/// Causality: value at index `i` depends only on `bars[0..=i]` (SMA windows,
/// recursive EMA/RSI/ATR/ADX, and cross flags vs `i-1`). See docs/CAUSALITY.md.
pub fn compute(bars: &[Bar], policy: &Policy) -> Vec<IndicatorRow> {
    let n = bars.len();
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let mut rows = vec![IndicatorRow::default(); n];

    let ind = &policy.indicators;

    let (sma_f, sma_s, sma_up, sma_dn) = if let (Some(f), Some(s)) = (ind.sma_fast, ind.sma_slow) {
        let a = sma(&closes, f);
        let b = sma(&closes, s);
        let up = cross_up(&a, &b);
        let dn = cross_down(&a, &b);
        (a, b, up, dn)
    } else {
        (vec![None; n], vec![None; n], vec![false; n], vec![false; n])
    };

    // Long-term SMA for optional price-above-trend filter (causal: window ends at i).
    let sma_t = if let Some(period) = ind.sma_trend {
        sma(&closes, period)
    } else {
        vec![None; n]
    };

    let (ema_f, ema_s, ema_up, ema_dn) = if let (Some(f), Some(s)) = (ind.ema_fast, ind.ema_slow) {
        let a = ema(&closes, f);
        let b = ema(&closes, s);
        let up = cross_up(&a, &b);
        let dn = cross_down(&a, &b);
        (a, b, up, dn)
    } else {
        (vec![None; n], vec![None; n], vec![false; n], vec![false; n])
    };

    let rsi_v = if let Some(p) = ind.rsi_period {
        rsi(&closes, p)
    } else {
        vec![None; n]
    };

    let atr_v = if let Some(p) = ind.atr_period {
        atr(bars, p)
    } else {
        vec![None; n]
    };

    let adx_v = if let Some(p) = ind.adx_period {
        adx(bars, p)
    } else {
        vec![None; n]
    };

    for i in 0..n {
        let mut rsi_exit_os = false;
        let mut rsi_exit_ob = false;
        let mut adx_cross_up = false;
        if i > 0 {
            if let (Some(prev), Some(cur)) = (rsi_v[i - 1], rsi_v[i]) {
                rsi_exit_os = prev <= ind.rsi_oversold && cur > ind.rsi_oversold;
                rsi_exit_ob = prev >= ind.rsi_overbought && cur < ind.rsi_overbought;
            }
            if let Some(min) = ind.adx_min {
                if let (Some(prev), Some(cur)) = (adx_v[i - 1], adx_v[i]) {
                    adx_cross_up = prev < min && cur >= min;
                }
            }
        }
        rows[i] = IndicatorRow {
            sma_fast: sma_f[i],
            sma_slow: sma_s[i],
            sma_trend: sma_t[i],
            ema_fast: ema_f[i],
            ema_slow: ema_s[i],
            rsi: rsi_v[i],
            atr: atr_v[i],
            adx: adx_v[i],
            sma_cross_up: sma_up[i],
            sma_cross_down: sma_dn[i],
            ema_cross_up: ema_up[i],
            ema_cross_down: ema_dn[i],
            rsi_exit_oversold: rsi_exit_os,
            rsi_exit_overbought: rsi_exit_ob,
            adx_cross_up,
        };
    }
    rows
}
