import { useEffect, useState } from 'react';
import { SetupView } from './components/SetupView';
import { ReportView } from './components/ReportView';
import { summarizeCsv } from './csv';
import { buildPolicyJson, defaultPolicyForm, validateForm } from './policy';
import type { BacktestResult, CsvMeta, PolicyForm } from './types';
import { errorText, preloadEngine, runBacktest } from './wasm/engine';

type View = 'setup' | 'report';
type EngineState = 'loading' | 'ready' | 'error';

export function App() {
  const [view, setView] = useState<View>('setup');
  const [engine, setEngine] = useState<EngineState>('loading');
  const [engineError, setEngineError] = useState<string | null>(null);
  const [form, setForm] = useState<PolicyForm>(defaultPolicyForm);
  const [yaml, setYaml] = useState('');
  const [csvText, setCsvText] = useState<string | null>(null);
  const [csvMeta, setCsvMeta] = useState<CsvMeta | null>(null);
  const [loadingSample, setLoadingSample] = useState(false);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<BacktestResult | null>(null);
  const [sourceName, setSourceName] = useState('Prices');

  useEffect(() => {
    let cancelled = false;
    preloadEngine()
      .then(() => {
        if (!cancelled) setEngine('ready');
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setEngine('error');
          setEngineError(errorText(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    function onPop() {
      setView('setup');
      window.scrollTo(0, 0);
    }
    window.addEventListener('popstate', onPop);
    return () => window.removeEventListener('popstate', onPop);
  }, []);

  function acceptCsv(text: string, name: string) {
    const summary = summarizeCsv(text, name);
    if ('error' in summary) {
      setError(summary.error);
      return;
    }
    setCsvText(text);
    setCsvMeta(summary.meta);
    setError(null);
  }

  async function loadSample() {
    setLoadingSample(true);
    setError(null);
    try {
      const response = await fetch('/data/CUPID.csv');
      if (!response.ok) throw new Error(`Could not load the CUPID sample (${response.status}).`);
      acceptCsv(await response.text(), 'CUPID.csv');
    } catch (err) {
      setError(errorText(err));
    } finally {
      setLoadingSample(false);
    }
  }

  function upload(file: File) {
    const reader = new FileReader();
    reader.onload = () => acceptCsv(String(reader.result ?? ''), file.name);
    reader.onerror = () => setError('Could not read that file.');
    reader.readAsText(file);
  }

  async function loadExampleYaml() {
    setError(null);
    try {
      const response = await fetch('/policy/example_policy.yaml');
      if (!response.ok) throw new Error('Could not load the example policy.');
      setYaml(await response.text());
    } catch (err) {
      setError(errorText(err));
    }
  }

  async function run() {
    if (!csvText || !csvMeta) {
      setError('Load a price CSV or the CUPID sample first.');
      return;
    }
    const yamlOn = yaml.trim().length > 0;
    if (!yamlOn) {
      const problem = validateForm(form);
      if (problem) {
        setError(problem);
        return;
      }
    }
    setRunning(true);
    setError(null);
    try {
      const output = await runBacktest(csvText, yamlOn ? yaml : buildPolicyJson(form));
      setResult(output);
      setSourceName(csvMeta.name);
      window.history.pushState({ view: 'report' }, '');
      setView('report');
      window.scrollTo(0, 0);
    } catch (err) {
      setError(errorText(err));
    } finally {
      setRunning(false);
    }
  }

  function showReport() {
    if (!result) return;
    window.history.pushState({ view: 'report' }, '');
    setView('report');
    window.scrollTo(0, 0);
  }

  function showSetup() {
    if (window.history.state && (window.history.state as { view?: string }).view === 'report') {
      window.history.back();
      return;
    }
    setView('setup');
    window.scrollTo(0, 0);
  }

  const visibleError = engine === 'error' ? engineError ?? 'The WASM engine failed to load.' : error;

  return (
    <div className="shell">
      <header className="top">
        <div className="brand">
          <span className="mark" aria-hidden="true">
            <svg viewBox="0 0 32 32">
              <rect x="4" y="16" width="6" height="12" rx="1.2" />
              <rect x="13" y="9" width="6" height="19" rx="1.2" />
              <rect x="22" y="4" width="6" height="24" rx="1.2" />
            </svg>
          </span>
          <div>
            <p className="eyebrow">Long-only · next-open fills</p>
            <h1>Delivery Backtester</h1>
          </div>
        </div>
        <p className={`engine engine-${engine}`} role="status">
          {engine === 'loading' && 'Loading WASM engine…'}
          {engine === 'ready' && 'Engine ready'}
          {engine === 'error' && 'Engine failed to load'}
        </p>
      </header>

      <main>
        {view === 'report' && result ? (
          <ReportView result={result} sourceName={sourceName} onBack={showSetup} />
        ) : (
          <SetupView
            form={form}
            onChange={setForm}
            yaml={yaml}
            onYaml={setYaml}
            csvMeta={csvMeta}
            loadingSample={loadingSample}
            onLoadSample={() => void loadSample()}
            onUpload={upload}
            onLoadExampleYaml={() => void loadExampleYaml()}
            onSubmit={() => void run()}
            running={running}
            engine={engine}
            error={visibleError}
            hasReport={result != null}
            onShowReport={showReport}
          />
        )}
      </main>

      <footer>
        <p>
          On each bar, indicators and the decision use only data through that close. A signal fills at the next
          bar&apos;s open. Stops can arm from that bar&apos;s high and low, then fill the following session. Research
          tool, not advice.
        </p>
      </footer>
    </div>
  );
}
