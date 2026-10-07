# Causality / no look-ahead rules

This backtester is built to avoid peeking at future bars. Summary of the
contract enforced in `crates/simulator-core/src/indicators.rs` and
`crates/simulator-core/src/engine.rs`.

## Indicators (bar `i`)

Every series at index `i` uses only `bars[0..=i]` (or a classic recursive
update that itself only depends on past values):

| Series | Rule |
|--------|------|
| SMA | Window `closes[i-period+1..=i]` |
| EMA | Seed = SMA of first `period` closes; then `α·close[i] + (1-α)·ema[i-1]` |
| RSI | Wilder avg gain/loss from close-to-close changes through `i` |
| ATR | Wilder smooth of true range; TR[i] uses bar `i` and close[i-1] only |
| ADX | +DM/−DM from consecutive highs/lows through `i`; Wilder DX → ADX |
| Cross / RSI / ADX flags | Compare values at `i-1` and `i` only |
| SMA trend (`sma_trend`) | Same as SMA; filter compares signal-bar **close** to `sma_trend[i]` |

No indicator reads `bars[i+1..]`.

## Signals (day `i`)

Buy/sell signal logic reads only `indicators[i]` (and policy thresholds).
It does **not** look at future bars. A signal on day `i` arms a **pending**
order filled at day `i+1` **open** (or is dropped if `i` is the last bar).

## Fills

- `execution.fill_at: next_open` only.
- Decision after day `i` close → fill at `i+1` open.
- Open position at end of data is marked at the last close (`end_of_data`).

## Stops / take-profit (same-day high/low)

**Assumption (documented):** once a position is open, the engine may use that
**same bar’s** high/low to detect stop-loss, trailing-stop, or take-profit
hits. That models “stop working during the session,” not a future bar.

Causal constraints that go with this:

1. **At the open fill** the initial hard stop uses **prior-bar ATR**
   (`atr[i-1]`), never `atr[i]` (today’s ATR is not known at the open).
2. **Peak high at fill** starts at the **entry/open** price, then ratchets
   with the day’s high when the bar is processed — it is not set to the
   full-day high at the instant of the open fill.
3. **Intrabar trail distance** also uses **prior-bar ATR** so stop levels
   checked against today’s low do not embed today’s yet-unknown true range.
4. Stop/TP detection sets a **pending sell**; the exit fills at the
   **next** open (not at the stop price itself). Gaps through the stop are
   therefore possible — pessimistic on gaps down, optimistic on gaps up.
5. **OHLC path ambiguity:** if both stop and take-profit could be touched
   on the same bar, the engine checks stop first (conservative).

## What is intentionally allowed

- Signal day `i` → fill at `i+1` open.
- Same-day high/low for stop/TP **after** entry, under the assumptions above.
- Force-close at last close when no further bar exists to fill a pending exit.
