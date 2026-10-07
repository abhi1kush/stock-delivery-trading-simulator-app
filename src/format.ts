const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

const inrFmt = new Intl.NumberFormat('en-IN', {
  style: 'currency',
  currency: 'INR',
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

const inrWholeFmt = new Intl.NumberFormat('en-IN', {
  style: 'currency',
  currency: 'INR',
  minimumFractionDigits: 0,
  maximumFractionDigits: 0,
});

const sharesFmt = new Intl.NumberFormat('en-IN', { maximumFractionDigits: 0 });

export function inr(value: number, digits: 0 | 2 = 2): string {
  return digits === 0 ? inrWholeFmt.format(value) : inrFmt.format(value);
}

export function signedInr(value: number): string {
  const text = inr(value);
  return value > 0 ? `+${text}` : text;
}

export function signedPct(value: number, digits = 2): string {
  const text = value.toFixed(digits);
  return value > 0 ? `+${text}%` : `${text}%`;
}

export function shares(value: number): string {
  return sharesFmt.format(value);
}

export function longDate(iso: string): string {
  const [year, month, day] = iso.split('-');
  const name = MONTHS[Number(month) - 1] ?? month;
  return `${Number(day)} ${name} ${year}`;
}

export function shortDate(iso: string): string {
  const [year, month] = iso.split('-');
  const name = MONTHS[Number(month) - 1] ?? month;
  return `${name} '${year.slice(2)}`;
}

export function trimNum(value: number): string {
  if (Number.isInteger(value)) return String(value);
  return String(Math.round(value * 100) / 100);
}

const REASONS: Record<string, string> = {
  stop_loss: 'Stop loss',
  trailing_stop: 'Trailing stop',
  take_profit: 'Take profit',
  signal: 'Signal',
  end_of_data: 'End of data',
};

export function reasonLabel(reason: string): string {
  return REASONS[reason] ?? reason.replaceAll('_', ' ');
}

export function tone(value: number): 'positive' | 'negative' | 'flat' {
  if (value > 0) return 'positive';
  if (value < 0) return 'negative';
  return 'flat';
}

export function axisInr(value: number): string {
  const abs = Math.abs(value);
  const sign = value < 0 ? '−' : '';
  if (abs >= 100000) return `${sign}₹${(abs / 100000).toFixed(abs >= 1000000 ? 1 : 2)}L`;
  if (abs >= 1000) return `${sign}₹${(abs / 1000).toFixed(0)}k`;
  return `${sign}₹${abs.toFixed(0)}`;
}
