import type { PolicyForm } from './types';

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
};

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
  const numbers = [cash, pct, emaFast, emaSlow, adx, rsi, sma, stop, trail];

  if (numbers.some((value) => !Number.isFinite(value))) {
    return 'Enter a number in every policy field.';
  }
  if (cash <= 0) return 'Starting capital must be greater than 0.';
  if (pct <= 0 || pct > 100) return 'Position size must be greater than 0 and at most 100 percent.';
  if ([emaFast, emaSlow, sma].some((value) => !Number.isInteger(value) || value < 1)) {
    return 'EMA and SMA lengths must be whole numbers of at least 1.';
  }
  if (emaFast >= emaSlow) return 'Fast EMA must be shorter than slow EMA.';
  if (adx < 0) return 'ADX threshold cannot be negative.';
  if (rsi <= 0 || rsi > 100) return 'RSI max must be greater than 0 and at most 100.';
  if (stop <= 0) return 'Stop loss percent must be greater than 0.';
  if (trail <= 0) return 'Trailing stop percent must be greater than 0.';
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
      rsi_period: 14,
      rsi_oversold: 30,
      rsi_overbought: Number(form.rsiMax),
      atr_period: 14,
      adx_period: 14,
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
  });
}
