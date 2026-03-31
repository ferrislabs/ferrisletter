import { useEffect, useRef, useState } from 'react';

interface NewsCard {
  title: string;
  items: { topic: string; headline: string; expanded?: string }[];
}

interface Message {
  role: 'user' | 'assistant' | 'system';
  text: string;
  card?: NewsCard;
  typing?: boolean;
}

const demoConversation: Message[] = [
  { role: 'system', text: 'Scheduled task: Ferrisletter weekly digest' },
  {
    role: 'assistant',
    text: 'Your weekly AI Digest just dropped — 4 stories this week:',
    card: {
      title: 'AI Digest — Week 14',
      items: [
        {
          topic: 'Research',
          headline: 'New architecture achieves SOTA on reasoning with 10x fewer params',
          expanded:
            'Researchers published a paper introducing a sparse mixture-of-experts architecture that matches frontier model performance on GSM8K, MATH, and ARC-AGI while using significantly fewer active parameters at inference time. The key insight is a novel routing mechanism that dynamically allocates compute based on problem complexity.',
        },
        { topic: 'Open Source', headline: 'Major LLM framework hits v3.0 with native MCP support' },
        { topic: 'Industry', headline: 'Two AI chip startups announce merger, challenging NVIDIA' },
        { topic: 'Opinion', headline: 'Why retrieval-augmented generation is already obsolete' },
      ],
    },
  },
  { role: 'user', text: 'Tell me more about the research paper' },
  { role: 'assistant', text: '', typing: true },
];

export default function ChatDemo() {
  const [visibleMessages, setVisibleMessages] = useState(0);
  const [expandedItem, setExpandedItem] = useState<number | null>(null);
  const [sectionVisible, setSectionVisible] = useState(false);
  const sectionRef = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!sectionRef.current) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !sectionVisible) {
          setSectionVisible(true);
        }
      },
      { threshold: 0.3 }
    );
    observer.observe(sectionRef.current);
    return () => observer.disconnect();
  }, [sectionVisible]);

  useEffect(() => {
    if (!sectionVisible) return;
    let i = 0;
    const interval = setInterval(() => {
      i++;
      setVisibleMessages(i);
      if (i >= demoConversation.length - 1) clearInterval(interval);
    }, 1000);
    return () => clearInterval(interval);
  }, [sectionVisible]);

  const handleExpand = (idx: number) => {
    setExpandedItem(expandedItem === idx ? null : idx);
    if (expandedItem !== idx) {
      setTimeout(() => setVisibleMessages(demoConversation.length), 300);
    }
  };

  return (
    <section id="how-it-works" className="py-28 relative z-1" ref={sectionRef}>
      <div className="max-w-[1200px] mx-auto px-8">
        <div className="text-center mb-16">
          <span className="inline-block font-mono text-xs font-medium text-[var(--color-accent)] uppercase tracking-widest mb-4">
            How it works
          </span>
          <h2 className="text-[clamp(2rem,5vw,3.2rem)] font-extrabold tracking-tight leading-tight mb-4">
            Your newsletter is a{' '}
            <span className="bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-secondary)] bg-clip-text text-transparent">
              conversation
            </span>
          </h2>
          <p className="text-lg text-[var(--color-text-muted)] max-w-[500px] mx-auto">
            Your LLM tells you when new content arrives. You just respond.
          </p>
        </div>

        <div className="max-w-[680px] mx-auto">
          <div className="bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-2xl overflow-hidden shadow-[0_20px_60px_rgba(0,0,0,0.4)]">
            <div className="flex items-center justify-between px-5 py-3.5 border-b border-[var(--color-border)] bg-white/[0.02]">
              <div className="flex gap-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-[#ff5f57]" />
                <span className="w-2.5 h-2.5 rounded-full bg-[#febc2e]" />
                <span className="w-2.5 h-2.5 rounded-full bg-[#28c840]" />
              </div>
              <span className="text-xs text-[var(--color-text-dim)] font-medium">Claude &middot; Ferrisletter MCP</span>
              <div />
            </div>

            <div className="p-6 flex flex-col gap-4 min-h-[400px]">
              {demoConversation.map((msg, i) =>
                i < visibleMessages ? (
                  <div
                    key={`msg-${i}`}
                    className={`animate-[msgIn_0.4s_cubic-bezier(0.16,1,0.3,1)_both] ${msg.role === 'user' ? 'flex justify-end' : ''}`}
                  >
                    {msg.role === 'system' ? (
                      <div className="flex items-center justify-center gap-2 px-4 py-2 font-mono text-[0.72rem] font-medium text-[var(--color-accent)] bg-[var(--color-tag-bg)] border border-[var(--color-tag-border)] rounded-full w-fit mx-auto tracking-wide">
                        <span className="text-sm">&#9889;</span>
                        {msg.text}
                      </div>
                    ) : msg.role === 'user' ? (
                      <div className="max-w-[90%] px-4 py-3 rounded-xl rounded-br-sm bg-[var(--color-accent)] text-white text-[0.95rem] leading-relaxed">
                        {msg.text}
                      </div>
                    ) : msg.typing ? (
                      <div className="max-w-[90%] px-4 py-3 rounded-xl rounded-bl-sm bg-[var(--color-bg-card)] border border-[var(--color-border)]">
                        <div className="flex gap-1 py-1">
                          {[0, 1, 2].map((j) => (
                            <span
                              key={j}
                              className="w-1.5 h-1.5 rounded-full bg-[var(--color-text-dim)] animate-bounce"
                              style={{ animationDelay: `${j * 0.2}s`, animationDuration: '1.4s' }}
                            />
                          ))}
                        </div>
                      </div>
                    ) : (
                      <div className="max-w-[90%] px-4 py-3 rounded-xl rounded-bl-sm bg-[var(--color-bg-card)] border border-[var(--color-border)]">
                        <p className="mb-3 text-[var(--color-text-muted)] text-[0.95rem]">{msg.text}</p>
                        {msg.card && (
                          <div className="bg-[var(--color-bg)] border border-[var(--color-border)] rounded-xl overflow-hidden">
                            <div className="flex items-center gap-2 px-4 py-3 border-b border-[var(--color-border)] text-sm font-semibold">
                              <span className="text-[var(--color-accent)]">&#9671;</span>
                              <span className="text-[var(--color-text-muted)]">{msg.card.title}</span>
                            </div>
                            {msg.card.items.map((item, j) => (
                              <button
                                key={`item-${j}`}
                                type="button"
                                onClick={() => handleExpand(j)}
                                className={`block w-full text-left border-b border-[var(--color-border)] last:border-b-0 px-4 py-3 cursor-pointer hover:bg-white/[0.02] transition-colors ${
                                  expandedItem === j ? 'bg-[rgba(109,90,255,0.04)]' : ''
                                }`}
                              >
                                <div className="flex items-start gap-3">
                                  <span className="shrink-0 font-mono text-[0.7rem] font-semibold uppercase tracking-wide text-[var(--color-accent)] bg-[var(--color-tag-bg)] border border-[var(--color-tag-border)] px-2 py-0.5 rounded mt-0.5">
                                    {item.topic}
                                  </span>
                                  <span className="flex-1 text-sm leading-snug">{item.headline}</span>
                                  <span className="shrink-0 text-[var(--color-text-dim)] text-base font-light">
                                    {expandedItem === j ? '−' : '+'}
                                  </span>
                                </div>
                                {expandedItem === j && item.expanded && (
                                  <div className="mt-3 pt-3 border-t border-[var(--color-border)] text-[0.85rem] leading-relaxed text-[var(--color-text-muted)] animate-[expandIn_0.3s_ease_both]">
                                    {item.expanded}
                                  </div>
                                )}
                              </button>
                            ))}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                ) : null
              )}
            </div>
          </div>
          <p className="text-center text-sm text-[var(--color-text-dim)] mt-6">
            The LLM notifies you. Click a topic to expand — just like in your chat client.
          </p>
        </div>
      </div>
    </section>
  );
}
