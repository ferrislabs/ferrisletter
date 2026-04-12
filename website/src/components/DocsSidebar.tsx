import { useState } from 'react';
import { Menu, X } from 'lucide-react';

interface Props {
  currentPath: string;
}

const sections = [
  {
    title: 'Getting Started',
    items: [
      { label: 'Overview', href: '/docs' },
      { label: 'Getting Started', href: '/docs/getting-started' },
      { label: 'Connect to Claude', href: '/docs/connect-claude' },
    ],
  },
  {
    title: 'Configuration',
    items: [
      { label: 'Display Preferences', href: '/docs/display-preferences' },
      { label: 'Scheduled Delivery', href: '/docs/scheduled-delivery' },
    ],
  },
  {
    title: 'Advanced',
    items: [
      { label: 'Building Connectors', href: '/docs/building-connectors' },
    ],
  },
];

function isActive(currentPath: string, href: string) {
  const clean = currentPath.replace(/\/$/, '') || '/';
  const target = href.replace(/\/$/, '') || '/';
  return clean === target;
}

export default function DocsSidebar({ currentPath }: Props) {
  const [open, setOpen] = useState(false);

  const nav = (
    <nav className="space-y-6">
      {sections.map((section) => (
        <div key={section.title}>
          <span className="text-xs font-semibold uppercase tracking-wider text-[var(--color-text-dim)] mb-2 block">
            {section.title}
          </span>
          <ul className="space-y-1">
            {section.items.map((item) => (
              <li key={item.href}>
                <a
                  href={item.href}
                  className={`block px-3 py-1.5 rounded-md text-sm transition-colors ${
                    isActive(currentPath, item.href)
                      ? 'text-[var(--color-accent)] bg-[var(--color-accent-glow)] font-medium border-l-2 border-[var(--color-accent)]'
                      : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-bg-elevated)]'
                  }`}
                >
                  {item.label}
                </a>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </nav>
  );

  return (
    <>
      {/* Desktop: always visible */}
      <div className="hidden md:block sticky top-28">
        {nav}
      </div>

      {/* Mobile: collapsible */}
      <div className="md:hidden">
        <button
          onClick={() => setOpen(!open)}
          className="inline-flex items-center gap-2 px-3 py-2 rounded-lg bg-[var(--color-bg-elevated)] border border-[var(--color-border)] text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors w-full"
        >
          {open ? <X size={16} /> : <Menu size={16} />}
          Docs Menu
        </button>
        {open && (
          <div className="mt-3 p-4 bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-xl">
            {nav}
          </div>
        )}
      </div>
    </>
  );
}
