import { useEffect, useRef, useState } from 'react';

export default function HowItsDifferent() {
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!ref.current) return;
    const observer = new IntersectionObserver(
      ([entry]) => { if (entry.isIntersecting) setVisible(true); },
      { threshold: 0.1 },
    );
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section id="how-it-works" className="py-28 relative z-[1]" ref={ref}>
      <div className="max-w-[1200px] mx-auto px-8">
        {/* Header */}
        <div className="text-center mb-20">
          <span className="inline-block font-mono text-xs font-medium text-[var(--color-accent)] uppercase tracking-widest mb-4">
            How it works
          </span>
          <h2 className="text-[clamp(2rem,5vw,3.2rem)] font-extrabold tracking-tight leading-tight mb-4">
            Four steps to your{' '}
            <span className="bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-secondary)] bg-clip-text text-transparent">
              personal newsroom
            </span>
          </h2>
          <p className="text-lg text-[var(--color-text-muted)] max-w-[520px] mx-auto">
            An interactive UI that renders right inside Claude — no separate app needed.
          </p>
        </div>

        <div className="flex flex-col gap-24">
          {/* Step 1: Connect */}
          <Step
            index={0}
            visible={visible}
            tag="Step 1"
            title="Connect in seconds"
            description="Add a single config snippet to Claude Desktop. The MCP server connects automatically — no install, no build, no signup."
            highlight="Paste the config, restart Claude, and you're live."
            link={{ href: '/connect', label: 'Setup guide' }}
          >
            <MockClaudeWindow title="claude_desktop_config.json">
              <pre className="font-mono text-xs text-[var(--color-text-muted)] leading-relaxed p-4">
                <span className="text-[var(--color-text-dim)]">{'{'}</span>{'\n'}
                {'  '}<span className="text-[var(--color-accent)]">"mcpServers"</span>: {'{'}{'\n'}
                {'    '}<span className="text-[var(--color-accent)]">"lattice"</span>: {'{'}{'\n'}
                {'      '}<span className="text-[var(--color-text-dim)]">"command"</span>: <span className="text-[var(--color-accent-secondary)]">"npx"</span>,{'\n'}
                {'      '}<span className="text-[var(--color-text-dim)]">"args"</span>: [<span className="text-[var(--color-accent-secondary)]">"mcp-remote@latest"</span>, <span className="text-[var(--color-accent-secondary)]">"https://..."</span>]{'\n'}
                {'    }'}{'\n'}
                {'  }'}{'\n'}
                <span className="text-[var(--color-text-dim)]">{'}'}</span>
              </pre>
              <div className="px-4 pb-3 flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                <span className="text-[0.65rem] text-green-400 font-mono">lattice connected</span>
              </div>
            </MockClaudeWindow>
          </Step>

          {/* Step 2: Browse */}
          <Step
            index={1}
            visible={visible}
            tag="Step 2"
            title="Browse your feed"
            description="The MCP App UI renders an interactive panel directly inside Claude. Filter by topic, scan headlines, and expand any item to read the full content."
            highlight="This UI renders right inside Claude — no separate app needed."
          >
            <MockClaudeWindow title="ferrisletter panel">
              <div className="p-4 space-y-3">
                {/* Topic filters */}
                <div className="flex gap-2 flex-wrap">
                  {['All', 'Rust', 'AI', 'MCP'].map((t, i) => (
                    <span
                      key={t}
                      className={`px-2.5 py-1 text-[0.65rem] font-medium rounded-full ${
                        i === 0
                          ? 'bg-[var(--color-tag-bg)] border border-[var(--color-tag-border)] text-[var(--color-accent)]'
                          : 'bg-transparent border border-[var(--color-border)] text-[var(--color-text-dim)]'
                      }`}
                    >
                      {t}
                    </span>
                  ))}
                </div>
                {/* Items */}
                <div className="space-y-2">
                  <ItemRow
                    topic="Rust"
                    headline="Async closures hit stable in Rust 1.94"
                    meta="blog.rust-lang.org · 4 min"
                    expanded
                    summary="The long-awaited async closures feature lands in stable Rust, eliminating a major friction point for async code."
                  />
                  <ItemRow
                    topic="AI"
                    headline="MCP Apps spec reaches v1.0"
                    meta="modelcontextprotocol.io · 3 min"
                  />
                  <ItemRow
                    topic="MCP"
                    headline="New transport layer draft published"
                    meta="github.com · 2 min"
                  />
                </div>
              </div>
            </MockClaudeWindow>
          </Step>

          {/* Step 3: Interact */}
          <Step
            index={2}
            visible={visible}
            tag="Step 3"
            title="Search and explore"
            description="Ask Claude for a recap of what you missed, search by keyword, or filter by tags. The UI updates in real-time alongside the conversation."
            highlight="Search, filter, and explore — all within the conversation."
          >
            <MockClaudeWindow title="ferrisletter search">
              <div className="p-4 space-y-3">
                {/* Search bar mockup */}
                <div className="flex items-center gap-2 px-3 py-2 bg-[var(--color-bg)] border border-[var(--color-border)] rounded-lg">
                  <span className="text-[var(--color-text-dim)] text-xs">&#128269;</span>
                  <span className="font-mono text-xs text-[var(--color-text-muted)]">async closures</span>
                </div>
                {/* Results */}
                <div className="text-[0.65rem] text-[var(--color-text-dim)] font-mono mb-1">3 results</div>
                <div className="space-y-2">
                  <ItemRow
                    topic="Rust"
                    headline="Async closures hit stable in Rust 1.94"
                    meta="blog.rust-lang.org · Apr 5"
                  />
                  <ItemRow
                    topic="Rust"
                    headline="RFC: async closure trait bounds"
                    meta="github.com/rust-lang · Mar 28"
                  />
                  <ItemRow
                    topic="Rust"
                    headline="Async patterns in embedded Rust"
                    meta="embedded.rs · Mar 15"
                  />
                </div>
              </div>
            </MockClaudeWindow>
          </Step>

          {/* Step 4: Personalize */}
          <Step
            index={3}
            visible={visible}
            tag="Step 4"
            title="Make it yours"
            description="Subscribe to topics, set your preferred theme, configure delivery — all through natural conversation. No settings page needed."
            highlight="Tell Claude what you care about. The UI updates instantly."
            link={{ href: '/features', label: 'See all features' }}
          >
            <MockClaudeWindow title="preferences">
              <div className="p-4 space-y-4">
                {/* Topic picker */}
                <div>
                  <div className="text-[0.65rem] font-semibold uppercase tracking-wider text-[var(--color-text-dim)] mb-2">Subscribed topics</div>
                  <div className="flex gap-2 flex-wrap">
                    {[
                      { name: 'Rust', active: true },
                      { name: 'AI', active: true },
                      { name: 'MCP', active: true },
                      { name: 'Web', active: false },
                      { name: 'Security', active: false },
                    ].map((t) => (
                      <span
                        key={t.name}
                        className={`px-2.5 py-1 text-[0.65rem] font-medium rounded-full transition-all ${
                          t.active
                            ? 'bg-[var(--color-tag-bg)] border border-[var(--color-tag-border)] text-[var(--color-accent)]'
                            : 'bg-transparent border border-[var(--color-border)] text-[var(--color-text-dim)]'
                        }`}
                      >
                        {t.active ? '✓ ' : ''}{t.name}
                      </span>
                    ))}
                  </div>
                </div>
                {/* Settings */}
                <div className="space-y-2">
                  <SettingRow label="Theme" value="Daltonian" />
                  <SettingRow label="Summary" value="Standard" />
                  <SettingRow label="Delivery" value="Weekdays · 9:00 AM" />
                </div>
                {/* Conversation snippet */}
                <div className="border-t border-[var(--color-border)] pt-3">
                  <div className="flex items-start gap-2 text-xs">
                    <span className="text-[var(--color-accent)] shrink-0 mt-0.5">&#9656;</span>
                    <span className="text-[var(--color-text-muted)] italic">"Use colorblind-safe theme with emoji headers"</span>
                  </div>
                  <div className="flex items-start gap-2 text-xs mt-2">
                    <span className="text-[var(--color-accent-secondary)] shrink-0 mt-0.5">&#9671;</span>
                    <span className="text-[var(--color-text-dim)]">Updated! Daltonian theme + emoji headers applied.</span>
                  </div>
                </div>
              </div>
            </MockClaudeWindow>
          </Step>
        </div>
      </div>
    </section>
  );
}

/* ─── Sub-components ─── */

function Step({
  index,
  visible,
  tag,
  title,
  description,
  highlight,
  link,
  children,
}: {
  index: number;
  visible: boolean;
  tag: string;
  title: string;
  description: string;
  highlight: string;
  link?: { href: string; label: string };
  children: React.ReactNode;
}) {
  const reverse = index % 2 === 1;
  return (
    <div
      className={`grid grid-cols-1 md:grid-cols-2 gap-12 items-center transition-all duration-700 ease-out ${
        visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'
      }`}
      style={{ transitionDelay: `${index * 100}ms` }}
    >
      <div className={reverse ? 'md:order-2' : ''}>
        <span className="inline-block font-mono text-xs font-semibold uppercase tracking-wider text-[var(--color-accent)] mb-3">
          {tag}
        </span>
        <h3 className="text-2xl font-extrabold tracking-tight mb-3">{title}</h3>
        <p className="text-[var(--color-text-muted)] leading-relaxed mb-3">{description}</p>
        <p className="text-sm text-[var(--color-accent)] font-medium">{highlight}</p>
        {link && (
          <a
            href={link.href}
            className="inline-flex items-center gap-1 text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] mt-3 transition-colors"
          >
            {link.label} &rarr;
          </a>
        )}
      </div>
      <div className={reverse ? 'md:order-1' : ''}>{children}</div>
    </div>
  );
}

function MockClaudeWindow({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-2xl overflow-hidden shadow-[0_0_40px_var(--color-accent-glow)]">
      {/* Title bar */}
      <div className="flex items-center gap-2 px-4 py-2.5 border-b border-[var(--color-border)]">
        <div className="flex gap-1.5">
          <span className="w-2.5 h-2.5 rounded-full bg-[#ff5f57]" />
          <span className="w-2.5 h-2.5 rounded-full bg-[#febc2e]" />
          <span className="w-2.5 h-2.5 rounded-full bg-[#28c840]" />
        </div>
        <span className="font-mono text-[0.65rem] text-[var(--color-text-dim)] ml-2">{title}</span>
      </div>
      {children}
    </div>
  );
}

function ItemRow({
  topic,
  headline,
  meta,
  expanded,
  summary,
}: {
  topic: string;
  headline: string;
  meta: string;
  expanded?: boolean;
  summary?: string;
}) {
  return (
    <div className={`p-3 rounded-lg border transition-all ${
      expanded
        ? 'bg-[rgba(109,90,255,0.04)] border-[var(--color-tag-border)]'
        : 'bg-[var(--color-bg-card)] border-[var(--color-border)]'
    }`}>
      <div className="flex items-start gap-2">
        <span className="inline-block text-[0.6rem] font-semibold uppercase tracking-wider text-[var(--color-accent)] bg-[var(--color-tag-bg)] border border-[var(--color-tag-border)] px-1.5 py-0.5 rounded shrink-0 mt-0.5">
          {topic}
        </span>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium truncate">{headline}</p>
          <p className="text-[0.65rem] text-[var(--color-text-dim)] mt-0.5">{meta}</p>
        </div>
      </div>
      {expanded && summary && (
        <p className="text-xs text-[var(--color-text-muted)] mt-2 pt-2 border-t border-[var(--color-border)] leading-relaxed">
          {summary}
        </p>
      )}
    </div>
  );
}

function SettingRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between py-1.5 px-3 rounded-lg bg-[var(--color-bg-card)] border border-[var(--color-border)]">
      <span className="text-xs text-[var(--color-text-dim)]">{label}</span>
      <span className="text-xs font-medium text-[var(--color-text)]">{value}</span>
    </div>
  );
}
