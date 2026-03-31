import { useEffect, useRef, useState } from 'react';

const layers = [
  { label: 'Protocol', tech: 'MCP', detail: 'Model Context Protocol — the open standard for LLM tool integration', color: '#6d5aff' },
  { label: 'Backend', tech: 'Rust', detail: 'Fast, memory-safe, single-binary deployment. Built for reliability.', color: '#DEA584' },
  { label: 'UI', tech: 'React', detail: 'MCP Apps rendered in sandboxed iframes. Rich, interactive views.', color: '#61DAFB' },
  { label: 'Auth', tech: 'AuthZEN + FerrisKey', detail: 'Standard-based authorization with PDP/PEP. Rust-powered IAM.', color: '#DEA584' },
  { label: 'Database', tech: 'PostgreSQL', detail: 'Battle-tested. Stores content, subscriptions, and preferences.', color: '#336791' },
  { label: 'Content', tech: 'AI Pipeline', detail: 'Automated scraping, LLM-assisted tagging, ready to serve on demand.', color: '#22d3ee' },
];

export default function TechStack() {
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observer = new IntersectionObserver(([entry]) => { if (entry.isIntersecting) setVisible(true); }, { threshold: 0.2 });
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section id="stack" className="py-28 relative z-1" ref={ref}>
      <div className="max-w-[800px] mx-auto px-8">
        <div className="text-center mb-16">
          <span className="inline-block font-mono text-xs font-medium text-[var(--color-accent)] uppercase tracking-widest mb-4">Tech Stack</span>
          <h2 className="text-[clamp(2rem,5vw,3.2rem)] font-extrabold tracking-tight leading-tight mb-4">
            Built with{' '}
            <span className="bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-secondary)] bg-clip-text text-transparent">
              opinionated choices
            </span>
          </h2>
          <p className="text-lg text-[var(--color-text-muted)]">Every layer chosen for a reason. Open source all the way down.</p>
        </div>

        <div className="flex flex-col">
          {layers.map((layer, i) => (
            <div
              key={layer.tech}
              className={`flex gap-6 py-6 transition-all duration-500 ${visible ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-5'}`}
              style={{ transitionDelay: `${i * 80}ms` }}
            >
              <div className="flex flex-col items-center shrink-0 w-5">
                <span className="w-3 h-3 rounded-full shrink-0" style={{ background: layer.color, boxShadow: `0 0 12px ${layer.color}` }} />
                {i < layers.length - 1 && <span className="flex-1 w-px bg-[var(--color-border)] min-h-5" />}
              </div>
              <div className={`flex-1 ${i < layers.length - 1 ? 'pb-4 border-b border-[var(--color-border)]' : ''}`}>
                <div className="flex items-center gap-4 mb-1">
                  <span className="font-mono text-xs uppercase tracking-wider text-[var(--color-text-dim)] min-w-[80px]">{layer.label}</span>
                  <span className="text-lg font-bold">{layer.tech}</span>
                </div>
                <p className="text-sm text-[var(--color-text-muted)] leading-relaxed pl-[calc(80px+1rem)] max-sm:pl-0">{layer.detail}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
