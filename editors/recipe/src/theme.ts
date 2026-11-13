// 純粋: TextMate のテーマに、セマンティックトークンの種類ごとの色の規則を足す（DOM・Monaco を import しない）
import type { SemanticToken } from "./ipc.ts";

export interface TmStyle {
  foreground?: string;
  fontStyle?: string;
}
export interface TmThemeRule {
  scope?: string | string[];
  settings?: TmStyle;
}
export interface TmTheme {
  name: string;
  type: "light" | "dark";
  colors?: Record<string, string>;
  tokenColors: TmThemeRule[];
}
export type SemanticTokenScopes = Readonly<Record<string, readonly string[]>>;

/** テーマに、セマンティックトークンの種類ごとの規則（scope に種類の名前・色は写し先のスコープの色）を足した新しいテーマを返す */
export function withSemanticRules(theme: TmTheme, scopes: SemanticTokenScopes): TmTheme {
  return { ...theme, tokenColors: [...theme.tokenColors, ...semanticRules(theme, scopes)] };
}

/** 足す規則だけ（scope は種類の名前）。写し先が色を持たない種類は含まない */
export function semanticRules(theme: TmTheme, scopes: SemanticTokenScopes): TmThemeRule[] {
  const added: TmThemeRule[] = [];
  for (const [kind, kindScopes] of Object.entries(scopes)) {
    const style = kindScopes.map((s) => resolveScopeStyle(theme, s)).find((s) => s?.foreground);
    if (style) added.push({ scope: kind, settings: style });
  }
  return added;
}

/**
 * `keep` が偽の種類のトークンを落とし、残りの差分を付け直す。
 * Monaco はテーマに規則の無い種類も既定の文字の色で塗り、下地の色を消すので、規則の無い種類は渡さない（拡張と同じ見た目・6.6）
 */
export function keepTokens(tokens: readonly SemanticToken[], keep: (tokenType: number) => boolean): SemanticToken[] {
  const out: SemanticToken[] = [];
  let line = 0;
  let char = 0;
  let keptLine = 0;
  let keptChar = 0;
  for (const t of tokens) {
    line += t.deltaLine;
    char = t.deltaLine ? t.deltaStartCharacter : char + t.deltaStartCharacter;
    if (!keep(t.tokenType)) continue;
    const deltaLine = line - keptLine;
    out.push({ ...t, deltaLine, deltaStartCharacter: deltaLine ? char : char - keptChar });
    keptLine = line;
    keptChar = char;
  }
  return out;
}

/** TextMate のスコープ 1 つの色: 規則の選択子（カンマで分け、空白を含む子孫の選択子は使わない）のうち、点で区切った先頭一致の最も長いもの */
export function resolveScopeStyle(theme: TmTheme, scope: string): TmStyle | undefined {
  // 色と字形は別々に解く（字形だけの規則が色を隠さない）。同じ長さなら後の規則が勝つ
  const best: Record<keyof TmStyle, { len: number; value?: string }> = {
    foreground: { len: -1 },
    fontStyle: { len: -1 },
  };
  for (const rule of theme.tokenColors) {
    if (!rule.scope || !rule.settings) continue;
    const selectors = (Array.isArray(rule.scope) ? rule.scope : [rule.scope])
      .flatMap((s) => s.split(","))
      .map((s) => s.trim());
    for (const sel of selectors) {
      if (sel === "" || /\s/.test(sel)) continue;
      if (scope !== sel && !scope.startsWith(sel + ".")) continue;
      for (const key of ["foreground", "fontStyle"] as const) {
        const value = rule.settings[key];
        if (value !== undefined && sel.length >= best[key].len) best[key] = { len: sel.length, value };
      }
    }
  }
  const out: TmStyle = {};
  if (best.foreground.value !== undefined) out.foreground = best.foreground.value;
  if (best.fontStyle.value !== undefined) out.fontStyle = best.fontStyle.value;
  return Object.keys(out).length ? out : undefined;
}
