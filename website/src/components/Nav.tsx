import { useEffect, useState } from 'react';
import { Menu, X } from 'lucide-react';

const links = [
  { label: 'Features', href: '/features' },
  { label: 'Themes', href: '/themes' },
  { label: 'Docs', href: '/docs' },
];

export default function Nav() {
  const [scrolled, setScrolled] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  useEffect(() => {
    if (mobileOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => { document.body.style.overflow = ''; };
  }, [mobileOpen]);

  return (
    <>
      <nav
        className={`fixed top-0 left-0 right-0 z-100 px-8 py-4 transition-all duration-250 ${
          scrolled ? 'bg-[var(--color-bg)]/80 backdrop-blur-xl border-b border-[var(--color-border)]' : ''
        }`}
      >
        <div className="max-w-[1200px] mx-auto flex items-center justify-between">
          <a href="/" className="flex items-center gap-2 font-bold text-lg tracking-tight">
            <span className="text-[var(--color-accent)] text-xl">&#9671;</span>
            <span>ferrisletter</span>
          </a>

          {/* Desktop links */}
          <div className="hidden md:flex items-center gap-8">
            {links.map((link) => (
              <a
                key={link.href}
                href={link.href}
                className="text-[var(--color-text-muted)] text-sm font-medium hover:text-[var(--color-text)] transition-colors"
              >
                {link.label}
              </a>
            ))}
            <a
              href="https://github.com/ferrislabs/ferrisletter"
              target="_blank"
              rel="noopener"
              className="inline-flex items-center gap-2 px-5 py-2 bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-lg text-sm font-medium hover:border-[var(--color-border-hover)] transition-all"
            >
              GitHub <span>&rarr;</span>
            </a>
            <a
              href="/connect"
              className="inline-flex items-center gap-2 px-5 py-2 bg-[var(--color-accent)] text-white rounded-lg text-sm font-semibold hover:bg-[var(--color-accent-hover)] transition-all"
            >
              Connect
            </a>
          </div>

          {/* Mobile hamburger */}
          <button
            className="md:hidden p-2 text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
            onClick={() => setMobileOpen(true)}
            aria-label="Open menu"
          >
            <Menu size={24} />
          </button>
        </div>
      </nav>

      {/* Mobile overlay */}
      {mobileOpen && (
        <div className="fixed inset-0 z-200 bg-[var(--color-bg)]/95 backdrop-blur-xl">
          <div className="flex items-center justify-between px-8 py-4">
            <a href="/" className="flex items-center gap-2 font-bold text-lg tracking-tight">
              <span className="text-[var(--color-accent)] text-xl">&#9671;</span>
              <span>ferrisletter</span>
            </a>
            <button
              className="p-2 text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
              onClick={() => setMobileOpen(false)}
              aria-label="Close menu"
            >
              <X size={24} />
            </button>
          </div>
          <div className="flex flex-col items-center gap-6 pt-16">
            {links.map((link) => (
              <a
                key={link.href}
                href={link.href}
                className="text-xl font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
                onClick={() => setMobileOpen(false)}
              >
                {link.label}
              </a>
            ))}
            <a
              href="https://github.com/ferrislabs/ferrisletter"
              target="_blank"
              rel="noopener"
              className="text-xl font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
            >
              GitHub
            </a>
            <a
              href="/connect"
              className="mt-4 inline-flex items-center gap-2 px-8 py-3 bg-[var(--color-accent)] text-white rounded-xl text-lg font-semibold hover:bg-[var(--color-accent-hover)] transition-all"
              onClick={() => setMobileOpen(false)}
            >
              Connect
            </a>
          </div>
        </div>
      )}
    </>
  );
}
