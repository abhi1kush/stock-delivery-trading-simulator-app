import type { CandleChoice, PolicyForm, CsvMeta } from '../types';
import { isDailyGranularity } from '../csv';
import { InfoTip } from './InfoTip';
import { MAX_CANDLE_DAYS, formFactor, withCandlePeriod, withLength } from '../policy';
import { scaleLengths, warmupWarning, type LengthKey } from '../lengths';
import { SAMPLES, SAMPLE_SOURCE_NOTE } from '../samples';
import { longDate } from '../format';

type EngineState = 'loading' | 'ready' | 'error';

type Props = {
  form: PolicyForm;
  onChange: (next: PolicyForm) => void;
  yaml: string;
  onYaml: (next: string) => void;
  csvMeta: CsvMeta | null;
  loadingSample: boolean;
  onLoadSample: () => void;
  sampleFile: string;
  onSampleFile: (file: string) => void;
  onUpload: (file: File) => void;
  onLoadExampleYaml: () => void;
  onSubmit: () => void;
  running: boolean;
  engine: EngineState;
  error: string | null;
  hasReport: boolean;
  onShowReport: () => void;
};

export function SetupView({
  form,
  onChange,
  yaml,
  onYaml,
  csvMeta,
  loadingSample,
  onLoadSample,
  sampleFile,
  onSampleFile,
  onUpload,
  onLoadExampleYaml,
  onSubmit,
  running,
  engine,
  error,
  hasReport,
  onShowReport,
}: Props) {
  const yamlActive = yaml.trim().length > 0;
  const daily = isDailyGranularity(csvMeta?.medianGapDays ?? null);
  const factor = formFactor(form);
  const warmup = csvMeta ? warmupWarning(csvMeta.rows, factor, scaleLengths(form.daily, factor)) : null;

  function lenField(id: string, k: LengthKey, label: string) {
    return (
      <Field
        id={id}
        label={label}
        value={form[k]}
        min="1"
        step="1"
        hint={factor === 1 ? undefined : `≈${form.daily[k]} trading days`}
        key={id}
        onChange={(value) => onChange(withLength(form, k, value))}
      />
    );
  }


  function set<K extends keyof PolicyForm>(key: K, value: PolicyForm[K]) {
    onChange({ ...form, [key]: value });
  }

  return (
    <form
      className="stack"
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <section className="card">
        <div className="card-head">
          <h2>Prices</h2>
          <p>Pick a bundled real-price sample or upload daily bars with the same columns.</p>
        </div>
        <label className="field" htmlFor="sample-picker">
          <span className="label-row">
            <span>Sample data</span>
            <InfoTip id="sample-picker" label="Sample data" />
          </span>
          <select id="sample-picker" value={sampleFile} onChange={(event) => onSampleFile(event.target.value)}>
            {SAMPLES.map((s) => (
              <option key={s.file} value={s.file}>
                {s.name} ({s.symbol}) · {s.range} · {s.category}
              </option>
            ))}
          </select>
          <small>{SAMPLES.find((s) => s.file === sampleFile)?.description}</small>
        </label>
        <div className="actions">
          <button type="button" className="btn primary" onClick={onLoadSample} disabled={loadingSample}>
            {loadingSample ? 'Loading sample…' : `Load ${SAMPLES.find((s) => s.file === sampleFile)?.symbol ?? ''} sample`}
          </button>
          <label className="btn ghost">
            Upload CSV
            <input
              type="file"
              accept=".csv,text/csv"
              onChange={(event) => {
                const file = event.target.files?.[0];
                event.target.value = '';
                if (file) onUpload(file);
              }}
            />
          </label>
        </div>
        <p className="hint">Columns: date, open, high, low, close, volume. Dates as YYYY-MM-DD.</p>
        <p className="hint source-note">{SAMPLE_SOURCE_NOTE}</p>
        <div className={csvMeta ? 'file-status ready' : 'file-status'}>
          {csvMeta ? (
            <>
              <strong>{csvMeta.name}</strong>
              <span>
                {csvMeta.rows} bars · {longDate(csvMeta.start)} – {longDate(csvMeta.end)}
              </span>
            </>
          ) : (
            <span>No prices loaded yet.</span>
          )}
        </div>
      </section>

      <section className={`card ${yamlActive ? 'dimmed' : ''}`}>
        <div className="card-head">
          <h2>Policy</h2>
          <p>Defaults match example_policy.yaml: EMA 12/26, ADX confirm, RSI cap, SMA 100 filter, 8% stop, 10% trail.</p>
        </div>

        <fieldset>
          <legend>Candles</legend>
          <div className="fields">
            <label className="field" htmlFor="candle-period">
              <span className="label-row">
                <span>Candle period</span>
                <InfoTip id="candle-period" label="Candle period" />
              </span>
              <select
                id="candle-period"
                value={form.candlePeriod}
                onChange={(event) => onChange(withCandlePeriod(form, event.target.value as CandleChoice))}
              >
                <option value="1D">1 day (input rows)</option>
                <option value="ND" disabled={!daily}>N trading days</option>
                <option value="1W" disabled={!daily}>1 week (calendar)</option>
                <option value="1M" disabled={!daily}>1 month (calendar)</option>
              </select>
              <small>
                {daily
                  ? 'Daily rows are grouped into candles first: open = first, high = max, low = min, close = last, volume = sum. Indicators, signals and patterns use the grouped candles.'
                  : `These prices are about ${csvMeta?.medianGapDays} days apart, so only the input period is available.`}
              </small>
            </label>
            {form.candlePeriod === 'ND' ? (
              <Field
                id="candle-days"
                label="Trading days per candle"
                value={form.candleDays}
                min="1"
                max={String(MAX_CANDLE_DAYS)}
                step="1"
                hint={`Whole number, 1 to ${MAX_CANDLE_DAYS}.`}
                onChange={(value) => onChange(withCandlePeriod(form, 'ND', value))}
              />
            ) : null}
          </div>
          {factor !== 1 ? (
            <p className="hint scale-note">
              Candle-count lengths (EMA, SMA, RSI, ADX, downtrend lookback) were rescaled from their daily values at ≈{factor} trading days per candle. Stop and trail percentages are unchanged.
            </p>
          ) : null}
          {warmup ? <p className="banner warmup">{warmup}</p> : null}
        </fieldset>

        <fieldset>
          <legend>Capital</legend>
          <div className="fields">
            <Field
              id="starting-cash"
              label="Starting capital (₹)"
              value={form.startingCash}
              min="1"
              step="any"
              onChange={(value) => set('startingCash', value)}
            />
            <Field
              id="position-pct"
              label="Position size (%)"
              value={form.positionPct}
              min="1"
              max="100"
              step="1"
              hint="Share of cash used on each buy."
              onChange={(value) => set('positionPct', value)}
            />
          </div>
        </fieldset>

        <fieldset>
          <legend>Entry</legend>
          <div className="fields">
            {lenField('ema-fast', 'emaFast', 'EMA fast')}
            {lenField('ema-slow', 'emaSlow', 'EMA slow')}
            <Field
              id="adx-min"
              label="ADX threshold"
              value={form.adxMin}
              min="0"
              step="1"
              hint="Buy when ADX crosses up through this level while the fast average is above the slow one."
              onChange={(value) => set('adxMin', value)}
            />
            <Field
              id="rsi-max"
              label="RSI max"
              value={form.rsiMax}
              min="1"
              max="100"
              step="1"
              hint="Skip buys when RSI is at or above this."
              onChange={(value) => set('rsiMax', value)}
            />
            {lenField('sma-trend', 'smaTrend', 'SMA trend length')}
            {lenField('rsi-period', 'rsiPeriod', 'RSI length')}
            {lenField('adx-period', 'adxPeriod', 'ADX length')}
            <label className="check" htmlFor="sma-filter">
              <input
                id="sma-filter"
                type="checkbox"
                checked={form.requirePriceAboveSma}
                onChange={(event) => set('requirePriceAboveSma', event.target.checked)}
              />
              <span>Only buy when close is above the SMA</span>
              <InfoTip id="sma-filter" label="SMA trend filter" />
            </label>
          </div>
        </fieldset>

        

        <fieldset>
          <legend>Candlestick patterns &amp; buy score</legend>
          <p className="hint">
            Buy score = signal weight (when an EMA/ADX entry signal fires) + each detected pattern&apos;s weight ×
            strength (0.5–1, how cleanly the candle fits). Buy when the score reaches the threshold and the RSI / SMA
            filters pass. With patterns off this is the original policy.
          </p>
          <div className="fields">
            {PATTERNS.map(([flag, weight, label, hint]) => (
              <div className="pattern-row" key={flag}>
                <label className="check" htmlFor={`pat-${flag}`}>
                  <input
                    id={`pat-${flag}`}
                    type="checkbox"
                    checked={form[flag]}
                    onChange={(event) => set(flag, event.target.checked)}
                  />
                  <span>{label}</span>
                  <InfoTip id={`pat-${flag}`} label={label} />
                </label>
                <Field
                  id={`w-${flag}`}
                  label={`${label} weight`}
                  value={form[weight]}
                  min="0"
                  max="1"
                  step="0.05"
                  hint={hint}
                  onChange={(value) => set(weight, value)}
                />
              </div>
            ))}
            <Field
              id="signal-weight"
              label="Signal weight"
              value={form.signalWeight}
              min="0"
              max="1"
              step="0.05"
              hint="Score added when the EMA cross or ADX confirm fires."
              onChange={(value) => set('signalWeight', value)}
            />
            <Field
              id="buy-threshold"
              label="Buy score threshold"
              value={form.buyThreshold}
              min="0.05"
              max="5"
              step="0.05"
              hint="Arm a buy when the score is at least this."
              onChange={(value) => set('buyThreshold', value)}
            />
            {lenField('downtrend-lookback', 'downtrendLookback', 'Downtrend lookback (candles)')}
          </div>
        </fieldset>

        <fieldset>
          <legend>Risk</legend>
          <div className="fields">
            <Field
              id="stop-pct"
              label="Stop loss (%)"
              value={form.stopLossPct}
              min="0.1"
              step="any"
              hint="Initial stop below the entry price."
              onChange={(value) => set('stopLossPct', value)}
            />
            <Field
              id="trail-pct"
              label="Trailing stop (%)"
              value={form.trailingStopPct}
              min="0.1"
              step="any"
              hint="Distance below the peak high. ATR stops stay off unless YAML sets them."
              onChange={(value) => set('trailingStopPct', value)}
            />
          </div>
        </fieldset>
      </section>

      <section className="card">
        <details className="advanced">
          <summary>Advanced YAML</summary>
          <p className="hint">
            A non-empty paste replaces the form. It uses the same fields as the engine YAML, including ATR stops.
          </p>
          {yamlActive ? <p className="banner">YAML override is on. Clear it to use the form again.</p> : null}
          <div className="actions">
            <button type="button" className="btn ghost" onClick={onLoadExampleYaml}>
              Load example YAML
            </button>
            <button type="button" className="btn ghost" onClick={() => onYaml('')} disabled={!yamlActive}>
              Clear
            </button>
          </div>
          <label className="yaml-label label-row" htmlFor="policy-yaml">
            Policy YAML <InfoTip id="policy-yaml" label="Policy YAML" />
          </label>
          <textarea
            id="policy-yaml"
            value={yaml}
            spellCheck={false}
            placeholder={'# paste a policy\ncapital:\n  starting_cash: 100000'}
            onChange={(event) => onYaml(event.target.value)}
          />
        </details>
      </section>

      <div className="runbar">
        {error ? (
          <p className="alert" role="alert">
            {error}
          </p>
        ) : null}
        <button className="btn run" type="submit" disabled={engine !== 'ready' || running}>
          {running ? 'Running…' : engine === 'loading' ? 'Loading engine…' : 'Run backtest'}
        </button>
        {hasReport ? (
          <button type="button" className="text-btn" onClick={onShowReport}>
            View last report
          </button>
        ) : null}
      </div>
    </form>
  );
}

type BoolKey = 'hammer' | 'threeWhiteSoldiers' | 'bullishEngulfing' | 'doji';
type WeightKey = 'hammerWeight' | 'threeWhiteSoldiersWeight' | 'bullishEngulfingWeight' | 'dojiWeight';

const PATTERNS: [BoolKey, WeightKey, string, string][] = [
  ['hammer', 'hammerWeight', 'Hammer', 'Small body near the top, lower shadow ≥ 2× body, after a 5-candle decline.'],
  ['threeWhiteSoldiers', 'threeWhiteSoldiersWeight', 'Three white soldiers', '3 rising bullish candles, each opening inside the prior body, small upper wicks.'],
  ['bullishEngulfing', 'bullishEngulfingWeight', 'Bullish engulfing', 'Bullish body engulfs the prior bearish body.'],
  ['doji', 'dojiWeight', 'Doji', 'Body ≤ 10% of range. Indecision, so only counted after a 5-candle decline.'],
];

function Field({
  id,
  label,
  value,
  onChange,
  hint,
  min,
  max,
  step,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  hint?: string;
  min?: string;
  max?: string;
  step?: string;
}) {
  return (
    <label className="field" htmlFor={id}>
      <span className="label-row">
        <span>{label}</span>
        <InfoTip id={id} label={label} />
      </span>
      <input
        id={id}
        inputMode="decimal"
        type="number"
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={(event) => onChange(event.target.value)}
      />
      {hint ? <small>{hint}</small> : null}
    </label>
  );
}
