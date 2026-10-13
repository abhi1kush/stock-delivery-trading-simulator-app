/**
 * Candle-count settings scale with the candle period.
 *
 * Daily values are the source of truth. The form shows
 *   candles = max(min, round(daily / tradingDaysPerCandle))
 * and a manual edit is stored back as daily = candles × tradingDaysPerCandle,
 * so switching periods back and forth never drifts.
 * Price percentages (stop loss, trailing stop) are not scaled.
 */
import type { CandleChoice } from './types';

export type LengthKey = 'emaFast' | 'emaSlow' | 'smaTrend' | 'rsiPeriod' | 'adxPeriod' | 'atrPeriod' | 'downtrendLookback';
export type DailyLengths = Record<LengthKey, number>;

export const DEFAULT_DAILY: DailyLengths = {
  emaFast: 12,
  emaSlow: 26,
  smaTrend: 100,
  rsiPeriod: 14,
  adxPeriod: 14,
  atrPeriod: 14,
  downtrendLookback: 5,
};

/** Smallest sensible length in candles. */
export const MIN_LEN: DailyLengths = {
  emaFast: 2,
  emaSlow: 4,
  smaTrend: 2,
  // RSI under ~5 candles is dominated by the last move and blocks most buys via the RSI cap.
  rsiPeriod: 5,
  adxPeriod: 2,
  atrPeriod: 2,
  downtrendLookback: 2,
};

/** Trading days per candle: 1D = 1, ND = N, 1W ≈ 5, 1M ≈ 21. */
export function tradingDaysPerCandle(choice: CandleChoice, nDays: string | number): number {
  switch (choice) {
    case '1W':
      return 5;
    case '1M':
      return 21;
    case 'ND': {
      const n = Number(nDays);
      return Number.isInteger(n) && n >= 1 ? n : 1;
    }
    default:
      return 1;
  }
}

/** Daily lengths → lengths in candles, with minimums and EMA fast < slow (≥ 2 apart). */
export function scaleLengths(daily: DailyLengths, factor: number): DailyLengths {
  if (factor === 1) return { ...daily };
  const out = {} as DailyLengths;
  for (const key of Object.keys(daily) as LengthKey[]) {
    out[key] = Math.max(MIN_LEN[key], Math.round(daily[key] / factor));
  }
  if (out.emaSlow - out.emaFast < 2) out.emaSlow = out.emaFast + 2;
  return out;
}

/** A length typed in candles → its daily equivalent (null when not a whole number ≥ 1). */
export function toDaily(candles: string, factor: number): number | null {
  const n = Number(candles);
  if (!Number.isInteger(n) || n < 1) return null;
  return n * factor;
}

/** Candles an indicator needs before it produces values (Wilder ADX needs ~2 × period). */
export function warmupCandles(len: DailyLengths): number {
  return Math.max(len.smaTrend, len.emaSlow, 2 * len.adxPeriod, len.rsiPeriod + 1);
}

/** Warning when the data has too few candles for the warm-up, else null. */
export function warmupWarning(rows: number, factor: number, len: DailyLengths): string | null {
  const candles = Math.floor(rows / factor);
  const need = warmupCandles(len);
  if (candles >= need + 10) return null;
  return `About ${candles} candles, but the longest indicator needs ${need} to warm up${
    candles <= need ? ', so no buy can trigger' : ', leaving few candles to trade'
  }. Use a shorter period or more data.`;
}
