import type { PolicyForm, CsvMeta } from '../types';
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

  function set<K extends keyof PolicyForm>(key: K, value: PolicyForm[K]) {
    onChange({ ...form, [key]: value });
  }

  return (
    <form
      className="stack"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <section className="card">
        <div className="card-head">
          <h2>Prices</h2>
          <p>Daily bars with the same columns as the bundled CUPID file.</p>
        </div>
        <div className="actions">
          <button type="button" className="btn primary" onClick={onLoadSample} disabled={loadingSample}>
            {loadingSample ? 'Loading sample…' : 'Load CUPID sample'}
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
          <legend>Capital</legend>
          <div className="fields">
            <Field
              id="starting-cash"
              label="Starting capital (₹)"
              value={form.startingCash}
              min="1"
              step="1000"
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
            <Field id="ema-fast" label="EMA fast" value={form.emaFast} min="1" step="1" onChange={(value) => set('emaFast', value)} />
            <Field id="ema-slow" label="EMA slow" value={form.emaSlow} min="2" step="1" onChange={(value) => set('emaSlow', value)} />
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
            <Field
              id="sma-trend"
              label="SMA trend length"
              value={form.smaTrend}
              min="1"
              step="1"
              onChange={(value) => set('smaTrend', value)}
            />
            <label className="check" htmlFor="sma-filter">
              <input
                id="sma-filter"
                type="checkbox"
                checked={form.requirePriceAboveSma}
                onChange={(event) => set('requirePriceAboveSma', event.target.checked)}
              />
              <span>Only buy when close is above the SMA</span>
            </label>
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
              step="0.1"
              hint="Initial stop below the entry price."
              onChange={(value) => set('stopLossPct', value)}
            />
            <Field
              id="trail-pct"
              label="Trailing stop (%)"
              value={form.trailingStopPct}
              min="0.1"
              step="0.1"
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
          <label className="yaml-label" htmlFor="policy-yaml">
            Policy YAML
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
      <span>{label}</span>
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
