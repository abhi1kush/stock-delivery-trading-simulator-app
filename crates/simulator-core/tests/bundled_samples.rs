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
