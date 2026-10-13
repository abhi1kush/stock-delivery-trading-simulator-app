/** Bundled sample datasets (real daily NSE prices). Files live in public/data/. */
export type Sample = {
  file: string;
  name: string;
  symbol: string;
  range: string;
  category: string;
  description: string;
};

export const SAMPLES: Sample[] = [
  {
    file: 'CUPID.csv',
    name: 'Cupid',
    symbol: 'CUPID',
    range: 'Oct 2025 – Oct 2026',
    category: 'Original sample',
    description: 'The default dataset the policy was tuned on.',
  },
  {
    file: 'PAYTM.csv',
    name: 'One 97 Communications (Paytm)',
    symbol: 'PAYTM',
    range: 'Nov 2021 – Nov 2022',
    category: 'One-year bear run',
    description: 'From the IPO listing: ₹1,561 → ₹482 (−69%), 76% max drawdown.',
  },
  {
    file: 'SUZLON.csv',
    name: 'Suzlon Energy',
    symbol: 'SUZLON',
    range: 'Jan 2021 – Dec 2022',
    category: 'Choppy, inconsistent swings',
    description: 'Repeated rallies and slumps (61% annualised volatility, 52% drawdown) with little net direction per move.',
  },
  {
    file: 'TRENT.csv',
    name: 'Trent',
    symbol: 'TRENT',
    range: 'Jan 2023 – Jun 2024',
    category: 'Strong steady bull run',
    description: '₹1,342 → ₹5,480 (+308%) with a max drawdown of only 13%.',
  },
  {
    file: 'ADANIENT.csv',
    name: 'Adani Enterprises',
    symbol: 'ADANIENT',
    range: 'Oct 2022 – Jan 2024',
    category: 'V-shaped crash and recovery',
    description: 'Hindenburg report crash: −71% from the Dec 2022 peak to Feb 2023, then +163% back to the starting level.',
  },
  {
    file: 'HINDUNILVR.csv',
    name: 'Hindustan Unilever',
    symbol: 'HINDUNILVR',
    range: 'Jun 2022 – Jun 2024',
    category: 'Range-bound blue chip',
    description: 'Low volatility (19% annualised), +7% over two years inside a ₹2,110–₹2,757 band.',
  },
];

export const SAMPLE_SOURCE_NOTE =
  'Real daily NSE prices from Yahoo Finance (chart API, split/bonus-adjusted, not dividend-adjusted), fetched 13 Oct 2026. For research and education only; Yahoo Finance and NSE terms apply, do not redistribute.';
