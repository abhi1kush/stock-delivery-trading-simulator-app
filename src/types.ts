export type PolicyForm = {
  startingCash: string;
  positionPct: string;
  emaFast: string;
  emaSlow: string;
  adxMin: string;
  rsiMax: string;
  smaTrend: string;
  requirePriceAboveSma: boolean;
  stopLossPct: string;
  trailingStopPct: string;
};

export type Stats = {
  starting_capital: number;
  ending_equity: number;
  total_return_pct: number;
  total_pnl: number;
  num_trades: number;
  winning_trades: number;
  losing_trades: number;
  win_rate_pct: number;
  max_drawdown_pct: number;
  avg_hold_days: number;
  num_bars: number;
  first_date: string;
  last_date: string;
};

export type PolicySummary = {
  starting_cash: number;
  position_sizing: string;
  fraction: number;
  ema_fast: number | null;
  ema_slow: number | null;
  adx_min: number | null;
  rsi_period: number | null;
  rsi_overbought: number;
  sma_trend: number | null;
  require_price_above_sma_trend: boolean;
  stop_loss_pct: number | null;
  trailing_stop_pct: number | null;
  atr_stop_mult: number | null;
  trailing_atr_mult: number | null;
  buy_on_ema_cross_up: boolean;
  buy_on_adx_confirm: boolean;
  require_rsi_not_overbought: boolean;
  fill_at: string;
};

export type EquityPoint = {
  date: string;
  equity: number;
  cash: number;
  shares: number;
  close: number;
};

export type PriceOut = {
  date: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  sma_trend: number | null;
  ema_fast: number | null;
  ema_slow: number | null;
};

export type PriceRow = PriceOut & {
  buy: number | null;
  sell: number | null;
};

export type Trade = {
  entry_date: string;
  exit_date: string;
  entry_price: number;
  exit_price: number;
  shares: number;
  side: string;
  exit_reason: string;
  pnl: number;
  return_pct: number;
  hold_days: number;
};

export type BacktestResult = {
  stats: Stats;
  policy: PolicySummary;
  equity: EquityPoint[];
  prices: PriceOut[];
  trades: Trade[];
};

export type CsvMeta = {
  name: string;
  rows: number;
  start: string;
  end: string;
};
