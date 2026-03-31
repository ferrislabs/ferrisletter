import { useEffect, useRef, useState } from 'react';

export default function HowItsDifferent() {
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observer = new IntersectionObserver(([entry]) => { if (entry.isIntersecting) setVisible(true); }, { threshold: 0.3 });
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section className="py-28 relative z-[1]" ref={ref}>
      <div className="max-w-[1200px] mx-auto px-8">
        <div
          className={`grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] gap-8 items-center transition-all duration-700 ease-out ${
            visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'
          }`}
        >
          {/* Traditional */}
          <div>
            <div className="font-mono text-xs font-semibold uppercase tracking-wider text-[var(--color-text-dim)] mb-6 text-center">
              Traditional Newsletter
            </div>
            <div className="bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-xl overflow-hidden min-h-[360px]">
              <div className="p-5">
                <div className="mb-5 pb-4 border-b border-[var(--color-border)]">
                  <div className="flex items-center gap-2 text-xs text-[var(--color-text-dim)] mb-1">
                    <span className="w-2 h-2 rounded-full bg-[var(--color-border-hover)]" />
                    newsletter@weekly-digest.com
                  </div>
                  <div className="text-sm font-semibold text-[var(--color-text-muted)]">Your Weekly AI Digest #147</div>
                </div>
                <div className="flex flex-col gap-1.5">
                  <div className="h-2.5 bg-[var(--color-border)] rounded opacity-70 w-3/5" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50 w-2/5" />
                  <div className="h-3" />
                  <div className="h-2.5 bg-[var(--color-border)] rounded opacity-70 w-3/5" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50 w-2/5" />
                  <div className="h-3" />
                  <div className="h-2.5 bg-[var(--color-border)] rounded opacity-70 w-3/5" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-50 w-2/5" />
                  <div className="h-3" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-20" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-20" />
                  <div className="h-2 bg-[var(--color-border)] rounded opacity-20 w-2/5" />
                </div>
                <div className="text-center text-xs text-[var(--color-text-dim)] pt-4 opacity-50">&#8942; 2000+ more words</div>
              </div>
            </div>
            <p className="text-center text-sm text-[var(--color-text-dim)] mt-4 font-medium">Long. Static. All or nothing.</p>
          </div>

          {/* Divider */}
          <div className="flex md:flex-col items-center justify-center">
            <span className="font-mono text-xs text-[var(--color-text-dim)] bg-[var(--color-bg)] px-2 py-1 rounded-md border border-[var(--color-border)]">
              vs
            </span>
          </div>

          {/* AI Letter */}
          <div>
            <div className="font-mono text-xs font-semibold uppercase tracking-wider text-[var(--color-accent)] mb-6 text-center">
              Ferrisletter
            </div>
            <div className="bg-[var(--color-bg-elevated)] border border-[var(--color-tag-border)] rounded-xl overflow-hidden min-h-[360px] shadow-[0_0_40px_var(--color-accent-glow)]">
              <div className="p-5 flex flex-col gap-3">
                <div className="flex items-center justify-center gap-1.5 px-3 py-1 font-mono text-[0.6rem] font-medium text-[var(--color-accent)] bg-[var(--color-tag-bg)] border border-[var(--color-tag-border)] rounded-full w-fit mx-auto">
                  <span className="text-[0.7rem]">&#9889;</span>
                  Scheduled: weekly digest
                </div>
                <div className="bg-[var(--color-bg-card)] border border-[var(--color-border)] rounded-lg p-3">
                  <div className="flex flex-col">
                    {[
                      { tag: 'Research', text: 'New arch beats SOTA with fewer params', active: false },
                      { tag: 'OSS', text: 'LLM framework goes v3.0', active: false },
                      { tag: 'Industry', text: 'AI chip merger shakes up market', active: true, expanded: 'Two startups combine to create a serious competitor to NVIDIA\'s dominance in training hardware...' },
                    ].map((item, i) => (
                      <div
                        key={item.tag}
                        className={`py-2 ${i < 2 ? 'border-b border-[var(--color-border)]' : ''} text-xs flex items-start gap-2 flex-wrap ${
                          item.active ? 'bg-[rgba(109,90,255,0.04)] -mx-3 px-3 py-2' : ''
                        }`}
                      >
                        <span className="font-mono text-[0.6rem] font-semibold uppercase tracking-wide text-[var(--color-accent)] bg-[var(--color-tag-bg)] border border-[var(--color-tag-border)] px-1.5 py-0.5 rounded shrink-0">
                          {item.tag}
                        </span>
                        <span className="text-[var(--color-text)]">{item.text}</span>
                        {item.active && item.expanded && (
                          <div className="w-full mt-1.5 pt-1.5 border-t border-[var(--color-border)] text-[0.75rem] text-[var(--color-text-muted)] leading-relaxed">
                            {item.expanded}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
                <div className="self-end bg-[var(--color-accent)] text-white text-xs px-3 py-1.5 rounded-lg rounded-br-sm max-w-[80%]">
                  Skip industry, more on the research
                </div>
              </div>
            </div>
            <p className="text-center text-sm text-[var(--color-text-muted)] mt-4 font-medium">Delivered to you. Interactive. On your terms.</p>
          </div>
        </div>
      </div>
    </section>
  );
}
