import { useEffect, useState } from 'react';

export default function Hero() {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const timeout = setTimeout(() => setVisible(true), 100);
    return () => clearTimeout(timeout);
  }, []);

  return (
    <section className="min-h-screen flex items-center justify-center text-center pt-32 pb-16 relative overflow-hidden z-[1]">
      <div className="absolute w-[600px] h-[600px] -top-[200px] left-1/2 -translate-x-1/2 rounded-full bg-[var(--color-accent)] opacity-[0.08] blur-[120px] pointer-events-none" />
      <div className="absolute w-[400px] h-[400px] bottom-[10%] right-[10%] rounded-full bg-[var(--color-accent-secondary)] opacity-[0.05] blur-[120px] pointer-events-none" />
      <div
        className={`max-w-[1200px] mx-auto px-8 transition-all duration-800 ease-out ${
          visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'
        }`}
      >
        <div className="inline-flex items-center gap-2 px-4 py-1.5 bg-[var(--color-tag-bg)] border border-[var(--color-tag-border)] rounded-full text-sm font-medium text-[var(--color-text-muted)] mb-3">
          <span className="w-1.5 h-1.5 rounded-full bg-[var(--color-accent)] animate-pulse" />
          Open source &middot; MCP-native &middot; POC
        </div>
        <p className="text-xs text-[var(--color-text-dim)] mb-8">
          by{' '}
          <a
            href="https://github.com/ferrislabs"
            target="_blank"
            rel="noopener"
            className="text-[var(--color-text-muted)] hover:text-[var(--color-accent)] transition-colors font-medium"
          >
            Ferrislabs
          </a>
          {' '}&middot;{' '}the team behind{' '}
          <a
            href="https://github.com/ferriskey"
            target="_blank"
            rel="noopener"
            className="text-[var(--color-text-muted)] hover:text-[var(--color-accent)] transition-colors font-medium"
          >
            FerrisKey
          </a>
        </p>

        <h1 className="text-[clamp(3rem,8vw,5.5rem)] font-black leading-[1.05] tracking-[-0.04em] mb-6">
          Newsletters that
          <br />
          <span className="bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-secondary)] bg-clip-text text-transparent">
            talk back.
          </span>
        </h1>

        <p className="text-[clamp(1.1rem,2vw,1.3rem)] text-[var(--color-text-muted)] max-w-[600px] mx-auto mb-12 leading-relaxed">
          Stop reading walls of text. Start having conversations with your news.
          <br />
          Ask what you missed. Expand what interests you. Skip the rest.
        </p>

        <div className="flex gap-4 justify-center mb-16 flex-wrap">
          <a
            href="#how-it-works"
            className="inline-flex items-center gap-2 px-8 py-3.5 bg-[var(--color-accent)] text-white rounded-xl text-base font-semibold hover:bg-[var(--color-accent-hover)] hover:-translate-y-0.5 hover:shadow-[0_8px_30px_rgba(109,90,255,0.3)] transition-all"
          >
            See how it works <span>&darr;</span>
          </a>
          <a
            href="https://github.com/ferrislabs/ferrisletter"
            target="_blank"
            rel="noopener"
            className="inline-flex items-center gap-2 px-8 py-3.5 bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-xl text-base font-semibold hover:border-[var(--color-border-hover)] hover:-translate-y-0.5 transition-all"
          >
            View on GitHub
          </a>
        </div>

        <div className="flex items-center justify-center gap-8 flex-wrap">
          {[
            { value: 'MCP', label: 'Protocol' },
            { value: 'Rust', label: 'Backend' },
            { value: 'React', label: 'UI' },
            { value: 'AuthZEN', label: 'Security' },
          ].map((stat, i) => (
            <div key={stat.value} className="flex items-center gap-8">
              <div className="flex flex-col gap-0.5">
                <span className="font-mono text-sm font-semibold">{stat.value}</span>
                <span className="text-xs text-[var(--color-text-dim)] uppercase tracking-wider">{stat.label}</span>
              </div>
              {i < 3 && <div className="w-px h-8 bg-[var(--color-border)]" />}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
