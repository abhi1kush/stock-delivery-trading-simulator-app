import init, { run_backtest } from './bridge.js';
import type { BacktestResult } from '../types';

let ready: Promise<void> | null = null;

export function preloadEngine(): Promise<void> {
  if (!ready) {
    ready = init()
      .then(() => undefined)
      .catch((error: unknown) => {
        ready = null;
        throw error;
      });
  }
  return ready;
}

export async function runBacktest(pricesCsv: string, policyText: string): Promise<BacktestResult> {
  await preloadEngine();
  const json = run_backtest(pricesCsv, policyText);
  return JSON.parse(json) as BacktestResult;
}

export function errorText(error: unknown): string {
  if (error instanceof Error && error.message) return error.message;
  if (typeof error === 'string' && error) return error;
  return 'The backtest failed.';
}
