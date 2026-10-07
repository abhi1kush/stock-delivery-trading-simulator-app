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

`npm run dev` compiles the WASM crate, then starts Vite.

## Production build

```bash
npm run build
npm run preview
```

`npm run build` runs `wasm-pack` (`--target web`) and then `tsc` and `vite build`. The output in `dist/` is a static site. Open it with any static file server. Backtests do not call a backend.

## Using it

1. Load the bundled CUPID sample or upload a daily OHLCV CSV with columns `date,open,high,low,close,volume` (`YYYY-MM-DD`).
2. Adjust the policy form, or paste YAML. A non-empty YAML box replaces the form. The form sends JSON; the engine accepts JSON or YAML.
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
