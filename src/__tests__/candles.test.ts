import { describe, expect, it } from 'vitest';
import { buildCandleRows } from '../components/CandleView';
import type { BacktestResult } from '../types';

const candle = (date: string, o: number, h: number, l: number, c: number, patterns: string[] = []) => ({
  date, open: o, high: h, low: l, close: c, volume: 10, sma_trend: null, ema_fast: null, ema_slow: null, patterns,
});

describe('candle rows', () => {
  it('puts buy/sell markers on the fill candles from the trade table', () => {
    const result = {
      prices: [candle('2024-01-01', 10, 11, 9, 10.5), candle('2024-01-02', 10.6, 12, 10, 11.8, ['BE']), candle('2024-01-03', 11.9, 12.5, 11, 12)],
      trades: [{ entry_date: '2024-01-02', exit_date: '2024-01-03', entry_price: 10.6, exit_price: 11.9, entry_reason: 'Bullish engulfing', exit_reason: 'trailing_stop' }],
    } as unknown as BacktestResult;
    const rows = buildCandleRows(result);
    expect(rows.map((r) => r.buyMark)).toEqual([null, 10, null]);
    expect(rows.map((r) => r.sellMark)).toEqual([null, null, 12.5]);
    expect(rows[1].fills).toEqual([{ side: 'buy', price: 10.6, reason: 'Bullish engulfing' }]);
    expect(rows[2].fills[0]).toMatchObject({ side: 'sell', reason: 'Trailing stop' });
    expect(rows[1].patternMark).toBe(12);
    expect(rows[1].range).toEqual([10, 12]);
  });
});
