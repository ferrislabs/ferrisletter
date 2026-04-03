import { applyHostStyleVariables } from "@modelcontextprotocol/ext-apps";
import type { McpUiStyles } from "@modelcontextprotocol/ext-apps";

/** Clamp range for numeric CSS values, keyed by variable-name prefix. */
const CLAMP_RULES: Array<{ prefix: string; min: number; max: number }> = [
  { prefix: "--font-text-", min: 10, max: 32 },
  { prefix: "--font-heading-", min: 10, max: 48 },
  { prefix: "--border-radius-", min: 0, max: 24 },
  { prefix: "--border-width-", min: 0, max: 8 },
];

const NUMERIC_RE = /^(-?\d+(?:\.\d+)?)(px|rem|em|pt)?$/;

function clampValue(key: string, value: string): string {
  const rule = CLAMP_RULES.find((r) => key.startsWith(r.prefix));
  if (!rule) return value;

  const match = value.trim().match(NUMERIC_RE);
  if (!match) return value;

  const num = parseFloat(match[1]);
  const unit = match[2] ?? "px";
  const clamped = Math.min(Math.max(num, rule.min), rule.max);
  return `${clamped}${unit}`;
}

/**
 * Apply host style variables with sensible clamps on numeric values.
 * Prevents extreme font sizes, border radii, etc. from breaking layout.
 */
export function applySafeHostStyleVariables(
  styles: McpUiStyles,
  root?: HTMLElement,
): void {
  const sanitized = { ...styles } as Record<string, string | undefined>;
  for (const [key, val] of Object.entries(sanitized)) {
    if (val != null) {
      sanitized[key] = clampValue(key, val);
    }
  }
  applyHostStyleVariables(sanitized as McpUiStyles, root);
}
