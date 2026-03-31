import { useEffect, useRef, useState } from 'react';

const steps = [
  { label: 'Your LLM', description: 'Claude, ChatGPT, or any MCP client — where you already work.', icon: '💬', highlight: false },
  { label: 'Ferrisletter', description: 'Our MCP server delivers curated, tagged content on demand.', icon: '◇', highlight: true },
  { label: 'Fresh content', description: "Sourced, summarized, and ready — so you don't have to be.", icon: '🌐', highlight: false },
];

export default function Architecture() {
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observer = new IntersectionObserver(([entry]) => { if (entry.isIntersecting) setVisible(true); }, { threshold: 0.2 });
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section className="py-28 relative z-[1]" ref={ref}>
      <div className="max-w-[800px] mx-auto px-8">
        <div className="text-center mb-16">
          <span className="inline-block font-mono text-xs font-medium text-[var(--color-accent)] uppercase tracking-widest mb-4">How it connects</span>
          <h2 className="text-[clamp(2rem,5vw,3.2rem)] font-extrabold tracking-tight leading-tight mb-4">
            Your LLM does the{' '}
            <span className="bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-secondary)] bg-clip-text text-transparent">talking</span>
          </h2>
          <p className="text-lg text-[var(--color-text-muted)] max-w-[420px] mx-auto">
            Your client connects to our server. We handle the rest.
          </p>
        </div>

        <div className="flex flex-col md:flex-row items-center justify-center gap-0">
          {steps.map((step, i) => (
            <div key={step.label} className="contents">
              <div
                className={`transition-all duration-500 ${visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'}`}
                style={{ transitionDelay: `${i * 150}ms` }}
              >
                <div
                  className={`w-[200px] p-8 bg-[var(--color-bg-elevated)] border rounded-2xl text-center transition-all hover:-translate-y-0.5 hover:shadow-[0_8px_30px_rgba(0,0,0,0.2)] ${
                    step.highlight
                      ? 'border-[var(--color-tag-border)] shadow-[0_0_40px_var(--color-accent-glow)] hover:border-[var(--color-accent)]'
                      : 'border-[var(--color-border)] hover:border-[var(--color-border-hover)]'
                  }`}
                >
                  <span className={`block text-2xl mb-3 ${step.highlight ? 'drop-shadow-[0_0_8px_var(--color-accent-glow)]' : ''}`}>
                    {step.icon}
                  </span>
                  <h3 className="text-base font-bold mb-1.5">{step.label}</h3>
                  <p className="text-xs text-[var(--color-text-muted)] leading-relaxed">{step.description}</p>
                </div>
              </div>
              {i < steps.length - 1 && (
                <div
                  className={`flex flex-col items-center gap-1.5 px-3 py-2 md:py-0 transition-all duration-500 ${visible ? 'opacity-100' : 'opacity-0'}`}
                  style={{ transitionDelay: `${i * 150 + 100}ms` }}
                >
                  <div className="w-0.5 h-8 md:w-12 md:h-0.5 bg-[var(--color-border)] rounded relative overflow-hidden">
                    <div className="absolute bg-[var(--color-accent)] shadow-[0_0_6px_var(--color-accent)] rounded animate-[particleH_2s_ease-in-out_infinite] hidden md:block w-2 h-1 -top-px -left-2" />
                    <div className="absolute bg-[var(--color-accent)] shadow-[0_0_6px_var(--color-accent)] rounded animate-[particleV_2s_ease-in-out_infinite] md:hidden w-1 h-2 -left-px -top-2" />
                  </div>
                  <span className="font-mono text-[0.65rem] font-medium uppercase tracking-wide text-[var(--color-text-dim)]">
                    {i === 0 ? 'requests' : 'delivers'}
                  </span>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
