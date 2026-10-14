/**
 * Plain-language help for every policy parameter, keyed by the form control id.
 * Written from the engine code (crates/simulator-core). Keep each 1–3 sentences.
 */
const SCALED = 'counts candles, auto-scaled from trading days when you change the candle period.';

export const HELP: Record<string, string> = {
  'sample-picker':
    'Bundled real daily prices to test on. Each sample shows a different market shape (bear run, choppy, bull run, crash and recovery, range-bound). Pick one, then press Load.',
  'candle-period':
    'Groups daily rows into longer candles before anything is computed (open = first, high = max, low = min, close = last). Signals fire on a candle close and fill at the next candle’s open. Changing it rescales every candle-count length.',
  'candle-days':
    'How many trading sessions make one candle when the period is “N trading days” (1–60). Example: 5 groups each block of 5 rows, starting from the first row.',
  'starting-cash':
    'Cash at the start of the backtest, in rupees. Returns and P&L are measured against it.',
  'position-pct':
    'Share of current cash spent on each buy, rounded down to whole shares at the next open. Example: ₹1,00,000 at 95% and a ₹200 open buys 475 shares. Typical 50–100%.',
  'ema-fast':
    `Fast exponential moving average. A buy signal fires when it crosses above the slow EMA on a candle close. Typical 5–20 days; ${SCALED}`,
  'ema-slow':
    `Slow exponential moving average; must be longer than the fast one (at least 2 more). Larger gaps give fewer, later signals. Typical 20–50 days; ${SCALED}`,
  'adx-min':
    'ADX trend-strength level. A second buy signal fires when ADX crosses up through this value while the fast EMA is above the slow EMA. Typical 20–25; 0 disables it in practice.',
  'rsi-max':
    'Buy filter: any buy is skipped while RSI is at or above this level (overbought). It does not trigger sells. Typical 65–80.',
  'sma-trend':
    `Long-term simple moving average used by the trend filter below. Until it has enough candles to compute, no buy can pass the filter. Typical 50–200 days; ${SCALED}`,
  'rsi-period':
    `Length of the RSI (Wilder smoothing); a shorter RSI swings more and hits the cap more often. Typical 14 days, minimum 5 candles; ${SCALED}`,
  'adx-period':
    `Number of candles in the ADX trend-strength calculation; ADX needs about twice this to warm up. Typical 14 days; ${SCALED}`,
  'sma-filter':
    'When on, a buy is only allowed if the signal candle closes above the SMA trend line, so you only buy in long-term uptrends. Turn off to also buy below it, e.g. reversal patterns after a fall.',
  'pat-hammer':
    'Counts a hammer as buy evidence: small body near the top, lower shadow at least 2× the body, tiny upper shadow, after a decline over the downtrend lookback.',
  'w-hammer':
    'How much a hammer adds to the buy score: weight × strength (0.5–1, how cleanly it fits). Example: 0.6 × 0.9 = 0.54, enough on its own for a 0.5 threshold. Range 0–1.',
  'pat-threeWhiteSoldiers':
    'Counts three white soldiers: three rising bullish candles, each opening inside the previous body and closing higher, with small upper wicks.',
  'w-threeWhiteSoldiers':
    'How much three white soldiers add to the buy score (weight × strength 0.5–1). Default 0.8, the strongest pattern. Range 0–1.',
  'pat-bullishEngulfing':
    'Counts a bullish engulfing: a bearish candle followed by a bullish candle whose body fully covers it.',
  'w-bullishEngulfing':
    'How much a bullish engulfing adds to the buy score (weight × strength 0.5–1; a bigger engulfing body is stronger). Range 0–1.',
  'pat-doji':
    'Counts a doji (body ≤ 10% of the range) after a decline. A doji shows indecision, so it is weak evidence on its own.',
  'w-doji':
    'How much a doji adds to the buy score (weight × strength). Default 0.3, so with a 0.5 threshold it only helps when combined with a signal or another pattern. Range 0–1.',
  'signal-weight':
    'Score added when an indicator buy signal fires (EMA cross or ADX confirm). At 1 with a 0.5 threshold, signals buy on their own, as in the original policy. Range 0–1.',
  'buy-threshold':
    'A buy is armed when signal + pattern scores reach this value and the RSI/SMA filters pass. Lower = more trades. Example: 0.5 lets a strong hammer buy alone; 1.2 needs a signal plus a pattern.',
  'downtrend-lookback':
    `Candles used to decide “after a decline” for hammer and doji: the candle before the pattern must close below the close this many candles earlier. Typical 5 days; ${SCALED}`,
  'stop-pct':
    'Fixed stop below the entry price. If a candle’s low touches it, the position sells at the next candle’s open (a gap can fill lower). Example: buy 100 with 8% gives a stop at 92 (a price %, not scaled by candle period).',
  'trail-pct':
    'Trailing stop below the highest high since entry; it only moves up. Example: buy 100, peak 150, 10% gives a stop at 135. The higher of this and the fixed stop is active; it is a price %, not scaled by candle period.',
  'policy-yaml':
    'Paste a full engine policy. Any text here replaces the whole form, including ATR stops and sell signals the form does not show. Lengths are in candles unless you set candles.scale_lengths: true.',
};

/** Every form control that must carry a tooltip. */
export const HELP_IDS = Object.keys(HELP);
