import { useState } from 'react';
import { Copy, Check } from 'lucide-react';

interface Props {
  code: string;
  filename?: string;
}

export default function CodeBlock({ code, filename }: Props) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-xl overflow-hidden">
      {filename && (
        <div className="px-4 py-2.5 border-b border-[var(--color-border)] flex items-center justify-between">
          <span className="font-mono text-xs text-[var(--color-text-dim)]">{filename}</span>
          <button
            onClick={handleCopy}
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-bg)] transition-all"
            aria-label="Copy to clipboard"
          >
            {copied ? <Check size={14} /> : <Copy size={14} />}
            {copied ? 'Copied' : 'Copy'}
          </button>
        </div>
      )}
      <div className="relative">
        {!filename && (
          <button
            onClick={handleCopy}
            className="absolute top-3 right-3 inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-bg)] transition-all"
            aria-label="Copy to clipboard"
          >
            {copied ? <Check size={14} /> : <Copy size={14} />}
            {copied ? 'Copied' : 'Copy'}
          </button>
        )}
        <pre className="p-4 overflow-x-auto font-mono text-sm text-[var(--color-text-muted)] leading-relaxed">
          <code>{code}</code>
        </pre>
      </div>
    </div>
  );
}
