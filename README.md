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

1. Load the bundled CUPID sample or upload a daily OHLCV CSV with columns `date,open,high,low,close,volume` (`YYYY-MM-DD`).
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
