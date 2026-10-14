//! The `prices` array the WASM returns is exactly the engine's aggregated candles.
use simulator_core::candles::{aggregate, parse_period};
use simulator_core::{execute, load_prices_csv};

const YAML: &str = include_str!("../../../policy/example_policy.yaml");
const SAMPLES: &[&str] = &[
    include_str!("../../../public/data/CUPID.csv"),
    include_str!("../../../public/data/TRENT.csv"),
    include_str!("../../../public/data/ADANIENT.csv"),
];

#[test]
fn exported_candles_match_aggregation() {
    for csv in SAMPLES {
        for period in ["1D", "3D", "1W", "1M"] {
            let out = execute(csv, &format!("{YAML}\ncandles:\n  period: {period}\n  scale_lengths: true\n")).unwrap();
            let expect = aggregate(&load_prices_csv(csv).unwrap(), parse_period(period).unwrap());
            assert_eq!(out.prices.len(), expect.len(), "{period}");
            for (p, b) in out.prices.iter().zip(&expect) {
                assert_eq!(p.date, b.date.to_string());
                assert_eq!((p.open, p.high, p.low, p.close, p.volume), (b.open, b.high, b.low, b.close, b.volume));
            }
            // Every fill date is a candle date, so chart markers line up with the trade table.
            let dates: std::collections::HashSet<_> = out.prices.iter().map(|p| p.date.clone()).collect();
            for t in &out.trades {
                assert!(dates.contains(&t.entry_date) && dates.contains(&t.exit_date));
                let c = out.prices.iter().find(|p| p.date == t.entry_date).unwrap();
                assert_eq!(c.open, t.entry_price, "buy fills at the candle open");
            }
        }
    }
}

#[test]
fn exported_pattern_labels() {
    let csv = SAMPLES[0];
    let off = execute(csv, YAML).unwrap();
    assert!(off.prices.iter().all(|p| p.patterns.is_empty()), "patterns off → no labels");
    let on = execute(csv, &format!("{YAML}\npatterns:\n  hammer: true\n  three_white_soldiers: true\n  bullish_engulfing: true\n  doji: true\n")).unwrap();
    let labels: Vec<&String> = on.prices.iter().flat_map(|p| p.patterns.iter()).collect();
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|l| ["H", "3WS", "BE", "D"].contains(&l.as_str())));
    // A pattern buy's explanation names a candle that carries that label.
    for t in on.trades.iter().filter(|t| t.entry_reason.contains("Bullish engulfing")) {
        let date = t.entry_explanation.split("Bullish engulfing on ").nth(1).unwrap().split(':').next().unwrap();
        let candle = on.prices.iter().find(|p| p.date == date).unwrap();
        assert!(candle.patterns.contains(&"BE".to_string()), "{date}");
    }
}
