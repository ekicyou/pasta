# Requirements Document

## Project Description (Input)
「＞カウント表示（＄カウンタ）」の展開結果が期待と異なる。()でくくられた場合は「同名の関数をそのまま呼び出す」挙動であるべき。シーン呼び出しではなく、関数呼び出しになる必要がある。

「＞カウント表示（＄カウンタ）」　⇒「for a in カウント表示(ctx, [ctx.local.カウンタ]) { yield a; }」でなければならない。

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
