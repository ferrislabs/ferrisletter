import { useEffect, useRef, useState } from 'react';

const features = [
  { icon: '⇄', title: 'Expand on demand', description: "Every topic is a one-liner. Tap to go deeper. No scrolling through content you don't care about." },
  { icon: '↺', title: 'Catch-up in seconds', description: '"What did I miss this week?" Get a smart recap of everything since your last visit. Days, not paragraphs.' },
  { icon: '⏱', title: 'Delivered to your chat', description: 'Scheduled delivery via your LLM client. The newsletter shows up like a message — zero effort.' },
  { icon: '⚙', title: 'Your preferences, your rules', description: 'Choose your topics, set summary length, pick your tone. The same content, shaped to how you consume it.' },
  { icon: '🔍', title: 'Search across issues', description: '"What were the big open source releases last month?" Search across all past issues by topic, date, or keyword.' },
  { icon: '◇', title: 'MCP-native, open source', description: 'Built on the Model Context Protocol. Works with Claude, ChatGPT, and any MCP-compatible client.' },
];

export default function Features() {
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observer = new IntersectionObserver(([entry]) => { if (entry.isIntersecting) setVisible(true); }, { threshold: 0.15 });
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section id="features" className="py-28 relative z-[1]" ref={ref}>
      <div className="max-w-[1200px] mx-auto px-8">
        <div className="text-center mb-16">
          <span className="inline-block font-mono text-xs font-medium text-[var(--color-accent)] uppercase tracking-widest mb-4">Features</span>
          <h2 className="text-[clamp(2rem,5vw,3.2rem)] font-extrabold tracking-tight leading-tight">
            Not another newsletter tool.
            <br />
            <span className="bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-secondary)] bg-clip-text text-transparent">
              A new way to stay informed.
            </span>
          </h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, i) => (
            <div
              key={feature.title}
              className={`p-8 bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-2xl transition-all duration-250 hover:border-[var(--color-border-hover)] hover:-translate-y-0.5 hover:shadow-[0_8px_30px_rgba(0,0,0,0.2)] ${
                visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-5'
              }`}
              style={{ transitionDelay: `${i * 80}ms` }}
            >
              <div className="text-2xl mb-4 text-[var(--color-accent)]">{feature.icon}</div>
              <h3 className="text-lg font-bold mb-2 tracking-tight">{feature.title}</h3>
              <p className="text-sm text-[var(--color-text-muted)] leading-relaxed">{feature.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
