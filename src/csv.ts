import type { CsvMeta } from './types';

const REQUIRED = ['date', 'open', 'high', 'low', 'close', 'volume'];

export function summarizeCsv(text: string, name: string): { meta: CsvMeta } | { error: string } {
  const cleaned = text.replace(/^\uFEFF/, '').trim();
  if (!cleaned) return { error: 'That CSV is empty.' };

  const lines = cleaned.split(/\r?\n/).filter((line) => line.trim().length > 0);
  if (lines.length < 2) {
    return { error: 'The CSV needs a header and at least one daily bar.' };
  }

  const header = lines[0].split(',').map((cell) => cell.trim().toLowerCase());
  const missing = REQUIRED.filter((column) => !header.includes(column));
  if (missing.length > 0) {
    const noun = missing.length > 1 ? 'columns' : 'column';
    return {
      error: `CSV is missing ${noun}: ${missing.join(', ')}. Expected date, open, high, low, close, volume.`,
    };
  }

  const dateIdx = header.indexOf('date');
  const dates = lines
    .slice(1)
    .map((line) => line.split(',')[dateIdx]?.trim() ?? '')
    .filter((date) => date.length > 0)
    .sort();

  if (dates.length === 0) return { error: 'No dates found in the CSV.' };

  return {
    meta: {
      name,
      rows: lines.length - 1,
      start: dates[0],
      end: dates[dates.length - 1],
    },
  };
}
