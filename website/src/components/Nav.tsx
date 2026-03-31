import { useEffect, useState } from 'react';

export default function Nav() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
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
        <div className="flex items-center gap-8">
          <a href="#how-it-works" className="hidden sm:inline text-[var(--color-text-muted)] text-sm font-medium hover:text-[var(--color-text)] transition-colors">
            How it works
          </a>
          <a href="#features" className="hidden sm:inline text-[var(--color-text-muted)] text-sm font-medium hover:text-[var(--color-text)] transition-colors">
            Features
          </a>
          <a href="#stack" className="hidden sm:inline text-[var(--color-text-muted)] text-sm font-medium hover:text-[var(--color-text)] transition-colors">
            Stack
          </a>
          <a
            href="https://github.com/ferrislabs/ferrisletter"
            target="_blank"
            rel="noopener"
            className="inline-flex items-center gap-2 px-5 py-2 bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-lg text-sm font-medium hover:border-[var(--color-border-hover)] transition-all"
          >
            GitHub <span>&rarr;</span>
          </a>
        </div>
      </div>
    </nav>
  );
}
