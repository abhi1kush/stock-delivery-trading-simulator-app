# Delivery Backtester

Long-only delivery (cash) equity backtester. The Rust engine runs as WebAssembly in the browser. There is no server once the static files have loaded.

The trading model is the engine from [stock-delivery-trading-simulator](https://github.com/abhi1kush/stock-delivery-trading-simulator): SMA / EMA / RSI / ATR / ADX, next-open fills, and the causality rules in [`docs/CAUSALITY.md`](docs/CAUSALITY.md). The CLI (`clap`) is not part of the WASM build. Shared logic lives in `crates/simulator-core`.

## Prerequisites

- Node.js 20 or newer
- Rust stable (1.85+). `rust-toolchain.toml` selects stable and the `wasm32-unknown-unknown` target
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
rustup target add wasm32-unknown-unknown
```

## Run

```bash
npm install
npm run dev
```

`npm run dev` compiles the WASM crate, then starts Vite at http://localhost:5173/stock-delivery-trading-simulator-app/ (the same project-site base path as GitHub Pages).

## Production build

```bash
npm run build
npm run preview
```

`npm run build` runs `wasm-pack` (`--target web`) and then `tsc` and `vite build`. The output in `dist/` is a static site. Open it with any static file server. Backtests do not call a backend.

Vite `base` is `/stock-delivery-trading-simulator-app/`. Sample CSV, policy YAML, and the WASM module are loaded under that prefix.

## GitHub Pages

Pushes to `main` run [`.github/workflows/pages.yml`](.github/workflows/pages.yml): Rust, wasm-pack, `npm ci`, `npm run build`, then `upload-pages-artifact` and `deploy-pages`.

The site is https://abhi1kush.github.io/stock-delivery-trading-simulator-app/

If the first deployment cannot publish, open the repository **Settings → Pages** and set **Source** to **GitHub Actions** once. Later pushes publish on their own.

## Using it

1. Pick a bundled sample (CUPID by default, or one of five real NSE stocks) or upload a daily OHLCV CSV with columns `date,open,high,low,close,volume` (`YYYY-MM-DD`).
2. Pick a candle period and the patterns you want, adjust the policy form, or paste YAML. A non-empty YAML box replaces the form. The form sends JSON; the engine accepts JSON or YAML.
3. Run. The report shows return, drawdown, win rate, the equity curve, price with buy/sell fills, and the trade list.

The layout is a single column on a phone (~375px) and a wider grid on desktop. Trade rows become cards on small screens.

### Default policy

The form matches [`policy/example_policy.yaml`](policy/example_policy.yaml):

| Piece | Default |
| --- | --- |
| Capital | ₹100,000, 95% of cash per trade |
| Entry | EMA 12/26 cross, plus ADX crossing up through 20 while the fast average is above the slow one |
| Filter | RSI below 70, and close above SMA 100 |
| Risk | 8% initial stop, 10% trail from the peak high. ATR stops are off (`null`), same as the YAML |
| Fills | Long only, signal on the close, fill at the next open |

On the bundled CUPID sample (246 sessions, 2025-10-06 through 2026-10-05) that policy returns **+40.81%** in **3 trades** (ending equity ₹1,40,814.80, max drawdown 10.87%). `cargo test -p simulator-core` locks those figures to the upstream engine.

### Candle period

The input CSV is daily. The **Candle period** control groups rows into longer candles before indicators, signals and patterns run (open = first, high = max, low = min, close = last, volume = sum):

| Choice | Grouping |
| --- | --- |
| 1 day (default) | input rows unchanged |
| N trading days (1–60) | consecutive blocks of N rows from the first row; the last block may be shorter |
| 1 week | ISO calendar week (Mon–Sun) |
| 1 month | calendar month |

A candle is dated by its last session (when its close is known), and a buy fills at the next candle's open. Indicator lengths count candles, so SMA 100 on weekly candles needs 100 weeks of data; turn the SMA filter off or shorten it for long periods. If the input rows are more than 4 days apart (already weekly or coarser) only the input period is allowed. YAML: `candles: { period: 1W }` (`1D`, `5D`, `1W`, `1M`).

### Lengths follow the candle period

Indicator lengths count candles, so SMA 100 on weekly candles would mean 100 weeks. When you change the candle period, the form rescales every candle-count field from its **daily** value: EMA fast/slow, SMA trend, RSI length, ADX length (and ATR length, used only by YAML ATR stops), and the pattern downtrend lookback.

```
candles = max(min, round(daily / trading days per candle))     1D = 1, N days = N, 1W ≈ 5, 1M ≈ 21
```

Minimums: 2 for every length, 5 for RSI (a shorter RSI is driven by the last one or two candles and sits above 70 after almost any up-candle, which would make the RSI cap block nearly every buy), and EMA slow ≥ fast + 2. Daily values are the source of truth, so switching periods back and forth never drifts; typing a length stores it back as its daily equivalent (weekly 30 → 150 days). Each scaled field shows a hint such as "≈100 trading days". Stop-loss and trailing percentages are prices and are not scaled. A banner warns when the data has too few candles for the longest indicator's warm-up (max of SMA, EMA slow, 2 × ADX, RSI + 1).

Default lengths by period: 1D 12/26 EMA, SMA 100, RSI 14, ADX 14, lookback 5 · 1W 2/5, SMA 20, RSI 5, ADX 3, lookback 2 · 1M 2/4, SMA 5, RSI 5, ADX 2, lookback 2.

In YAML, lengths are taken as candles unless you add `candles: { period: 1W, scale_lengths: true }`, which makes the engine treat them as trading days and apply the same rules.

Default policy, by period (trades, return):

| Sample | 1D | 1W | 1M |
| --- | --- | --- | --- |
| CUPID | 3, +40.81% | 1, +22.20% | 0 |
| PAYTM | 2, −11.95% | 1, +6.14% | 0 |
| SUZLON | 4, +6.41% | 4, +42.68% | 2, −19.49% |
| TRENT | 3, +42.05% | 0 | 0 |
| ADANIENT | 1, −1.26% | 2, −9.37% | 2, +5.66% |
| HINDUNILVR | 3, −14.60% | 1, −4.61% | 2, −8.20% |

Zeros: Trent's only weekly EMA crossover (week ending 2023-10-27) has RSI 73, above the 70 cap; it was trending the whole time, so there is no other cross. On monthly candles CUPID and PAYTM have 13 candles and Trent 18, fewer than the warm-up plus a cross, so nothing triggers; the form warns about this.

### Candlestick patterns and the buy score

Patterns are weighted evidence, not hard triggers. On each candle the engine computes a deterministic buy score:

```
score = signal_weight × [EMA cross / ADX confirm (or other enabled entry signal) fired]
      + Σ over enabled patterns found on this candle: weight × strength
```

A buy is armed when `score ≥ buy_score_threshold` **and** the existing filters pass (RSI below the cap, close above SMA trend when that box is ticked). The strategy still trades on indicator signals when no pattern shows up; a pattern adds to the score when it does, and a strong enough pattern can trigger a buy on its own. There is no randomness.

| Setting | Default |
| --- | --- |
| Hammer weight | 0.6 |
| Three white soldiers weight | 0.8 |
| Bullish engulfing weight | 0.7 |
| Doji weight | 0.3 |
| Signal weight | 1.0 |
| Buy score threshold | 0.5 |
| Downtrend lookback | 5 candles |

With every pattern off (the default) the score is 1.0 whenever an entry signal fires, so results equal the original policy. With the defaults a pattern alone buys when weight × strength ≥ 0.5: a hammer needs strength ≥ 0.84, and a doji (0.3 × ≤ 1) never buys alone, only together with a signal.

Thresholds (`crates/simulator-core/src/patterns.rs`; body = |close − open|, range = high − low):

| Pattern | Rule | Strength (0.5–1) |
| --- | --- | --- |
| Downtrend | close of the candle before the pattern < close 5 candles earlier | decline % feeds strength |
| Hammer | body ≤ 35% of range, lower shadow ≥ 2× body and ≥ 60% of range, upper shadow ≤ 10% of range, after a downtrend | 0.5 + 0.25·min(1, (lower/body − 2)/2) + 0.25·min(1, decline/5%) |
| Three white soldiers | 3 bullish candles, body ≥ 50% of range, each of the last two opens inside the prior body and closes above the prior close, upper wick ≤ 30% of body | 0.5 + 0.5·(1 − largest wick/body ÷ 0.30) |
| Bullish engulfing | bearish candle, then a bullish candle with open ≤ prior close, close ≥ prior open, larger body | 0.5 + 0.5·min(1, body ratio − 1) |
| Doji | body ≤ 10% of range, after a downtrend (indecision is only a buy hint after selling) | 0.5 + 0.25·(1 − body%/10%) + 0.25·min(1, decline/5%) |

Patterns read only the current and earlier candles, so they follow the same no-look-ahead rule as the indicators.

### Buy explanations

Every trade row shows why the buy happened: the score breakdown, each signal or pattern with its candle date(s) and key numbers, the filters that passed, and the fill. Example:

> Buy score 0.52 ≥ threshold 0.50 [Hammer 0.60×0.86 = 0.52]. Hammer on 2026-05-20: lower shadow 3.0x body, upper shadow 4% of range, after 5-candle decline of 4.6%; close 119.44. Filters passed: RSI 62.9 < 70. Filled at the next candle's open 120.40 (candle dated 2026-05-22).

YAML for the same settings:

```yaml
candles:
  period: 2D
patterns:
  hammer: true
  three_white_soldiers: true
  bullish_engulfing: true
  doji: true
  hammer_weight: 0.6
  three_white_soldiers_weight: 0.8
  bullish_engulfing_weight: 0.7
  doji_weight: 0.3
  signal_weight: 1.0
  buy_score_threshold: 0.5
  downtrend_lookback: 5
```

## Tests

```bash
npm test        # vitest (form, period validation) + cargo test --workspace
```

## Bundled sample data

The picker on the Prices card offers CUPID (the original default) plus five real NSE stocks, each chosen for a distinct chart shape and checked numerically (closes, daily log returns, annualised volatility = stdev × √252):

| File | Stock | Window | Shape | Backing numbers | Default policy |
| --- | --- | --- | --- | --- | --- |
| `PAYTM.csv` | One 97 Communications (PAYTM) | 2021-11-18 – 2022-11-30 (257 rows) | One-year bear run | ₹1,560.80 → ₹481.70 (−69.1%), max drawdown 75.5%, vol 58% | −11.95%, 2 trades |
| `SUZLON.csv` | Suzlon Energy (SUZLON) | 2021-01-01 – 2022-12-30 (496) | Choppy, inconsistent swings | vol 61%, drawdown 52.3%, net move / total path 0.041 | +6.41%, 4 trades |
| `TRENT.csv` | Trent (TRENT) | 2023-01-02 – 2024-06-28 (365) | Strong steady bull run | ₹1,342 → ₹5,480 (+308%), max drawdown 12.7% | +42.05%, 3 trades |
| `ADANIENT.csv` | Adani Enterprises (ADANIENT) | 2022-10-03 – 2024-01-31 (328) | V-shaped crash and recovery | −71.3% peak (2022-12-20) to low (2023-02-27), +163% from the low; ends −0.5% vs start | −1.26%, 1 trade |
| `HINDUNILVR.csv` | Hindustan Unilever (HINDUNILVR) | 2022-06-01 – 2024-06-28 (512) | Range-bound blue chip | +7.1% over two years, closes ₹2,110–₹2,757, vol 19% | −14.60%, 3 trades |

Source: Yahoo Finance chart API (`query1.finance.yahoo.com/v8/finance/chart/<SYM>.NS`, `interval=1d`), fetched 13 Oct 2026. Prices are split/bonus-adjusted by Yahoo and not dividend-adjusted. Rows with null prices were dropped, prices rounded to 2 decimals, and high/low widened to include open/close where rounding broke the OHLC invariant. Suzlon has 16 sessions in early 2021 with open = high = low = close (circuit-locked days); they are real. Candidates were picked from well-known moves (Paytm post-IPO slide, Adani after the Hindenburg report, Trent's 2023 rally) and confirmed against the data above.

The sample data is for research and education only. Yahoo Finance and NSE terms of use apply; do not redistribute it commercially.

## Layout

```
crates/simulator-core/     engine: CSV, policy, indicators, backtest
crates/simulator-wasm/     wasm-bindgen cdylib
                           run_backtest(prices_csv, policy_json) -> JSON
src/                       Vite + React + TypeScript UI
public/data/CUPID.csv      bundled sample
policy/example_policy.yaml default policy
docs/CAUSALITY.md          no-look-ahead rules
```

Generated WASM (`src/wasm/pkg/`) is gitignored. `npm run wasm` rebuilds it.

## Engine API

```ts
import init, { run_backtest } from './src/wasm/pkg/simulator_wasm.js';

await init();
const result = JSON.parse(run_backtest(csvText, policyJson));
```

`policyJson` may also be YAML when the text does not start with `{`. The JSON result has `stats`, `policy`, `equity`, `prices`, and `trades`.
