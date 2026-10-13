import type { CandleChoice, PolicyForm } from './types';
import { DEFAULT_DAILY, scaleLengths, toDaily, tradingDaysPerCandle, type LengthKey } from './lengths';

export const defaultPolicyForm: PolicyForm = {
  startingCash: '100000',
  positionPct: '95',
  emaFast: '12',
  emaSlow: '26',
  adxMin: '20',
  rsiMax: '70',
  smaTrend: '100',
  requirePriceAboveSma: true,
  stopLossPct: '8',
  trailingStopPct: '10',
  candlePeriod: '1D',
  candleDays: '2',
  rsiPeriod: '14',
  adxPeriod: '14',
  atrPeriod: '14',
  downtrendLookback: '5',
  daily: { ...DEFAULT_DAILY },
  hammer: false,
  threeWhiteSoldiers: false,
  bullishEngulfing: false,
  doji: false,
  hammerWeight: '0.6',
  threeWhiteSoldiersWeight: '0.8',
  bullishEngulfingWeight: '0.7',
  dojiWeight: '0.3',
  signalWeight: '1',
  buyThreshold: '0.5',
};

export const MAX_CANDLE_DAYS = 60;

export const LENGTH_KEYS: LengthKey[] = ['emaFast', 'emaSlow', 'smaTrend', 'rsiPeriod', 'adxPeriod', 'atrPeriod', 'downtrendLookback'];

export function formFactor(form: PolicyForm): number {
  return tradingDaysPerCandle(form.candlePeriod, form.candleDays);
}

/** Change the candle period and rescale every candle-count field from its daily value. */
export function withCandlePeriod(form: PolicyForm, choice: CandleChoice, candleDays = form.candleDays): PolicyForm {
  const next = { ...form, candlePeriod: choice, candleDays };
  const scaled = scaleLengths(form.daily, formFactor(next));
  for (const key of LENGTH_KEYS) next[key] = String(scaled[key]);
  return next;
}

/** A length edited in candles; stored back as its daily equivalent when valid. */
export function withLength(form: PolicyForm, key: LengthKey, value: string): PolicyForm {
  const daily = toDaily(value, formFactor(form));
  return { ...form, [key]: value, daily: daily == null ? form.daily : { ...form.daily, [key]: daily } };
}

/** Engine period string for the form, e.g. '1D', '5D', '1W', '1M'. */
export function candlePeriodString(form: PolicyForm): string {
  return form.candlePeriod === 'ND' ? `${Number(form.candleDays)}D` : form.candlePeriod;
}

/** Validate the candle period against the loaded data's granularity. */
export function validateCandlePeriod(form: PolicyForm, medianGapDays: number | null): string | null {
  if (form.candlePeriod === 'ND') {
    const n = Number(form.candleDays);
    if (!Number.isInteger(n) || n < 1 || n > MAX_CANDLE_DAYS) {
      return `Candle length must be a whole number of days from 1 to ${MAX_CANDLE_DAYS}.`;
    }
  }
  const period = candlePeriodString(form);
  if (period !== '1D' && medianGapDays != null && medianGapDays > 4) {
    return `The loaded prices are about ${medianGapDays} days apart, not daily. Use the 1 day candle period.`;
  }
  return null;
}

function finite(value: string): number {
  return Number(value);
}

export function validateForm(form: PolicyForm): string | null {
  const cash = finite(form.startingCash);
  const pct = finite(form.positionPct);
  const emaFast = finite(form.emaFast);
  const emaSlow = finite(form.emaSlow);
  const adx = finite(form.adxMin);
  const rsi = finite(form.rsiMax);
  const sma = finite(form.smaTrend);
  const stop = finite(form.stopLossPct);
  const trail = finite(form.trailingStopPct);
  const weights = [form.hammerWeight, form.threeWhiteSoldiersWeight, form.bullishEngulfingWeight, form.dojiWeight, form.signalWeight].map(finite);
  const threshold = finite(form.buyThreshold);
  const lens = [form.rsiPeriod, form.adxPeriod, form.atrPeriod, form.downtrendLookback].map(finite);
  const numbers = [cash, pct, emaFast, emaSlow, adx, rsi, sma, stop, trail, threshold, ...weights, ...lens];

  if (numbers.some((value) => !Number.isFinite(value))) {
    return 'Enter a number in every policy field.';
  }
  if (cash <= 0) return 'Starting capital must be greater than 0.';
  if (pct <= 0 || pct > 100) return 'Position size must be greater than 0 and at most 100 percent.';
  if ([emaFast, emaSlow, sma].some((value) => !Number.isInteger(value) || value < 1)) {
    return 'EMA and SMA lengths must be whole numbers of at least 1.';
  }
  if (lens.some((value) => !Number.isInteger(value) || value < 2)) {
    return 'RSI, ADX, ATR and downtrend lengths must be whole numbers of at least 2.';
  }
  if (Number(form.downtrendLookback) > 50) return 'Downtrend lookback must be at most 50 candles.';
  if (emaFast >= emaSlow) return 'Fast EMA must be shorter than slow EMA.';
  if (adx < 0) return 'ADX threshold cannot be negative.';
  if (rsi <= 0 || rsi > 100) return 'RSI max must be greater than 0 and at most 100.';
  if (stop <= 0) return 'Stop loss percent must be greater than 0.';
  if (trail <= 0) return 'Trailing stop percent must be greater than 0.';
  if (weights.some((w) => w < 0 || w > 1)) return 'Pattern and signal weights must be between 0 and 1.';
  if (threshold <= 0 || threshold > 5) return 'Buy score threshold must be greater than 0 and at most 5.';
  return null;
}

/** JSON policy matching `policy/example_policy.yaml`, with the form overrides applied. */
export function buildPolicyJson(form: PolicyForm): string {
  return JSON.stringify({
    capital: {
      starting_cash: Number(form.startingCash),
      position_sizing: 'fraction',
      fraction: Number(form.positionPct) / 100,
      fixed_shares: 100,
      fixed_amount: 10000,
    },
    risk: {
      stop_loss_pct: Number(form.stopLossPct),
      take_profit_pct: null,
      atr_stop_mult: null,
      trailing_stop_pct: Number(form.trailingStopPct),
      trailing_atr_mult: null,
      allow_short: false,
    },
    indicators: {
      sma_fast: null,
      sma_slow: null,
      sma_trend: Number(form.smaTrend),
      ema_fast: Number(form.emaFast),
      ema_slow: Number(form.emaSlow),
      rsi_period: Number(form.rsiPeriod),
      rsi_oversold: 30,
      rsi_overbought: Number(form.rsiMax),
      atr_period: Number(form.atrPeriod),
      adx_period: Number(form.adxPeriod),
      adx_min: Number(form.adxMin),
    },
    signals: {
      buy_on_sma_cross_up: false,
      sell_on_sma_cross_down: false,
      buy_on_ema_cross_up: true,
      sell_on_ema_cross_down: false,
      buy_on_rsi_oversold_exit: false,
      sell_on_rsi_overbought_exit: false,
      require_rsi_not_overbought: true,
      require_adx_trend: false,
      buy_on_adx_confirm: true,
      require_price_above_sma_trend: form.requirePriceAboveSma,
    },
    execution: { fill_at: 'next_open' },
    candles: { period: candlePeriodString(form) },
    patterns: {
      hammer: form.hammer,
      three_white_soldiers: form.threeWhiteSoldiers,
      bullish_engulfing: form.bullishEngulfing,
      doji: form.doji,
      hammer_weight: Number(form.hammerWeight),
      three_white_soldiers_weight: Number(form.threeWhiteSoldiersWeight),
      bullish_engulfing_weight: Number(form.bullishEngulfingWeight),
      doji_weight: Number(form.dojiWeight),
      signal_weight: Number(form.signalWeight),
      buy_score_threshold: Number(form.buyThreshold),
      downtrend_lookback: Number(form.downtrendLookback),
    },
  });
}
