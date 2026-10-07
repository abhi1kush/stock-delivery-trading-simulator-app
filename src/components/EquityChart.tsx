import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import { axisInr, inr, longDate, shortDate } from '../format';
import type { EquityPoint } from '../types';

type TipProps = {
  active?: boolean;
  payload?: Array<{ payload: EquityPoint }>;
  label?: string;
};

export function EquityChart({ data }: { data: EquityPoint[] }) {
  return (
    <section className="card chart-card" aria-label="Equity curve">
      <div className="card-head">
        <h2>Equity</h2>
        <p>Marked to the close each day. An open position at the end is closed at the last close.</p>
      </div>
      <div className="chart-body">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{ top: 8, right: 8, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id="equity-fill" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#0e6b52" stopOpacity={0.28} />
                <stop offset="100%" stopColor="#0e6b52" stopOpacity={0.02} />
              </linearGradient>
            </defs>
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
              tickFormatter={axisInr}
              width={72}
              tick={{ fill: '#6d655c', fontSize: 11 }}
              axisLine={false}
              tickLine={false}
              domain={['auto', 'auto']}
            />
            <Tooltip content={<EquityTip />} />
            <Area
              type="monotone"
              dataKey="equity"
              name="Equity"
              stroke="#0e6b52"
              strokeWidth={2}
              fill="url(#equity-fill)"
              dot={false}
              isAnimationActive={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </section>
  );
}

function EquityTip({ active, payload, label }: TipProps) {
  if (!active || !payload?.length) return null;
  const row = payload[0].payload;
  return (
    <div className="tip">
      <p className="tip-date">{longDate(label ?? row.date)}</p>
      <p>Equity {inr(row.equity)}</p>
      <p>Cash {inr(row.cash)}</p>
      <p>Close {inr(row.close)}</p>
    </div>
  );
}
