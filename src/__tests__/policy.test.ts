import { describe, expect, it } from 'vitest';
import { buildPolicyJson, candlePeriodString, defaultPolicyForm, validateCandlePeriod, validateForm } from '../policy';
import { isDailyGranularity, medianGapDays, summarizeCsv } from '../csv';

describe('candle period', () => {
  it('maps the form to engine period strings', () => {
    expect(candlePeriodString(defaultPolicyForm)).toBe('1D');
    expect(candlePeriodString({ ...defaultPolicyForm, candlePeriod: '1W' })).toBe('1W');
    expect(candlePeriodString({ ...defaultPolicyForm, candlePeriod: 'ND', candleDays: '5' })).toBe('5D');
  });

  it('validates N days', () => {
    for (const bad of ['0', '61', '2.5', '', 'x']) {
      expect(validateCandlePeriod({ ...defaultPolicyForm, candlePeriod: 'ND', candleDays: bad }, 1)).not.toBeNull();
    }
    expect(validateCandlePeriod({ ...defaultPolicyForm, candlePeriod: 'ND', candleDays: '60' }, 1)).toBeNull();
  });

  it('only allows aggregation of daily data', () => {
    expect(validateCandlePeriod({ ...defaultPolicyForm, candlePeriod: '1W' }, 7)).toMatch(/not daily/);
    expect(validateCandlePeriod({ ...defaultPolicyForm, candlePeriod: '1D' }, 7)).toBeNull();
    expect(validateCandlePeriod({ ...defaultPolicyForm, candlePeriod: '1M' }, 1)).toBeNull();
  });

  it('detects input granularity', () => {
    expect(medianGapDays(['2024-03-01', '2024-03-04', '2024-03-05', '2024-03-06'])).toBe(1);
    expect(medianGapDays(['2024-01-05', '2024-01-12', '2024-01-19'])).toBe(7);
    expect(isDailyGranularity(3)).toBe(true);
    expect(isDailyGranularity(7)).toBe(false);
    const res = summarizeCsv('date,open,high,low,close,volume\n2024-01-02,1,1,1,1,1\n2024-01-03,1,1,1,1,1\n', 'x.csv');
    expect('meta' in res && res.meta.medianGapDays).toBe(1);
  });
});

describe('pattern weights and score', () => {
  it('validates weights and threshold', () => {
    expect(validateForm(defaultPolicyForm)).toBeNull();
    expect(validateForm({ ...defaultPolicyForm, hammerWeight: '1.2' })).toMatch(/weights/);
    expect(validateForm({ ...defaultPolicyForm, signalWeight: '-0.1' })).toMatch(/weights/);
    expect(validateForm({ ...defaultPolicyForm, buyThreshold: '0' })).toMatch(/threshold/);
  });

  it('sends candles and patterns to the engine', () => {
    const json = JSON.parse(
      buildPolicyJson({ ...defaultPolicyForm, candlePeriod: '1W', hammer: true, doji: true, buyThreshold: '0.6' }),
    );
    expect(json.candles.period).toBe('1W');
    expect(json.patterns).toMatchObject({
      hammer: true,
      doji: true,
      bullish_engulfing: false,
      hammer_weight: 0.6,
      three_white_soldiers_weight: 0.8,
      bullish_engulfing_weight: 0.7,
      doji_weight: 0.3,
      signal_weight: 1,
      buy_score_threshold: 0.6,
    });
  });
});
