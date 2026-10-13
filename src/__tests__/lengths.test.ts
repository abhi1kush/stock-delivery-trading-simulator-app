import { describe, expect, it } from 'vitest';
import { DEFAULT_DAILY, scaleLengths, toDaily, tradingDaysPerCandle, warmupCandles, warmupWarning } from '../lengths';
import { buildPolicyJson, defaultPolicyForm, withCandlePeriod, withLength } from '../policy';

describe('length scaling', () => {
  it('trading days per candle', () => {
    expect(tradingDaysPerCandle('1D', '')).toBe(1);
    expect(tradingDaysPerCandle('ND', '3')).toBe(3);
    expect(tradingDaysPerCandle('ND', 'x')).toBe(1);
    expect(tradingDaysPerCandle('1W', '')).toBe(5);
    expect(tradingDaysPerCandle('1M', '')).toBe(21);
  });

  it('1D keeps daily values', () => {
    expect(scaleLengths(DEFAULT_DAILY, 1)).toEqual(DEFAULT_DAILY);
  });

  it('weekly and monthly match the engine (candles.rs scale_policy_lengths)', () => {
    expect(scaleLengths(DEFAULT_DAILY, 5)).toEqual({
      emaFast: 2, emaSlow: 5, smaTrend: 20, rsiPeriod: 5, adxPeriod: 3, atrPeriod: 3, downtrendLookback: 2,
    });
    expect(scaleLengths(DEFAULT_DAILY, 21)).toEqual({
      emaFast: 2, emaSlow: 4, smaTrend: 5, rsiPeriod: 5, adxPeriod: 2, atrPeriod: 2, downtrendLookback: 2,
    });
    expect(scaleLengths(DEFAULT_DAILY, 2).smaTrend).toBe(50);
  });

  it('keeps EMA fast < slow with a gap of at least 2', () => {
    const s = scaleLengths({ ...DEFAULT_DAILY, emaFast: 20, emaSlow: 25 }, 10);
    expect(s.emaSlow - s.emaFast).toBeGreaterThanOrEqual(2);
  });

  it('switching periods does not drift', () => {
    let f = defaultPolicyForm;
    for (const p of ['1W', '1M', 'ND', '1W', '1D'] as const) f = withCandlePeriod(f, p, '3');
    expect([f.emaFast, f.emaSlow, f.smaTrend, f.rsiPeriod, f.adxPeriod]).toEqual(['12', '26', '100', '14', '14']);
    expect(f.daily).toEqual(DEFAULT_DAILY);
  });

  it('manual edits are stored as daily equivalents', () => {
    let f = withCandlePeriod(defaultPolicyForm, '1W');
    expect(f.smaTrend).toBe('20');
    f = withLength(f, 'smaTrend', '30');
    expect(f.daily.smaTrend).toBe(150);
    f = withCandlePeriod(f, '1D');
    expect(f.smaTrend).toBe('150');
    f = withLength(f, 'smaTrend', 'abc');
    expect(f.daily.smaTrend).toBe(150);
    expect(toDaily('4', 21)).toBe(84);
  });

  it('stop percentages are not scaled and the JSON carries candle lengths', () => {
    const f = withCandlePeriod(defaultPolicyForm, '1W');
    const j = JSON.parse(buildPolicyJson(f));
    expect(j.risk.stop_loss_pct).toBe(8);
    expect(j.risk.trailing_stop_pct).toBe(10);
    expect(j.indicators).toMatchObject({ ema_fast: 2, ema_slow: 5, sma_trend: 20, rsi_period: 5, adx_period: 3 });
    expect(j.patterns.downtrend_lookback).toBe(2);
    expect(j.candles.period).toBe('1W');
  });

  it('warm-up warning', () => {
    expect(warmupCandles(DEFAULT_DAILY)).toBe(100);
    expect(warmupWarning(246, 1, DEFAULT_DAILY)).toBeNull();
    expect(warmupWarning(246, 5, DEFAULT_DAILY)).toMatch(/no buy can trigger/);
    expect(warmupWarning(246, 5, scaleLengths(DEFAULT_DAILY, 5))).toBeNull();
    expect(warmupWarning(246, 21, scaleLengths(DEFAULT_DAILY, 21))).toMatch(/few candles/);
  });
});
