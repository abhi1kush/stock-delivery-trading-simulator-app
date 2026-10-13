//! Candle period parsing and OHLCV aggregation.
//!
//! The input CSV is expected to be daily (one row per trading session). Before
//! indicators, signals and patterns are computed, rows are grouped into candles
//! of the chosen period:
//!
//! | Period | Grouping |
//! | --- | --- |
//! | `1D` | unchanged (one candle per input row) |
//! | `ND` (N = 2..=60) | consecutive blocks of N input rows (trading sessions), starting at the first row; the last block may be shorter |
//! | `1W` | ISO calendar week (Mon–Sun) |
//! | `1M` | calendar month |
//!
//! Each candle: open = first open, high = max high, low = min low,
//! close = last close, volume = sum. The candle is dated by its **last**
//! session, because that is when its close is known; a signal on that candle
//! fills at the next candle's open (the first session of the next period).
use crate::data::Bar;
use crate::error::{Result, SimError};
use chrono::Datelike;
use serde::{Deserialize, Serialize};

/// Maximum N accepted for `ND` periods.
pub const MAX_DAY_MULTIPLE: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandlePeriod {
    Days(usize),
    Week,
    Month,
}

impl CandlePeriod {
    pub fn label(&self) -> String {
        match self {
            CandlePeriod::Days(n) => format!("{n}D"),
            CandlePeriod::Week => "1W".into(),
            CandlePeriod::Month => "1M".into(),
        }
    }
}

/// Parse `1D`, `5D`, `1W`, `1M` (case-insensitive, whitespace ignored).
pub fn parse_period(text: &str) -> Result<CandlePeriod> {
    let t: String = text.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_ascii_uppercase();
    let bad = || {
        SimError::Validation(format!(
            "invalid candle period '{text}': use ND (N = 1..={MAX_DAY_MULTIPLE}), 1W or 1M"
        ))
    };
    if t.len() < 2 {
        return Err(bad());
    }
    let (num, unit) = t.split_at(t.len() - 1);
    let n: usize = num.parse().map_err(|_| bad())?;
    match unit {
        "D" if (1..=MAX_DAY_MULTIPLE).contains(&n) => Ok(CandlePeriod::Days(n)),
        "W" if n == 1 => Ok(CandlePeriod::Week),
        "M" if n == 1 => Ok(CandlePeriod::Month),
        _ => Err(bad()),
    }
}

/// Median calendar-day gap between consecutive input rows (None for < 2 rows).
pub fn median_gap_days(bars: &[Bar]) -> Option<i64> {
    let mut gaps: Vec<i64> = bars.windows(2).map(|w| (w[1].date - w[0].date).num_days()).collect();
    if gaps.is_empty() {
        return None;
    }
    gaps.sort_unstable();
    Some(gaps[gaps.len() / 2])
}

/// Group sorted bars into candles of `period`. Input must be sorted ascending.
pub fn aggregate(bars: &[Bar], period: CandlePeriod) -> Vec<Bar> {
    if bars.is_empty() {
        return Vec::new();
    }
    if let CandlePeriod::Days(1) = period {
        return bars.to_vec();
    }
    let key = |i: usize, b: &Bar| -> (i32, u32) {
        match period {
            CandlePeriod::Days(n) => ((i / n) as i32, 0),
            CandlePeriod::Week => {
                let w = b.date.iso_week();
                (w.year(), w.week())
            }
            CandlePeriod::Month => (b.date.year(), b.date.month()),
        }
    };
    let mut out: Vec<Bar> = Vec::new();
    let mut cur_key = key(0, &bars[0]);
    let mut cur = bars[0].clone();
    for (i, b) in bars.iter().enumerate().skip(1) {
        let k = key(i, b);
        if k == cur_key {
            cur.high = cur.high.max(b.high);
            cur.low = cur.low.min(b.low);
            cur.close = b.close;
            cur.volume += b.volume;
            cur.date = b.date;
        } else {
            out.push(cur);
            cur = b.clone();
            cur_key = k;
        }
    }
    out.push(cur);
    out
}

/// Validate the period against the input granularity and aggregate.
///
/// Multiples of a day / week / month need daily input: if the median gap
/// between rows is more than 4 calendar days (i.e. the file is already weekly
/// or coarser) only `1D` (no aggregation) is accepted.
pub fn apply_period(bars: Vec<Bar>, period: CandlePeriod) -> Result<Vec<Bar>> {
    if period == CandlePeriod::Days(1) {
        return Ok(bars);
    }
    if let Some(gap) = median_gap_days(&bars) {
        if gap > 4 {
            return Err(SimError::Validation(format!(
                "candle period {} needs daily input, but rows are about {gap} days apart; use 1D",
                period.label()
            )));
        }
    }
    let out = aggregate(&bars, period);
    if out.len() < 2 {
        return Err(SimError::Validation(format!(
            "candle period {} leaves only {} candle(s); use a shorter period or more data",
            period.label(),
            out.len()
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn bar(d: &str, o: f64, h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { date: NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap(), open: o, high: h, low: l, close: c, volume: v }
    }

    fn week_of_days() -> Vec<Bar> {
        // Mon 2024-03-04 .. Fri 2024-03-08, then Mon 2024-03-11 .. Tue 2024-03-12
        vec![
            bar("2024-03-04", 10.0, 12.0, 9.0, 11.0, 100.0),
            bar("2024-03-05", 11.0, 13.0, 10.0, 12.0, 200.0),
            bar("2024-03-06", 12.0, 12.5, 8.0, 9.0, 300.0),
            bar("2024-03-07", 9.0, 10.0, 8.5, 9.5, 400.0),
            bar("2024-03-08", 9.5, 11.0, 9.0, 10.5, 500.0),
            bar("2024-03-11", 10.5, 14.0, 10.0, 13.0, 600.0),
            bar("2024-03-12", 13.0, 13.5, 12.0, 12.5, 700.0),
        ]
    }

    #[test]
    fn parses_periods() {
        assert_eq!(parse_period("1D").unwrap(), CandlePeriod::Days(1));
        assert_eq!(parse_period(" 5d ").unwrap(), CandlePeriod::Days(5));
        assert_eq!(parse_period("1w").unwrap(), CandlePeriod::Week);
        assert_eq!(parse_period("1M").unwrap(), CandlePeriod::Month);
        for bad in ["", "D", "0D", "61D", "2W", "3M", "1Y", "-1D", "1.5D", "abc"] {
            assert!(parse_period(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn one_day_is_identity() {
        let b = week_of_days();
        let out = aggregate(&b, CandlePeriod::Days(1));
        assert_eq!(out.len(), 7);
        assert_eq!(out[3].close, 9.5);
    }

    #[test]
    fn n_day_blocks() {
        let out = aggregate(&week_of_days(), CandlePeriod::Days(3));
        assert_eq!(out.len(), 3);
        let a = &out[0];
        assert_eq!((a.open, a.high, a.low, a.close, a.volume), (10.0, 13.0, 8.0, 9.0, 600.0));
        assert_eq!(a.date.to_string(), "2024-03-06");
        let b = &out[1];
        assert_eq!((b.open, b.high, b.low, b.close, b.volume), (9.0, 14.0, 8.5, 13.0, 1500.0));
        let c = &out[2]; // short trailing block
        assert_eq!((c.open, c.high, c.low, c.close, c.volume), (13.0, 13.5, 12.0, 12.5, 700.0));
    }

    #[test]
    fn weekly_candles() {
        let out = aggregate(&week_of_days(), CandlePeriod::Week);
        assert_eq!(out.len(), 2);
        let w = &out[0];
        assert_eq!((w.open, w.high, w.low, w.close, w.volume), (10.0, 13.0, 8.0, 10.5, 1500.0));
        assert_eq!(w.date.to_string(), "2024-03-08");
        assert_eq!(out[1].date.to_string(), "2024-03-12");
        assert_eq!(out[1].volume, 1300.0);
    }

    #[test]
    fn monthly_candles_across_year() {
        let b = vec![
            bar("2023-12-28", 5.0, 6.0, 4.0, 5.5, 1.0),
            bar("2023-12-29", 5.5, 7.0, 5.0, 6.5, 2.0),
            bar("2024-01-02", 6.5, 6.6, 3.0, 4.0, 3.0),
            bar("2024-01-31", 4.0, 8.0, 3.5, 7.0, 4.0),
            bar("2024-02-01", 7.0, 7.5, 6.0, 7.2, 5.0),
        ];
        let out = aggregate(&b, CandlePeriod::Month);
        assert_eq!(out.len(), 3);
        assert_eq!((out[0].open, out[0].high, out[0].low, out[0].close, out[0].volume), (5.0, 7.0, 4.0, 6.5, 3.0));
        assert_eq!((out[1].open, out[1].high, out[1].low, out[1].close, out[1].volume), (6.5, 8.0, 3.0, 7.0, 7.0));
        assert_eq!(out[2].date.to_string(), "2024-02-01");
    }

    #[test]
    fn rejects_aggregating_non_daily_input() {
        let weekly = vec![
            bar("2024-01-05", 1.0, 1.0, 1.0, 1.0, 1.0),
            bar("2024-01-12", 1.0, 1.0, 1.0, 1.0, 1.0),
            bar("2024-01-19", 1.0, 1.0, 1.0, 1.0, 1.0),
        ];
        assert!(apply_period(weekly.clone(), CandlePeriod::Week).is_err());
        assert!(apply_period(weekly, CandlePeriod::Days(1)).is_ok());
    }

    #[test]
    fn rejects_too_few_candles() {
        assert!(apply_period(week_of_days(), CandlePeriod::Month).is_err());
        assert!(apply_period(week_of_days(), CandlePeriod::Days(7)).is_err());
        assert_eq!(apply_period(week_of_days(), CandlePeriod::Days(6)).unwrap().len(), 2);
    }
}
