import { useCallback, useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { HELP } from '../help';

const MARGIN = 8;

/**
 * Small "i" button with a tooltip. Hover or keyboard focus opens it on desktop;
 * tap toggles it on touch. Esc or a tap outside closes it. The tooltip is
 * position: fixed and clamped/flipped to stay inside the viewport.
 */
export function InfoTip({ id, label }: { id: string; label?: string }) {
  const text = HELP[id];
  const tipId = `${useId().replace(/:/g, '')}-tip`;
  const [open, setOpen] = useState(false);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  const btn = useRef<HTMLButtonElement>(null);
  const tip = useRef<HTMLSpanElement>(null);
  const pinned = useRef(false);

  const close = useCallback(() => {
    pinned.current = false;
    setOpen(false);
  }, []);

  const place = useCallback(() => {
    if (!btn.current || !tip.current) return;
    const b = btn.current.getBoundingClientRect();
    const t = tip.current.getBoundingClientRect();
    const vw = document.documentElement.clientWidth;
    const vh = window.innerHeight;
    let left = b.left + b.width / 2 - t.width / 2;
    left = Math.max(MARGIN, Math.min(left, vw - t.width - MARGIN));
    let top = b.bottom + 6;
    if (top + t.height > vh - MARGIN && b.top - t.height - 6 >= MARGIN) top = b.top - t.height - 6;
    setPos({ left, top });
  }, []);

  useLayoutEffect(() => {
    if (open) place();
  }, [open, place]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    const onDown = (e: PointerEvent) => {
      const target = e.target as Node;
      if (!btn.current?.contains(target) && !tip.current?.contains(target)) close();
    };
    // Keep the tooltip next to its button while the page scrolls or resizes.
    const onScroll = () => place();
    document.addEventListener('keydown', onKey);
    document.addEventListener('pointerdown', onDown);
    window.addEventListener('scroll', onScroll, { passive: true });
    window.addEventListener('resize', onScroll);
    return () => {
      document.removeEventListener('keydown', onKey);
      document.removeEventListener('pointerdown', onDown);
      window.removeEventListener('scroll', onScroll);
      window.removeEventListener('resize', onScroll);
    };
  }, [open, close, place]);

  if (!text) return null;

  return (
    <span className="infotip">
      <button
        ref={btn}
        type="button"
        className="infotip-btn"
        aria-label={`About ${label ?? 'this setting'}`}
        aria-describedby={tipId}
        aria-expanded={open}
        onMouseEnter={(e) => {
          if (e.nativeEvent instanceof PointerEvent && e.nativeEvent.pointerType === 'touch') return;
          setOpen(true);
        }}
        onMouseLeave={() => {
          if (!pinned.current) setOpen(false);
        }}
        onFocus={() => setOpen(true)}
        onBlur={() => {
          if (!pinned.current) setOpen(false);
        }}
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          pinned.current = !pinned.current || !open;
          setOpen(pinned.current);
        }}
      >
        i
      </button>
      <span
        ref={tip}
        id={tipId}
        role="tooltip"
        className={open ? 'infotip-pop open' : 'infotip-pop'}
        style={open && pos ? { left: pos.left, top: pos.top } : undefined}
      >
        {text}
      </span>
    </span>
  );
}
