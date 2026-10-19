/** Scores how well `query` matches `text` as an in-order subsequence,
 * case-insensitively - `null` when it doesn't match at all. Higher is
 * better: consecutive characters and matches at the start of a word count
 * extra, so "web" ranks "web01" above "dev-webserver" above "w-e-b". */
export function fuzzyScore(query: string, text: string): number | null {
  const q = query.trim().toLowerCase();
  if (!q) return 0;
  const t = text.toLowerCase();
  let score = 0;
  let ti = 0;
  let prevMatch = -2;
  for (const ch of q) {
    if (ch === " ") continue;
    const found = t.indexOf(ch, ti);
    if (found === -1) return null;
    score += 1;
    if (found === prevMatch + 1) score += 3;
    if (found === 0 || /[\s/@._-]/.test(t[found - 1])) score += 2;
    prevMatch = found;
    ti = found + 1;
  }
  // Shorter texts win ties: the match covers more of them.
  return score - t.length * 0.01;
}
