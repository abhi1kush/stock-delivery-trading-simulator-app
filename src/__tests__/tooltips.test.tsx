// @vitest-environment jsdom
import { afterEach, describe, expect, it } from 'vitest';
import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { SetupView } from '../components/SetupView';
import { defaultPolicyForm } from '../policy';
import { HELP, HELP_IDS } from '../help';

afterEach(cleanup);

function renderSetup(form = { ...defaultPolicyForm, candlePeriod: 'ND' as const }) {
  return render(
    <SetupView
      form={form}
      onChange={() => {}}
      yaml=""
      onYaml={() => {}}
      csvMeta={{ name: 'x.csv', rows: 246, start: '2025-01-01', end: '2025-12-31', medianGapDays: 1 }}
      loadingSample={false}
      onLoadSample={() => {}}
      sampleFile="CUPID.csv"
      onSampleFile={() => {}}
      onUpload={() => {}}
      onLoadExampleYaml={() => {}}
      onSubmit={() => {}}
      running={false}
      engine="ready"
      error={null}
      hasReport={false}
      onShowReport={() => {}}
    />,
  );
}

describe('policy parameter tooltips', () => {
  it('every help text is 1–3 sentences and non-trivial', () => {
    for (const id of HELP_IDS) {
      const text = HELP[id];
      expect(text.length, id).toBeGreaterThan(40);
      const sentences = text.split(/(?<=[.!?])\s+(?=[A-Z])/).length;
      expect(sentences, id).toBeLessThanOrEqual(3);
    }
  });

  it('every policy control in the form has a tooltip, and every tooltip is used', () => {
    const { container } = renderSetup();
    const controls = [...container.querySelectorAll('input[id], select[id], textarea[id]')]
      .map((el) => el.id)
      .filter((id) => id);
    expect(controls.length).toBeGreaterThan(20);
    for (const id of controls) expect(HELP[id], `missing help for #${id}`).toBeTruthy();
    const tips = container.querySelectorAll('[role="tooltip"]');
    expect(tips.length).toBe(HELP_IDS.length);
    const texts = new Set([...tips].map((t) => t.textContent));
    for (const id of HELP_IDS) expect(texts.has(HELP[id]), id).toBe(true);
  });

  it('buttons are described by their tooltip', () => {
    renderSetup();
    for (const btn of screen.getAllByRole('button', { name: /^About / })) {
      const tip = document.getElementById(btn.getAttribute('aria-describedby') ?? '');
      expect(tip?.getAttribute('role')).toBe('tooltip');
    }
  });

  it('opens on focus, closes on Esc, toggles on tap and closes on outside tap', () => {
    renderSetup();
    const btn = screen.getByRole('button', { name: 'About Trailing stop (%)' });
    const tip = document.getElementById(btn.getAttribute('aria-describedby')!)!;
    expect(tip.className).not.toContain('open');
    act(() => btn.focus());
    expect(tip.className).toContain('open');
    expect(tip.textContent).toContain('buy 100, peak 150, 10% gives a stop at 135');
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(tip.className).not.toContain('open');
    act(() => btn.blur());
    fireEvent.click(btn);
    expect(tip.className).toContain('open');
    fireEvent.pointerDown(document.body);
    expect(tip.className).not.toContain('open');
    fireEvent.click(btn);
    fireEvent.click(btn);
    expect(tip.className).not.toContain('open');
  });
});
