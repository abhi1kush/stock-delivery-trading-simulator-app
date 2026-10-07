import { CartesianGrid, ComposedChart, Line, ResponsiveContainer, Scatter, Tooltip, XAxis, YAxis } from 'recharts';
import { inr, longDate, shortDate } from '../format';
import type { PriceRow } from '../types';

type TipProps = {
  active?: boolean;
  payload?: Array<{ payload: PriceRow }>;
  label?: string;
};

type DotProps = {
  cx?: number;
  cy?: number;
  payload?: PriceRow;
};

export function PriceChart({ data }: { data: PriceRow[] }) {
  return (
    <section className="card chart-card" aria-label="Price with trade markers">
      <div className="card-head">
        <h2>Price and trades</h2>
        <p>Green dots are buy fills. Copper dots are sell fills. Averages are the causal values at that close.</p>
      </div>
      <div className="chart-body">
        <ResponsiveContainer width="100%" height="100%">
          <ComposedChart data={data} margin={{ top: 8, right: 8, left: 0, bottom: 0 }}>
            <CartesianGrid stroke="#e7dfd1" vertical={false} />
            <XAxis
              dataKey="date"
              tickFormatter={shortDate}
              minTickGap={28}
              tick={{ fill: '#6d655c', fontSize: 11 }}
              axisLine={false}
              tickLine={false}
            />
            <YAxis
              tickFormatter={(value: number) => inr(value, 0)}
              width={64}
              tick={{ fill: '#6d655c', fontSize: 11 }}
              axisLine={false}
              tickLine={false}
              domain={['auto', 'auto']}
            />
            <Tooltip content={<PriceTip />} />
            <Line type="monotone" dataKey="close" name="Close" stroke="#1b1714" strokeWidth={1.6} dot={false} isAnimationActive={false} />
            <Line type="monotone" dataKey="ema_fast" name="EMA fast" stroke="#1d6fbf" strokeWidth={1.2} dot={false} connectNulls={false} isAnimationActive={false} />
            <Line type="monotone" dataKey="ema_slow" name="EMA slow" stroke="#c45c26" strokeWidth={1.2} dot={false} connectNulls={false} isAnimationActive={false} />
            <Line
              type="monotone"
              dataKey="sma_trend"
              name="SMA trend"
              stroke="#0e6b52"
              strokeWidth={1.2}
              strokeDasharray="4 3"
              dot={false}
              connectNulls={false}
              isAnimationActive={false}
            />
            <Scatter dataKey="buy" name="Buy" fill="#0e6b52" isAnimationActive={false} shape={buyDot} />
            <Scatter dataKey="sell" name="Sell" fill="#a34720" isAnimationActive={false} shape={sellDot} />
          </ComposedChart>
        </ResponsiveContainer>
      </div>
      <ul className="legend">
        <li><i className="swatch close" /> Close</li>
        <li><i className="swatch ema-fast" /> EMA fast</li>
        <li><i className="swatch ema-slow" /> EMA slow</li>
        <li><i className="swatch sma" /> SMA trend</li>
        <li><i className="swatch buy" /> Buy</li>
        <li><i className="swatch sell" /> Sell</li>
      </ul>
    </section>
  );
}

function buyDot(props: DotProps) {
  if (props.cx == null || props.cy == null || props.payload?.buy == null) return <g />;
  return <circle cx={props.cx} cy={props.cy} r={5.5} fill="#0e6b52" stroke="#fffdf8" strokeWidth={1.5} />;
}

function sellDot(props: DotProps) {
  if (props.cx == null || props.cy == null || props.payload?.sell == null) return <g />;
  return <circle cx={props.cx} cy={props.cy} r={5.5} fill="#a34720" stroke="#fffdf8" strokeWidth={1.5} />;
}

function PriceTip({ active, payload, label }: TipProps) {
  if (!active || !payload?.length) return null;
  const row = payload[0].payload;
  return (
    <div className="tip">
      <p className="tip-date">{longDate(label ?? row.date)}</p>
      <p>
        O {inr(row.open)} · H {inr(row.high)}
      </p>
      <p>
        L {inr(row.low)} · C {inr(row.close)}
      </p>
      {row.buy != null ? <p>Buy fill {inr(row.buy)}</p> : null}
      {row.sell != null ? <p>Sell fill {inr(row.sell)}</p> : null}
    </div>
  );
}
