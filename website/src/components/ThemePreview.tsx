import { useEffect, useRef, useState } from 'react';

interface ThemeColors {
  bg: string;
  cardBg: string;
  text: string;
  muted: string;
  dim: string;
  accent: string;
  accentSecondary: string;
  border: string;
  tagBg: string;
  tagBorder: string;
}

interface ThemeDef {
  id: string;
  name: string;
  tagline: string;
  colors: ThemeColors;
  font?: string;
  tight?: boolean;
  headlinesOnly?: boolean;
}

const themes: ThemeDef[] = [
  {
    id: 'default',
    name: 'Default',
    tagline: 'Dark theme with purple accents',
    colors: {
      bg: '#09090b', cardBg: '#18181b', text: '#fafafa', muted: '#a1a1aa', dim: '#71717a',
      accent: '#6d5aff', accentSecondary: '#22d3ee', border: '#27272a',
      tagBg: 'rgba(109,90,255,0.1)', tagBorder: 'rgba(109,90,255,0.25)',
    },
  },
  {
    id: 'daltonian',
    name: 'Daltonian',
    tagline: 'Colorblind-safe palette',
    colors: {
      bg: '#0a0a0f', cardBg: '#151520', text: '#e8e8f0', muted: '#9898a8', dim: '#686878',
      accent: '#4cc9f0', accentSecondary: '#f77f00', border: '#252535',
      tagBg: 'rgba(76,201,240,0.1)', tagBorder: 'rgba(76,201,240,0.25)',
    },
  },
  {
    id: 'minimal',
    name: 'Minimal',
    tagline: 'Headlines only, zero noise',
    colors: {
      bg: '#0a0a0a', cardBg: '#111111', text: '#d4d4d4', muted: '#888888', dim: '#666666',
      accent: '#999999', accentSecondary: '#999999', border: '#222222',
      tagBg: 'rgba(153,153,153,0.1)', tagBorder: 'rgba(153,153,153,0.2)',
    },
    font: "'JetBrains Mono', monospace",
    tight: true,
    headlinesOnly: true,
  },
];

const items = [
  { topic: 'Rust', headline: 'Async closures ship in Rust 1.94', meta: 'blog.rust-lang.org · 4 min', summary: 'Async closures hit stable, eliminating a major pain point.' },
  { topic: 'AI', headline: 'MCP Apps spec reaches v1.0', meta: 'modelcontextprotocol.io · 3 min', summary: 'The interactive UI extension for MCP is now finalized.' },
];

export default function ThemePreview() {
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observer = new IntersectionObserver(
      ([entry]) => { if (entry.isIntersecting) setVisible(true); },
      { threshold: 0.15 },
    );
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section className="py-28 relative z-[1]" ref={ref}>
      <div className="max-w-[1200px] mx-auto px-8">
        {/* Header */}
        <div className="text-center mb-16">
          <span className="inline-block font-mono text-xs font-medium text-[var(--color-accent)] uppercase tracking-widest mb-4">
            Themes
          </span>
          <h2 className="text-[clamp(2rem,5vw,3.2rem)] font-extrabold tracking-tight leading-tight mb-4">
            Your digest,{' '}
            <span className="bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-secondary)] bg-clip-text text-transparent">
              your style
            </span>
          </h2>
          <p className="text-lg text-[var(--color-text-muted)] max-w-[520px] mx-auto">
            Choose from 5 preset themes or describe your own with natural language.
          </p>
        </div>

        {/* Theme cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {themes.map((theme, i) => (
            <div
              key={theme.id}
              className={`transition-all duration-700 ease-out ${
                visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-6'
              }`}
              style={{ transitionDelay: `${i * 120}ms` }}
            >
              <div
                className="rounded-2xl border overflow-hidden h-full"
                style={{
                  backgroundColor: theme.colors.bg,
                  borderColor: theme.colors.border,
                  fontFamily: theme.font || "'Inter', sans-serif",
                  color: theme.colors.text,
                }}
              >
                {/* Card header */}
                <div
                  className="px-4 py-3 flex items-center justify-between border-b"
                  style={{ borderColor: theme.colors.border }}
                >
                  <span className="font-semibold text-xs" style={{ color: theme.colors.accent }}>
                    {theme.name}
                  </span>
                  <span className="text-[0.6rem]" style={{ color: theme.colors.dim }}>
                    {theme.tagline}
                  </span>
                </div>

                {/* Items */}
                <div className={`px-4 ${theme.tight ? 'py-2' : 'py-3'} space-y-${theme.tight ? '1' : '2'}`}>
                  {items.map((item, j) => (
                    <div key={j} className={j < items.length - 1 ? 'pb-2 border-b' : ''} style={{ borderColor: theme.colors.border }}>
                      <div className="flex items-start gap-1.5 mb-0.5">
                        <span
                          className="inline-block text-[0.55rem] font-semibold uppercase tracking-wider px-1.5 py-0.5 rounded-full shrink-0"
                          style={{
                            color: j === 0 ? theme.colors.accent : theme.colors.accentSecondary,
                            backgroundColor: j === 0 ? theme.colors.tagBg : `${theme.colors.accentSecondary}15`,
                            border: `1px solid ${j === 0 ? theme.colors.tagBorder : `${theme.colors.accentSecondary}40`}`,
                          }}
                        >
                          {item.topic}
                        </span>
                      </div>
                      <p className={`font-semibold ${theme.tight ? 'text-[0.7rem]' : 'text-xs'} leading-snug`}>
                        {item.headline}
                      </p>
                      <p className="text-[0.6rem] mt-0.5" style={{ color: theme.colors.dim }}>
                        {item.meta}
                      </p>
                      {!theme.headlinesOnly && (
                        <p className="text-[0.65rem] mt-1 leading-relaxed" style={{ color: theme.colors.muted }}>
                          {item.summary}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div
          className={`text-center mt-10 transition-all duration-700 ease-out ${
            visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
          }`}
          style={{ transitionDelay: '400ms' }}
        >
          <p className="text-sm text-[var(--color-text-muted)] mb-4">
            5 presets available. Or just tell Claude what you want.
          </p>
          <a
            href="/themes"
            className="inline-flex items-center gap-1 text-sm font-medium text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] transition-colors"
          >
            Explore all themes &rarr;
          </a>
        </div>
      </div>
    </section>
  );
}
