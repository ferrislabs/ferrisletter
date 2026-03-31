import { useEffect, useRef, useState } from 'react';

export default function CTA() {
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observer = new IntersectionObserver(([entry]) => { if (entry.isIntersecting) setVisible(true); }, { threshold: 0.3 });
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <>
      <section className="py-28 pb-24 relative z-[1]" ref={ref}>
        <div className="absolute w-[500px] h-[500px] bottom-0 left-1/2 -translate-x-1/2 rounded-full bg-[var(--color-accent)] opacity-[0.06] blur-[120px] pointer-events-none" />
        <div
          className={`max-w-[800px] mx-auto px-8 transition-all duration-700 ease-out ${visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-5'}`}
        >
          <div className="text-center p-16 bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-3xl relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-[rgba(109,90,255,0.05)] via-transparent to-[rgba(34,211,238,0.03)] pointer-events-none" />
            <span className="inline-block font-mono text-xs font-semibold uppercase tracking-wider text-[var(--color-accent)] mb-6 relative">
              Early access
            </span>
            <h2 className="text-[clamp(1.8rem,4vw,2.8rem)] font-extrabold tracking-tight leading-tight mb-4 relative">
              The future of newsletters
              <br />
              is a conversation.
            </h2>
            <p className="text-base text-[var(--color-text-muted)] max-w-[480px] mx-auto mb-8 leading-relaxed relative">
              Ferrisletter is open source and under active development. Star the repo, follow the progress, or contribute.
            </p>
            <a
              href="https://github.com/ferrislabs/ferrisletter"
              target="_blank"
              rel="noopener"
              className="inline-flex items-center gap-2 px-10 py-3.5 bg-[var(--color-accent)] text-white rounded-xl text-base font-semibold hover:bg-[var(--color-accent-hover)] hover:-translate-y-0.5 hover:shadow-[0_8px_30px_rgba(109,90,255,0.3)] transition-all relative"
            >
              Star on GitHub
            </a>
          </div>
        </div>
      </section>

      <footer className="py-8 border-t border-[var(--color-border)] relative z-[1]">
        <div className="max-w-[1200px] mx-auto px-8 flex items-center justify-between max-sm:flex-col max-sm:gap-3 max-sm:text-center">
          <div className="flex items-center gap-2 font-bold text-base">
            <span className="text-[var(--color-accent)]">&#9671;</span>
            <span>ferrisletter</span>
          </div>
          <p className="text-xs text-[var(--color-text-dim)]">
            Open source &middot; MCP-native &middot; Built with Rust
          </p>
        </div>
      </footer>
    </>
  );
}
