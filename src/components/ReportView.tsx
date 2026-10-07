import { EquityChart } from './EquityChart';
import { PriceChart } from './PriceChart';
import { inr, longDate, reasonLabel, shares, signedInr, signedPct, tone, trimNum } from '../format';
import type { BacktestResult, PolicySummary, PriceRow, Trade } from '../types';

type Props = {
  result: BacktestResult;
  sourceName: string;
  onBack: () => void;
};

export function ReportView({ result, sourceName, onBack }: Props) {
  const { stats, policy, trades } = result;
  const prices = withMarkers(result);

  return (
    <div className="stack">
      <div className="report-top">
        <button type="button" className="btn ghost" onClick={onBack}>
          Edit inputs
        </button>
        <div>
          <p className="eyebrow">{sourceName}</p>
          <h2 className="report-title">
            {stats.num_bars} bars · {longDate(stats.first_date)} – {longDate(stats.last_date)}
          </h2>
        </div>
      </div>

      <ul className="chips">
        {policyChips(policy).map((chip) => (
          <li key={chip}>{chip}</li>
        ))}
      </ul>

      <section className="kpis" aria-label="Backtest summary">
        <Kpi label="Total return" value={signedPct(stats.total_return_pct)} tone={tone(stats.total_return_pct)} />
        <Kpi label="Ending equity" value={inr(stats.ending_equity)} tone={tone(stats.ending_equity - stats.starting_capital)} />
        <Kpi label="Total P&L" value={signedInr(stats.total_pnl)} tone={tone(stats.total_pnl)} />
        <Kpi label="Max drawdown" value={`${stats.max_drawdown_pct.toFixed(2)}%`} tone={stats.max_drawdown_pct > 0 ? 'negative' : 'flat'} />
        <Kpi label="Trades" value={String(stats.num_trades)} tone="flat" />
        <Kpi label="Win rate" value={`${stats.win_rate_pct.toFixed(2)}%`} tone="flat" />
        <Kpi label="Avg hold" value={`${stats.avg_hold_days.toFixed(1)} days`} tone="flat" />
        <Kpi label="Starting capital" value={inr(stats.starting_capital)} tone="flat" />
      </section>

      <div className="charts">
        <EquityChart data={result.equity} />
        <PriceChart data={prices} />
      </div>

      <section className="card">
        <div className="card-head">
          <h2>Trades</h2>
          <p>{trades.length === 0 ? 'No round trip closed.' : `${trades.length} long delivery trades.`}</p>
        </div>
        {trades.length === 0 ? (
          <p className="hint">The policy never filled a buy on this series. Try a shorter SMA or turn the trend filter off.</p>
        ) : (
          <>
            <div className="trade-cards">
              {trades.map((trade) => (
                <TradeCard key={`${trade.entry_date}-${trade.exit_date}-${trade.shares}`} trade={trade} />
              ))}
            </div>
            <div className="table-wrap">
              <table>
                <caption className="sr-only">Closed trades</caption>
                <thead>
                  <tr>
                    <th scope="col">Entry</th>
                    <th scope="col">Exit</th>
                    <th scope="col">Reason</th>
                    <th scope="col">Shares</th>
                    <th scope="col">Entry price</th>
                    <th scope="col">Exit price</th>
                    <th scope="col">Hold</th>
                    <th scope="col">Return</th>
                    <th scope="col">P&amp;L</th>
                  </tr>
                </thead>
                <tbody>
                  {trades.map((trade) => (
                    <tr key={`${trade.entry_date}-${trade.exit_date}-${trade.shares}`}>
                      <td>{longDate(trade.entry_date)}</td>
                      <td>{longDate(trade.exit_date)}</td>
                      <td>{reasonLabel(trade.exit_reason)}</td>
                      <td>{shares(trade.shares)}</td>
                      <td>{inr(trade.entry_price)}</td>
                      <td>{inr(trade.exit_price)}</td>
                      <td>{trade.hold_days}d</td>
                      <td className={tone(trade.return_pct)}>{signedPct(trade.return_pct)}</td>
                      <td className={tone(trade.pnl)}>{signedInr(trade.pnl)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        )}
      </section>
    </div>
  );
}

function Kpi({ label, value, tone: toneName }: { label: string; value: string; tone: 'positive' | 'negative' | 'flat' }) {
  return (
    <article className={`kpi ${toneName}`}>
      <p>{label}</p>
      <strong>{value}</strong>
    </article>
  );
}

function TradeCard({ trade }: { trade: Trade }) {
  return (
    <article className="trade-card">
      <header>
        <span className="tag">Long</span>
        <span className="reason">{reasonLabel(trade.exit_reason)}</span>
        <strong className={tone(trade.pnl)}>{signedInr(trade.pnl)}</strong>
      </header>
      <p className="dates">
        {longDate(trade.entry_date)} → {longDate(trade.exit_date)} · {trade.hold_days} days
      </p>
      <dl>
        <div>
          <dt>Entry</dt>
          <dd>{inr(trade.entry_price)}</dd>
        </div>
        <div>
          <dt>Exit</dt>
          <dd>{inr(trade.exit_price)}</dd>
        </div>
        <div>
          <dt>Shares</dt>
          <dd>{shares(trade.shares)}</dd>
        </div>
        <div>
          <dt>Return</dt>
          <dd className={tone(trade.return_pct)}>{signedPct(trade.return_pct)}</dd>
        </div>
      </dl>
    </article>
  );
}

function withMarkers(result: BacktestResult): PriceRow[] {
  const buys = new Map(result.trades.map((trade) => [trade.entry_date, trade.entry_price]));
  const sells = new Map(result.trades.map((trade) => [trade.exit_date, trade.exit_price]));
  return result.prices.map((row) => ({
    ...row,
    buy: buys.get(row.date) ?? null,
    sell: sells.get(row.date) ?? null,
  }));
}

function policyChips(policy: PolicySummary): string[] {
  const chips = [
    `Cash ${inr(policy.starting_cash, 0)}`,
    policy.position_sizing === 'fraction' ? `${trimNum(policy.fraction * 100)}% of cash` : policy.position_sizing,
  ];
  if (policy.ema_fast != null && policy.ema_slow != null) {
    chips.push(`EMA ${policy.ema_fast}/${policy.ema_slow}`);
  }
  if (policy.buy_on_adx_confirm && policy.adx_min != null) {
    chips.push(`ADX confirm ≥ ${trimNum(policy.adx_min)}`);
  }
  if (policy.require_rsi_not_overbought) {
    chips.push(`RSI < ${trimNum(policy.rsi_overbought)}`);
  }
  if (policy.require_price_above_sma_trend && policy.sma_trend != null) {
    chips.push(`Close > SMA ${policy.sma_trend}`);
  }
  if (policy.stop_loss_pct != null) chips.push(`Stop ${trimNum(policy.stop_loss_pct)}%`);
  if (policy.trailing_stop_pct != null) chips.push(`Trail ${trimNum(policy.trailing_stop_pct)}%`);
  if (policy.atr_stop_mult != null) chips.push(`ATR stop × ${trimNum(policy.atr_stop_mult)}`);
  if (policy.trailing_atr_mult != null) chips.push(`ATR trail × ${trimNum(policy.trailing_atr_mult)}`);
  chips.push(policy.fill_at === 'next_open' ? 'Fill next open' : policy.fill_at);
  return chips;
}
