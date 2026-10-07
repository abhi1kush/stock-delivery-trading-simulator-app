use crate::error::{Result, SimError};
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Bar {
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Deserialize)]
struct CsvRow {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Load OHLCV bars from a CSV string.
///
/// Columns match the upstream engine: `date,open,high,low,close,volume`.
/// Dates must be `YYYY-MM-DD`. Bars are sorted ascending and rejected if a
/// date repeats or OHLC is inconsistent.
pub fn load_prices_csv(text: &str) -> Result<Vec<Bar>> {
    let text = text.trim_start_matches('\u{feff}');
    let mut rdr = csv::Reader::from_reader(text.as_bytes());
    let mut bars = Vec::new();
    for (i, rec) in rdr.deserialize().enumerate() {
        let row: CsvRow = rec.map_err(|e| SimError::Parse(format!("row {}: {e}", i + 2)))?;
        let date = NaiveDate::parse_from_str(row.date.trim(), "%Y-%m-%d").map_err(|e| {
            SimError::Parse(format!("invalid date '{}' on row {}: {e}", row.date, i + 2))
        })?;
        if row.high < row.open.max(row.close) {
            return Err(SimError::Validation(format!(
                "row {}: high < max(open, close)",
                i + 2
            )));
        }
        if row.low > row.open.min(row.close) {
            return Err(SimError::Validation(format!(
                "row {}: low > min(open, close)",
                i + 2
            )));
        }
        bars.push(Bar {
            date,
            open: row.open,
            high: row.high,
            low: row.low,
            close: row.close,
            volume: row.volume,
        });
    }
    if bars.is_empty() {
        return Err(SimError::Validation("price CSV is empty".into()));
    }
    bars.sort_by_key(|b| b.date);
    for w in bars.windows(2) {
        if w[0].date == w[1].date {
            return Err(SimError::Validation(format!(
                "duplicate date {}",
                w[0].date
            )));
        }
    }
    Ok(bars)
}
