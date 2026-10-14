// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { CHART_MODE_KEY, PriceChart } from '../components/PriceChart';
import type { BacktestResult } from '../types';

class RO { observe() {} unobserve() {} disconnect() {} }
(globalThis as unknown as { ResizeObserver: typeof RO }).ResizeObserver = RO;

const result = {
  stats: { first_date: '2024-01-01', num_bars: 1 },
  policy: { candle_period: '1W', ema_fast: 2, ema_slow: 5, sma_trend: 20, require_price_above_sma_trend: true },
  prices: [{ date: '2024-01-05', open: 1, high: 2, low: 0.5, close: 1.5, volume: 1, sma_trend: null, ema_fast: null, ema_slow: null, patterns: [] }],
  trades: [],
} as unknown as BacktestResult;

beforeEach(() => localStorage.clear());
afterEach(cleanup);

describe('price chart Line | Candles toggle', () => {
  it('defaults to Line and remembers Candles', () => {
    const { unmount } = render(<PriceChart data={[]} result={result} />);
    expect(screen.getByRole('button', { name: 'Line' }).getAttribute('aria-pressed')).toBe('true');
    expect(screen.queryByText(/1W candles/)).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Candles' }));
    expect(screen.getByText(/1W candles: 1 total/)).toBeTruthy();
    expect(localStorage.getItem(CHART_MODE_KEY)).toBe('candles');
    unmount();
    render(<PriceChart data={[]} result={result} />);
    expect(screen.getByRole('button', { name: 'Candles' }).getAttribute('aria-pressed')).toBe('true');
  });
});
