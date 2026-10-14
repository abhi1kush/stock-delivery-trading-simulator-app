import { useMemo, useState } from 'react';
import { Bar, Brush, CartesianGrid, ComposedChart, Line, ResponsiveContainer, Scatter, Tooltip, XAxis, YAxis } from 'recharts';
import { inr, longDate, reasonLabel, shortDate } from '../format';
import type { BacktestResult, PriceOut } from '../types';

const UP = '#1f8a68';
const DOWN = '#b23b3b';
const PATTERN_NAMES: Record<string, string> = { H: 'Hammer', '3WS': 'Three white soldiers', BE: 'Bullish engulfing', D: 'Doji' };

type Fill = { side: 'buy' | 'sell'; price: number; reason: string };

export type CandleRow = PriceOut & {
  range: [number, number];
  fills: Fill[];
  buyMark: number | null;
  sellMark: number | null;
  patternMark: number | null;
};

/** Candles plus the trade fills that happened on each candle (keyed by the same date as the trade table). */
export function buildCandleRows(result: BacktestResult): CandleRow[] {
  const fills = new Map<string, Fill[]>();
  const add = (date: string, f: Fill) => fills.set(date, [...(fills.get(date) ?? []), f]);
  for (const t of result.trades) {
    add(t.entry_date, { side: 'buy', price: t.entry_price, reason: t.entry_reason || 'Signal' });
    add(t.exit_date, { side: 'sell', price: t.exit_price, reason: reasonLabel(t.exit_reason) });
  }
  return result.prices.map((p) => {
    const f = fills.get(p.date) ?? [];
    return {
      ...p,
      range: [p.low, p.high],
      fills: f,
      buyMark: f.some((x) => x.side === 'buy') ? p.low : null,
      sellMark: f.some((x) => x.side === 'sell') ? p.high : null,
      patternMark: p.patterns?.length ? p.high : null,
    };
  });
}

type ShapeProps = { x?: number; y?: number; width?: number; height?: number; payload?: CandleRow };

function CandleShape({ x = 0, y = 0, width = 0, height = 0, payload }: ShapeProps) {
  if (!payload) return <g />;
  const { open, high, low, close } = payload;
  const span = high - low;
  const py = (v: number) => (span > 0 ? y + ((high - v) / span) * height : y);
  const up = close >= open;
  const color = up ? UP : DOWN;
  const cx = x + width / 2;
  const bw = Math.max(1, Math.min(width * 0.75, 14));
  const top = py(Math.max(open, close));
  const bodyH = Math.max(1, py(Math.min(open, close)) - top);
  return (
    <g className={up ? 'candle up' : 'candle down'} data-date={payload.date} data-cx={cx.toFixed(1)}>
      <line x1={cx} x2={cx} y1={y} y2={y + Math.max(height, 1)} stroke={color} strokeWidth={1} />
      <rect x={cx - bw / 2} y={top} width={bw} height={bodyH} fill={up ? '#fffdf8' : color} stroke={color} strokeWidth={1} />
    </g>
  );
}

type DotProps = { cx?: number; cy?: number; payload?: CandleRow };

function buyArrow({ cx, cy, payload }: DotProps) {
  if (cx == null || cy == null || payload?.buyMark == null) return <g />;
  const y = cy + 6;
  return <path className="mark-buy" data-date={payload.date} data-cx={cx.toFixed(1)} d={`M${cx},${y} l6,10 h-12 z`} fill="#0e6b52" stroke="#fffdf8" strokeWidth={1} />;
}

function sellArrow({ cx, cy, payload }: DotProps) {
  if (cx == null || cy == null || payload?.sellMark == null) return <g />;
  const y = cy - 6;
  return <path className="mark-sell" data-date={payload.date} data-cx={cx.toFixed(1)} d={`M${cx},${y} l6,-10 h-12 z`} fill="#a34720" stroke="#fffdf8" strokeWidth={1} />;
}

function patternLabel({ cx, cy, payload }: DotProps) {
  if (cx == null || cy == null || !payload?.patterns?.length) return <g />;
  const y = cy - (payload.sellMark != null ? 22 : 8);
  return (
    <text className="mark-pattern" x={cx} y={y} textAnchor="middle" fontSize={10} fontWeight={700} fill="#6b4fa3">
      {payload.patterns.join('·')}
    </text>
  );
}

function CandleTip({ active, payload }: { active?: boolean; payload?: Array<{ payload: CandleRow }> }) {
  if (!active || !payload?.length) return null;
  const r = payload[0].payload;
  return (
    <div className="tip">
      <p className="tip-date">{longDate(r.date)}</p>
      <p>
        O {inr(r.open)} · H {inr(r.high)}
      </p>
      <p>
        L {inr(r.low)} · C {inr(r.close)}
      </p>
      <p>Vol {Math.round(r.volume).toLocaleString('en-IN')}</p>
      {r.fills.map((f, k) => (
        <p key={k} className={f.side === 'buy' ? 'tip-buy' : 'tip-sell'}>
          {f.side === 'buy' ? '▲ Buy' : '▼ Sell'} {inr(f.price)} · {f.reason}
        </p>
      ))}
      {r.patterns?.length ? <p>Pattern: {r.patterns.map((c) => PATTERN_NAMES[c] ?? c).join(', ')}</p> : null}
    </div>
  );
}

const WINDOWS = [
  { label: '60', n: 60 },
  { label: '120', n: 120 },
  { label: '250', n: 250 },
  { label: 'All', n: Infinity },
];

/** Candlestick view of the price chart (body only; the card and toggle live in PriceChart). */
export function CandleView({ result }: { result: BacktestResult }) {
  const rows = useMemo(() => buildCandleRows(result), [result]);
  const n = rows.length;
  const initial = n > 160 ? 120 : Infinity;
  const [range, setRange] = useState<{ start: number; end: number }>(() => ({
    start: Math.max(0, n - Math.min(n, initial)),
    end: n - 1,
  }));
  const showSma = result.policy.require_price_above_sma_trend && result.policy.sma_trend != null;
  const showEma = result.policy.ema_fast != null && result.policy.ema_slow != null;
  const maxVol = useMemo(() => rows.reduce((m, r) => Math.max(m, r.volume), 0), [rows]);
  const visible = range.end - range.start + 1;
  // Price axis fitted to the visible candles (and averages), not to zero.
  const yDomain = useMemo(() => {
    let lo = Infinity;
    let hi = -Infinity;
    for (const r of rows.slice(range.start, range.end + 1)) {
      for (const v of [r.low, r.high, showEma ? r.ema_fast : null, showEma ? r.ema_slow : null, showSma ? r.sma_trend : null]) {
        if (v != null) {
          lo = Math.min(lo, v);
          hi = Math.max(hi, v);
        }
      }
    }
    if (!Number.isFinite(lo)) return ['auto', 'auto'] as [string, string];
    const pad = (hi - lo) * 0.08 || hi * 0.02;
    const raw = (hi - lo + 2.6 * pad) / 4;
    const mag = 10 ** Math.floor(Math.log10(raw));
    const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((v) => v >= raw) ?? raw;
    const a = Math.max(0, Math.floor((lo - pad * 1.6) / step) * step);
    const b = Math.ceil((hi + pad) / step) * step;
    return [a, b] as [number, number] | [string, string];
  }, [rows, range, showEma, showSma]);
  const period = result.policy.candle_period;
  const hasPatterns = rows.some((r) => r.patterns?.length);

  function show(count: number) {
    setRange({ start: Math.max(0, n - Math.min(n, count)), end: n - 1 });
  }

  return (
    <div className="candle-view">
      <p className="hint candle-note">
        {period} candles: {n} total, showing {visible}. Drag the handles under the chart to zoom and pan. ▲ buy and ▼ sell mark the fill candle.
      </p>
      <div className="range-btns" role="group" aria-label="Visible candles">
        {WINDOWS.filter((w) => w.n === Infinity || w.n < n).map((w) => (
          <button
            key={w.label}
            type="button"
            className={(w.n === Infinity ? visible === n : visible === w.n && range.end === n - 1) ? 'chip-btn active' : 'chip-btn'}
            onClick={() => show(w.n)}
          >
            {w.n === Infinity ? 'All' : `Last ${w.label}`}
          </button>
        ))}
      </div>
      <div className="candle-body">
        <ResponsiveContainer width="100%" height="100%">
          <ComposedChart data={rows} margin={{ top: 26, right: 8, left: 0, bottom: 0 }} barCategoryGap="20%">
            <CartesianGrid stroke="#e7dfd1" vertical={false} />
            <XAxis
              dataKey="date"
              tickFormatter={visible > 90 ? shortDate : (d: string) => longDate(d).replace(/ \d{4}$/, '')}
              minTickGap={24}
              tick={{ fill: '#6d655c', fontSize: 12 }}
              axisLine={false}
              tickLine={false}
            />
            <XAxis xAxisId="vol" dataKey="date" hide />
            <YAxis
              tickFormatter={(value: number) => inr(value, 0)}
              width={64}
              tick={{ fill: '#6d655c', fontSize: 12 }}
              axisLine={false}
              tickLine={false}
              domain={yDomain}
              allowDataOverflow
            />
            <YAxis yAxisId="vol" hide domain={[0, maxVol * 4 || 1]} />
            <Tooltip content={<CandleTip />} cursor={{ stroke: '#cdbfa8' }} />
            <Bar xAxisId="vol" yAxisId="vol" dataKey="volume" fill="#cdbfa8" opacity={0.45} isAnimationActive={false} />
            <Bar dataKey="range" shape={<CandleShape />} isAnimationActive={false} />
            {showEma ? (
              <Line type="monotone" dataKey="ema_fast" stroke="#1d6fbf" strokeWidth={1.2} dot={false} isAnimationActive={false} />
            ) : null}
            {showEma ? (
              <Line type="monotone" dataKey="ema_slow" stroke="#c45c26" strokeWidth={1.2} dot={false} isAnimationActive={false} />
            ) : null}
            {showSma ? (
              <Line type="monotone" dataKey="sma_trend" stroke="#0e6b52" strokeWidth={1.2} strokeDasharray="4 3" dot={false} isAnimationActive={false} />
            ) : null}
            <Scatter dataKey="buyMark" shape={buyArrow} isAnimationActive={false} />
            <Scatter dataKey="sellMark" shape={sellArrow} isAnimationActive={false} />
            {hasPatterns ? <Scatter dataKey="patternMark" shape={patternLabel} isAnimationActive={false} /> : null}
            <Brush
              dataKey="date"
              height={36}
              travellerWidth={16}
              stroke="#8a7d6b"
              fill="#f7f3eb"
              startIndex={range.start}
              endIndex={range.end}
              tickFormatter={shortDate}
              onChange={(r) => {
                if (r.startIndex != null && r.endIndex != null) setRange({ start: r.startIndex, end: r.endIndex });
              }}
            />
          </ComposedChart>
        </ResponsiveContainer>
      </div>
      <ul className="legend">
        <li><i className="swatch candle-up" /> Up candle</li>
        <li><i className="swatch candle-down" /> Down candle</li>
        {showEma ? <li><i className="swatch ema-fast" /> EMA {result.policy.ema_fast}</li> : null}
        {showEma ? <li><i className="swatch ema-slow" /> EMA {result.policy.ema_slow}</li> : null}
        {showSma ? <li><i className="swatch sma" /> SMA {result.policy.sma_trend}</li> : null}
        <li><span className="arrow-buy">▲</span> Buy</li>
        <li><span className="arrow-sell">▼</span> Sell</li>
        {hasPatterns ? <li><b className="pattern-key">H 3WS BE D</b> Hammer, 3 soldiers, engulfing, doji</li> : null}
        <li><i className="swatch vol" /> Volume</li>
      </ul>
    </div>
  );
}
