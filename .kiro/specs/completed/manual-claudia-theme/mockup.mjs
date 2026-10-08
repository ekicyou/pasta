// 要件ディスカッション議題 2 で承認した見た目のモックアップ生成器。
// 使い方: ponapalt/claudia の site/img/f{0,1,5,9,25,26,10,11}.png を 1 つのディレクトリに置き、
//   node mockup.mjs <icons-dir> <out.html>
// 生成物は顔アイコンを data URI で埋め込んだ単一 HTML（右上ボタンで navy 切り替え）。
import { readFileSync, writeFileSync } from 'node:fs';
const dir = process.argv[2], out = process.argv[3];
const img = (n) => `data:image/png;base64,${readFileSync(`${dir}/${n}.png`).toString('base64')}`;
const FACE = { 素: 'f0', 照れ: 'f1', 高笑い: 'f5', 照れ怒り: 'f9', にっこり: 'f25', したり顔: 'f26' };
const AFACE = { 素: 'f10', 刮目: 'f11' };
const talk = (who, face, html) => {
  const a = who === 'anthony';
  const src = img(a ? AFACE[face] : FACE[face]);
  const name = a ? 'Anthony' : 'Claudia';
  return `<div class="talk ${who}"><img class="face" src="${src}" alt="${a ? 'アンソニー' : 'クローディア'}（${face}）" width="56" height="56"><div class="bubble"><span class="who">${name}</span>${html}</div></div>`;
};
const C = (f, t) => talk('claudia', f, t), A = (f, t) => talk('anthony', f, t);
const divider = `<div class="divider" aria-hidden="true"><span></span><i></i><span></span></div>`;

const html = `<!doctype html>
<html lang="ja"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Claudia テーマ モックアップ</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=BIZ+UDPGothic:wght@400;700&family=BIZ+UDPMincho&family=Cormorant+Garamond:ital,wght@1,500&family=Shippori+Mincho+B1:wght@600;800&display=swap" rel="stylesheet">
<style>
:root{--paper:#FAF5EA;--paper2:#F0E4CE;--edge:#E3D2B3;--ink:#3A2419;--ink2:#6B4E3D;--accent:#B4532F;--gold:#B98D4A;--wax:#6D1F2C;
 --bubble:#FFFDF6;--c-ink:#6A2A26;--c-face:#F0E4CE;--c-ring:#B98D4A;--a-ink:#2C3E66;--a-face:#EEF2FA;--a-ring:#8FA3C8;--code:#F3EAD6;--sidebar:#F0E4CE}
.navy{--paper:#1A1E2B;--paper2:#232838;--edge:#3A4157;--ink:#E9E1D1;--ink2:#B8AE9C;--accent:#E0866A;--gold:#C9A467;--wax:#C96B7C;
 --bubble:#242A3B;--c-ink:#F0B1A2;--c-face:#3A3040;--c-ring:#C9A467;--a-ink:#AFC3F2;--a-face:#242F4A;--a-ring:#6F86B8;--code:#141827;--sidebar:#141827}
*{box-sizing:border-box}
body{margin:0;background:var(--paper);color:var(--ink);font-family:"BIZ UDPGothic","Hiragino Sans","Yu Gothic UI",system-ui,sans-serif;font-size:16px;line-height:1.85;transition:background .2s}
.bar{position:sticky;top:0;display:flex;gap:12px;align-items:center;padding:8px 16px;background:var(--sidebar);border-bottom:1px solid var(--edge);font-size:14px;z-index:2}
.bar b{font-family:"Shippori Mincho B1",serif;font-weight:800;letter-spacing:.06em}
.bar button{margin-left:auto;background:var(--bubble);color:var(--ink);border:1px solid var(--edge);border-radius:4px;padding:4px 10px;cursor:pointer;font:inherit;font-size:13px}
.bar .latin{font-family:"Cormorant Garamond",serif;font-style:italic;color:var(--ink2);font-size:15px}
main{max-width:760px;margin:0 auto;padding:24px 16px 64px}
h1,h2{font-family:"Shippori Mincho B1","Yu Mincho",serif;font-weight:800;letter-spacing:.04em;line-height:1.4}
h1{font-size:1.9rem;margin:.4em 0 .6em;border-bottom:2px solid var(--gold);padding-bottom:.25em}
h2{font-size:1.3rem;margin:1.6em 0 .5em;padding-left:.6em;border-left:4px solid var(--accent)}
p{margin:.8em 0}
a{color:var(--accent)}
code{font-family:"Source Code Pro",Consolas,monospace;background:var(--code);padding:.1em .35em;border-radius:4px;font-size:.92em}
pre{background:var(--code);border:1px solid var(--edge);border-radius:10px;padding:14px 16px;overflow:auto;box-shadow:0 1px 0 rgba(58,36,25,.06),0 3px 10px rgba(58,36,25,.08)}
pre code{background:none;padding:0;font-size:.9em;line-height:1.6}
.k{color:#8A3B1E}.s{color:#2F6B4F}.v{color:#5A4FA0}.cm{color:var(--ink2);font-style:italic}.n{color:var(--wax)}
.navy .k{color:#F0A080}.navy .s{color:#9FD6B3}.navy .v{color:#BDB3F5}
table{border-collapse:collapse;width:100%;font-size:.95em}th,td{border:1px solid var(--edge);padding:6px 10px;text-align:left}th{background:var(--paper2)}
blockquote.note{margin:1em 0;padding:.6em 1em;border-left:4px solid var(--gold);background:var(--paper2);border-radius:0 10px 10px 0;color:var(--ink2)}
.divider{display:flex;align-items:center;gap:10px;margin:28px 0}.divider span{flex:1;height:1px;background:var(--gold);opacity:.7}.divider i{width:9px;height:9px;background:var(--gold);transform:rotate(45deg)}
/* 台詞部品 */
.talk{display:flex;gap:12px;align-items:flex-start;margin:14px 0;max-width:92%}
.talk .face{flex:none;width:56px;height:56px;border-radius:50%;object-fit:cover;border:2px solid var(--c-ring);background:var(--c-face);box-shadow:0 1px 2px rgba(58,36,25,.15)}
.talk .bubble{position:relative;flex:1;background:var(--bubble);border:1px solid var(--edge);border-radius:10px;padding:10px 14px;font-family:"BIZ UDPMincho","Yu Mincho","Hiragino Mincho ProN",serif;font-size:1.02em;color:var(--c-ink);box-shadow:0 1px 0 rgba(58,36,25,.05),0 3px 10px rgba(58,36,25,.07)}
.talk .bubble::before{content:"";position:absolute;top:18px;left:-7px;width:12px;height:12px;background:var(--bubble);border-left:1px solid var(--edge);border-bottom:1px solid var(--edge);transform:rotate(45deg)}
.talk .who{position:absolute;top:-10px;left:12px;font-family:"Cormorant Garamond",serif;font-style:italic;font-size:.8em;color:var(--gold);background:var(--bubble);padding:0 6px;line-height:1.2}
.talk.anthony{flex-direction:row-reverse;margin-left:auto}
.talk.anthony .face{border-color:var(--a-ring);background:var(--a-face)}
.talk.anthony .bubble{color:var(--a-ink)}
.talk.anthony .bubble::before{left:auto;right:-7px;transform:rotate(-135deg)}
.talk.anthony .who{left:auto;right:12px;color:var(--a-ring)}
.talk .bubble code{font-family:"Source Code Pro",Consolas,monospace;color:var(--ink)}
/* 表紙の扉 */
.hero{position:relative;background:var(--paper2);border:1px solid var(--edge);border-radius:14px;padding:28px 28px 20px;margin:8px 0 28px;box-shadow:0 1px 0 rgba(58,36,25,.05),0 6px 18px rgba(58,36,25,.10)}
.hero .corner{position:absolute;width:18px;height:18px;border-color:var(--gold);border-style:solid;border-width:0}
.hero .tl{top:8px;left:8px;border-top-width:2px;border-left-width:2px}.hero .tr{top:8px;right:8px;border-top-width:2px;border-right-width:2px}
.hero .bl{bottom:8px;left:8px;border-bottom-width:2px;border-left-width:2px}.hero .br{bottom:8px;right:8px;border-bottom-width:2px;border-right-width:2px}
.hero .title{font-family:"Shippori Mincho B1",serif;font-weight:800;font-size:1.7rem;text-align:center;margin:0}
.hero .latin{display:block;text-align:center;font-family:"Cormorant Garamond",serif;font-style:italic;color:var(--ink2);font-size:1.05rem;margin-bottom:10px}
.hero .duo{display:flex;justify-content:center;gap:18px;margin:8px 0 14px}
.hero .duo img{width:84px;height:84px;border-radius:50%;border:3px solid var(--c-ring);background:var(--c-face)}
.hero .duo img.a{border-color:var(--a-ring);background:var(--a-face)}
.hero .bubble{background:var(--bubble)}
.toc{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:8px;margin-top:14px}
.toc a{display:block;background:var(--bubble);border:1px solid var(--edge);border-radius:10px;padding:8px 10px;text-decoration:none;font-size:.9em;color:var(--ink)}
.toc a small{display:block;color:var(--ink2);font-family:"Cormorant Garamond",serif;font-style:italic}
.credit{font-size:.8em;color:var(--ink2);text-align:center;margin-top:40px}
.caption{font-size:.8em;color:var(--ink2);text-align:center;margin:-8px 0 20px}
@media (max-width:480px){.talk{max-width:100%}.talk .face{width:44px;height:44px}main{padding:16px 12px}}
</style></head><body>
<div class="bar"><b>pasta</b><span class="latin">— Claudia et Anthony</span><button onclick="document.body.classList.toggle('navy');this.textContent=document.body.classList.contains('navy')?'☀ ライト':'☾ ダーク(navy)'">☾ ダーク(navy)</button></div>
<main>
<p class="caption">モックアップ（1/2）: 表紙の扉 — 立ち絵なし、顔アイコン二人で迎える</p>
<section class="hero"><i class="corner tl"></i><i class="corner tr"></i><i class="corner bl"></i><i class="corner br"></i>
<h1 class="title" style="border:0;padding:0">pasta 利用者マニュアル</h1>
<span class="latin">Claudia et Anthony — ゴーストの作り方、二人でご案内いたしますわ</span>
<div class="duo"><img src="${img('f25')}" alt="クローディア"><img class="a" src="${img('f10')}" alt="アンソニー"></div>
${C('高笑い', 'ようこそいらっしゃいまし。わたくしがこのマニュアルの案内役、クローディアですわ。pasta で<strong>あなただけのゴースト</strong>を、熱く作りこんでいきますわよ！')}
${A('素', '執事のアンソニーでございます。お嬢様が走りすぎましたら、わたくしが手綱を引きますので、どうぞご安心を。')}
<div class="toc"><a href="#">入門／チュートリアル<small>Getting started</small></a><a href="#">Pasta DSL 文法<small>Grammar</small></a><a href="#">Lua API／コーディング<small>Lua</small></a><a href="#">デバッグ<small>Debug</small></a><a href="#">リファレンス<small>Reference</small></a><a href="#">内部設計<small>Internals</small></a></div>
</section>

<p class="caption">モックアップ（2/2）: ふつうの章 — 導入（掛け合い）→ 本文（ゴシック）→ 締め（掛け合い）</p>
<h1>トーク（会話）を書く</h1>
${C('高笑い', 'さあ、今日はいよいよトークを書きますわよ。ゴーストは喋ってこそ、ですもの！')}
${A('素', 'お嬢様、まずは<strong>一行だけ</strong>でございます。昨日のように欲張られますと、辞書が読み込めなくなりますので。')}
${C('照れ怒り', 'フンッ、あれはちょっと張り切っただけですわ！ ……一行から、ですわね。')}
${divider}
<p>トークは <code>＠</code> で始まるシーン名の下に、話し手と台詞を書く。1 行が 1 つの発話になり、ゴーストはその順に話す。</p>
<pre><code><span class="cm">＃ はじめてのトーク</span>
<span class="k">＠</span><span class="n">あいさつ</span>
　<span class="v">さくら</span>：こんにちは、わたしはさくらです。
　<span class="v">うにゅう</span>：<span class="s">おう。</span></code></pre>
<h2>話し手を切り替える</h2>
<p>台詞の前に書いた名前が話し手になる。名前を省略すると、直前の話し手が続けて話す。詳しくは<a href="#">アクター辞書</a>を参照。</p>
<table><tr><th>書き方</th><th>意味</th></tr><tr><td><code>さくら：…</code></td><td>さくらが話す</td></tr><tr><td><code>　：…</code></td><td>直前の話し手が続ける</td></tr></table>
<blockquote class="note">将来変更あり: 名前の省略規則は v0.4 で見直す予定。</blockquote>
${divider}
${A('刮目', 'おや。一度もつまずかずにお書きになりましたね。')}
${C('にっこり', '当然ですわ。……次の章では、この子たちに<strong>表情</strong>も付けてあげますわよ。')}
<p class="credit">顔アイコン・意匠: <a href="#">ponapalt/claudia</a>（Unlicense）／ 参考: 悪役令嬢クローディア 紹介ページ</p>
</main></body></html>`;
writeFileSync(out, html);
console.log('wrote', out, Buffer.byteLength(html), 'bytes');
