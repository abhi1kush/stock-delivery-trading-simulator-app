//! Every bundled sample CSV parses and runs with the default example policy.
use simulator_core::execute;

const YAML: &str = include_str!("../../../policy/example_policy.yaml");

const SAMPLES: &[(&str, &str, usize)] = &[
    ("CUPID", include_str!("../../../public/data/CUPID.csv"), 246),
    ("PAYTM", include_str!("../../../public/data/PAYTM.csv"), 257),
    ("SUZLON", include_str!("../../../public/data/SUZLON.csv"), 496),
    ("TRENT", include_str!("../../../public/data/TRENT.csv"), 365),
    ("ADANIENT", include_str!("../../../public/data/ADANIENT.csv"), 328),
    ("HINDUNILVR", include_str!("../../../public/data/HINDUNILVR.csv"), 512),
];

#[test]
fn every_bundled_csv_parses_and_backtests() {
    for (name, csv, rows) in SAMPLES {
        let out = execute(csv, YAML).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(out.stats.num_bars, *rows, "{name}");
        assert!(out.stats.ending_equity.is_finite() && out.stats.ending_equity > 0.0, "{name}");
        assert!(out.stats.max_drawdown_pct >= 0.0 && out.stats.max_drawdown_pct < 100.0, "{name}");
        for period in ["1W", "1M", "5D"] {
            let y = format!("{YAML}\ncandles:\n  period: {period}\n");
            execute(csv, &y).unwrap_or_else(|e| panic!("{name} {period}: {e}"));
        }
        println!(
            "{name}: {} bars, return {:.2}%, {} trades, max DD {:.2}%",
            out.stats.num_bars, out.stats.total_return_pct, out.stats.num_trades, out.stats.max_drawdown_pct
        );
    }
}

#[test]
fn scaled_lengths_trade_on_every_period() {
    for period in ["1D", "1W", "1M"] {
        for (name, csv, _) in SAMPLES {
            let y = format!("{YAML}\ncandles:\n  period: {period}\n  scale_lengths: true\n");
            let out = execute(csv, &y).unwrap();
            if period == "1D" && *name == "CUPID" {
                assert_eq!(out.stats.num_trades, 3);
                assert_eq!((out.stats.total_return_pct * 100.0).round() / 100.0, 40.81);
            }
            // Trent's single weekly EMA cross (2023-10-27) has RSI 73 ≥ 70, so it is filtered.
            if period == "1W" && *name != "TRENT" {
                assert!(out.stats.num_trades >= 1, "{name} 1W has no trades");
            }
            println!("{period} {name}: {} candles, {} trades, {:+.2}%", out.stats.num_bars, out.stats.num_trades, out.stats.total_return_pct);
        }
    }
}
