import { useState } from 'react';

interface ThemeDef {
  id: string;
  name: string;
  tagline: string;
  colors: {
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
  };
  font?: string;
  tight?: boolean;
  headlinesOnly?: boolean;
}

const themes: ThemeDef[] = [
  {
    id: 'default',
    name: 'Default',
    tagline: 'Perfect for late-night reading.',
    colors: {
      bg: '#09090b', cardBg: '#18181b', text: '#fafafa', muted: '#a1a1aa', dim: '#71717a',
      accent: '#6d5aff', accentSecondary: '#22d3ee', border: '#27272a',
      tagBg: 'rgba(109,90,255,0.1)', tagBorder: 'rgba(109,90,255,0.25)',
    },
  },
  {
    id: 'light',
    name: 'Light',
    tagline: 'Ideal for daytime use.',
    colors: {
      bg: '#ffffff', cardBg: '#f4f4f5', text: '#09090b', muted: '#52525b', dim: '#71717a',
      accent: '#6d5aff', accentSecondary: '#0891b2', border: '#e4e4e7',
      tagBg: 'rgba(109,90,255,0.08)', tagBorder: 'rgba(109,90,255,0.2)',
    },
  },
  {
    id: 'daltonian',
    name: 'Daltonian',
    tagline: 'Accessible to everyone.',
    colors: {
      bg: '#0a0a0f', cardBg: '#151520', text: '#e8e8f0', muted: '#9898a8', dim: '#686878',
      accent: '#4cc9f0', accentSecondary: '#f77f00', border: '#252535',
      tagBg: 'rgba(76,201,240,0.1)', tagBorder: 'rgba(76,201,240,0.25)',
    },
  },
  {
    id: 'high-contrast',
    name: 'High Contrast',
    tagline: 'Maximum readability.',
    colors: {
      bg: '#000000', cardBg: '#0a0a0a', text: '#ffffff', muted: '#d4d4d4', dim: '#a0a0a0',
      accent: '#ffff00', accentSecondary: '#00ffff', border: '#444444',
      tagBg: 'rgba(255,255,0,0.1)', tagBorder: 'rgba(255,255,0,0.3)',
    },
  },
  {
    id: 'minimal',
    name: 'Minimal',
    tagline: 'Zero noise.',
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

export default function ThemeGallery() {
  const [activeId, setActiveId] = useState('default');
  const theme = themes.find((t) => t.id === activeId)!;

  return (
    <div>
      {/* Tab bar */}
      <div className="flex gap-1 justify-center mb-3 flex-wrap">
        {themes.map((t) => (
          <button
            key={t.id}
            onClick={() => setActiveId(t.id)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-all ${
              t.id === activeId
                ? 'bg-[var(--color-accent)] text-white'
                : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-bg-elevated)]'
            }`}
          >
            {t.name}
          </button>
        ))}
      </div>

      <p className="text-center text-sm text-[var(--color-text-muted)] mb-8">{theme.tagline}</p>

      {/* Preview card */}
      <div className="max-w-[640px] mx-auto">
        <div
          className="rounded-2xl border overflow-hidden transition-all duration-300"
          style={{
            backgroundColor: theme.colors.bg,
            borderColor: theme.colors.border,
            fontFamily: theme.font || "'Inter', sans-serif",
            color: theme.colors.text,
          }}
        >
          {/* Header */}
          <div
            className="px-6 py-4 flex items-center justify-between border-b"
            style={{ borderColor: theme.colors.border }}
          >
            <span className="font-semibold text-sm" style={{ color: theme.colors.accent }}>
              Lattice Digest
            </span>
            <span className="text-xs" style={{ color: theme.colors.dim }}>
              April 6, 2026 &middot; 5 items
            </span>
          </div>

          {/* Content */}
          <div className={`px-6 ${theme.tight ? 'py-3' : 'py-5'} space-y-${theme.tight ? '2' : '4'}`}>
            {/* Topic: Rust */}
            <div>
              <span
                className="inline-block text-[0.65rem] font-semibold uppercase tracking-wider px-2 py-0.5 rounded-full mb-2"
                style={{
                  color: theme.colors.accent,
                  backgroundColor: theme.colors.tagBg,
                  border: `1px solid ${theme.colors.tagBorder}`,
                }}
              >
                Rust
              </span>
              <div className={theme.tight ? 'space-y-1' : 'space-y-3'}>
                <div>
                  <p className={`font-semibold ${theme.tight ? 'text-xs' : 'text-sm'}`}>
                    <a href="#" style={{ color: theme.colors.text }} className="hover:underline">
                      Rust 1.94 ships async closures
                    </a>
                  </p>
                  <p className="text-xs mt-0.5" style={{ color: theme.colors.dim }}>
                    blog.rust-lang.org &middot; Apr 5 &middot; 4 min
                  </p>
                  {!theme.headlinesOnly && (
                    <p className="text-xs mt-1" style={{ color: theme.colors.muted }}>
                      Async closures hit stable, eliminating a major pain point for async code.
                    </p>
                  )}
                </div>
                <div>
                  <p className={`font-semibold ${theme.tight ? 'text-xs' : 'text-sm'}`}>
                    <a href="#" style={{ color: theme.colors.text }} className="hover:underline">
                      cargo-semver-checks reaches 1.0
                    </a>
                  </p>
                  <p className="text-xs mt-0.5" style={{ color: theme.colors.dim }}>
                    github.com &middot; Apr 4 &middot; 2 min
                  </p>
                  {!theme.headlinesOnly && (
                    <p className="text-xs mt-1" style={{ color: theme.colors.muted }}>
                      The Rust ecosystem's semver linter is now stable and ready for CI.
                    </p>
                  )}
                </div>
              </div>
            </div>

            {/* Topic: AI */}
            <div
              className="pt-3 border-t"
              style={{ borderColor: theme.colors.border }}
            >
              <span
                className="inline-block text-[0.65rem] font-semibold uppercase tracking-wider px-2 py-0.5 rounded-full mb-2"
                style={{
                  color: theme.colors.accentSecondary,
                  backgroundColor: theme.id === 'minimal' ? theme.colors.tagBg : `${theme.colors.accentSecondary}15`,
                  border: `1px solid ${theme.id === 'minimal' ? theme.colors.tagBorder : `${theme.colors.accentSecondary}40`}`,
                }}
              >
                AI
              </span>
              <div>
                <p className={`font-semibold ${theme.tight ? 'text-xs' : 'text-sm'}`}>
                  <a href="#" style={{ color: theme.colors.text }} className="hover:underline">
                    MCP Apps spec reaches v1.0
                  </a>
                </p>
                <p className="text-xs mt-0.5" style={{ color: theme.colors.dim }}>
                  modelcontextprotocol.io &middot; Apr 6 &middot; 3 min
                </p>
                {!theme.headlinesOnly && (
                  <p className="text-xs mt-1" style={{ color: theme.colors.muted }}>
                    The Model Context Protocol's interactive UI extension is now finalized.
                  </p>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
