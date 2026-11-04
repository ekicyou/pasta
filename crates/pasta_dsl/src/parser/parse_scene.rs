//! Scene parsing functions for global and local scene scopes.

use super::*;

/// Parse global scene scope.
pub(crate) fn parse_global_scene_scope(
    pair: Pair<Rule>,
    last_name: &mut Option<String>,
    filename: &str,
) -> Result<GlobalSceneScope, ParseError> {
    let span = Span::from_pair(&pair);
    let mut scene_name = String::new();
    let mut is_continuation = false;
    let mut attrs = Vec::new();
    let mut words = Vec::new();
    let mut actors = Vec::new();
    let mut code_blocks = Vec::new();
    let mut local_scenes = Vec::new();
    let mut next_actor_number: u32 = 0;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::global_scene_start => {
                // 宣言行の属性が先、初期部の属性行が後に積まれる
                let (name, cont) =
                    parse_global_scene_start(inner, last_name, &mut attrs, filename)?;
                scene_name = name.clone();
                is_continuation = cont;
                *last_name = Some(name);
            }
            Rule::global_scene_attr_line => {
                for attr_pair in inner.into_inner() {
                    if attr_pair.as_rule() == Rule::attr {
                        attrs.push(parse_attr(attr_pair, filename)?);
                    }
                }
            }
            Rule::global_scene_word_line => {
                for kw_pair in inner.into_inner() {
                    if kw_pair.as_rule() == Rule::key_words {
                        words.push(parse_key_words(kw_pair, filename)?);
                    }
                }
            }
            Rule::scene_actors_line => {
                let items = parse_scene_actors_line(inner, &mut next_actor_number, filename)?;
                actors.extend(items);
            }
            Rule::code_block => {
                code_blocks.push(parse_code_block(inner)?);
            }
            Rule::local_start_scene_scope => {
                local_scenes.push(parse_local_start_scene_scope(inner, filename)?);
            }
            Rule::local_scene_scope => {
                local_scenes.push(parse_local_scene_scope(inner, filename)?);
            }
            _ => {}
        }
    }

    Ok(GlobalSceneScope {
        name: scene_name,
        is_continuation,
        attrs,
        words,
        actors,
        code_blocks,
        local_scenes,
        span,
    })
}

/// Parse global scene start (line or continue line).
///
/// 宣言行の属性は `attrs` に積む。名前なしの `＊` は名前だけを受け継ぎ、
/// 直前のシーンの属性は受け継がない。
pub(crate) fn parse_global_scene_start(
    pair: Pair<Rule>,
    last_name: &Option<String>,
    attrs: &mut Vec<Attr>,
    filename: &str,
) -> Result<(String, bool), ParseError> {
    let mut result = (String::new(), false);
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::global_scene_line => {
                // Named scene
                for scene_inner in inner.into_inner() {
                    match scene_inner.as_rule() {
                        Rule::id => result = (scene_inner.as_str().to_string(), false),
                        Rule::attr => attrs.push(parse_attr(scene_inner, filename)?),
                        _ => {}
                    }
                }
            }
            Rule::global_scene_continue_line => {
                // Continuation scene - inherit name from last
                let Some(name) = last_name else {
                    let span = inner.as_span();
                    let (line, col) = span.start_pos().line_col();
                    return Err(ParseError::SyntaxError {
                        file: filename.to_string(),
                        line,
                        column: col,
                        message: "Unnamed global scene at start of file. A named global scene must appear before any unnamed scenes.".to_string(),
                    });
                };
                result = (name.clone(), true);
                for scene_inner in inner.into_inner() {
                    if scene_inner.as_rule() == Rule::attr {
                        attrs.push(parse_attr(scene_inner, filename)?);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(result)
}

/// Parse scene_actors_line to extract SceneActorItems.
///
/// grammar.pest:
/// - `scene_actors_line = { pad ~ actor_marker ~ actors ~ or_comment_eol }`
/// - `actors = _{ actors_item ~ ( comma_sep ~ actors_item )* ~ comma_sep? }`
///
/// # Arguments
/// * `pair` - Rule::scene_actors_lineのPair
/// * `next_number` - 次の採番値（可変参照、更新される）
/// * `filename` - 誤りに書くファイル名
///
/// # Returns
/// パースされたSceneActorItemのベクタ
pub(crate) fn parse_scene_actors_line(
    pair: Pair<Rule>,
    next_number: &mut u32,
    filename: &str,
) -> Result<Vec<SceneActorItem>, ParseError> {
    let mut items = Vec::new();

    // scene_actors_line = { pad ~ actor_marker ~ actors ~ or_comment_eol }
    // actors is a silent rule, so actors_item pairs appear directly
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::actors_item {
            let item = parse_actors_item(inner, next_number, filename)?;
            items.push(item);
        }
    }

    Ok(items)
}

/// Parse a single actors_item to SceneActorItem.
///
/// grammar.pest: `actors_item = { id ~ ( s ~ set_marker ~ s ~ digit_id )? }`
///
/// C#のenum採番ルール:
/// - 番号指定あり: その番号を使用し、next_number = その番号 + 1
/// - 番号指定なし: next_numberを使用し、next_number += 1
/// - 0〜4294967295 に収まらない番号は、番号の位置の構文の誤り
pub(crate) fn parse_actors_item(
    pair: Pair<Rule>,
    next_number: &mut u32,
    filename: &str,
) -> Result<SceneActorItem, ParseError> {
    let span = Span::from_pair(&pair);
    let mut name = String::new();
    let mut explicit_number: Option<u32> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                name = inner.as_str().to_string();
            }
            Rule::digit_id => {
                // 全角数字を半角に正規化してパース
                let normalized = normalize_number_str(inner.as_str());
                // 文法が数字だけを通すので、失敗は範囲の外だけ
                let n = normalized.parse::<u32>().map_err(|_| {
                    syntax_error(
                        filename,
                        inner.line_col(),
                        &format!(
                            "アクターの番号 {} は 0〜4294967295 の範囲を超えています",
                            inner.as_str()
                        ),
                    )
                })?;
                explicit_number = Some(n);
            }
            _ => {}
        }
    }

    // C#のenum採番ルールを適用
    // 飽和加算: 明示番号 u32::MAX などでの整数オーバーフロー panic を防止
    // （不正入力ハードニング。通常範囲の番号では挙動不変）
    let number = explicit_number.unwrap_or(*next_number);
    *next_number = number.saturating_add(1);

    Ok(SceneActorItem { name, number, span })
}

/// Parse local start scene scope (no name).
pub(crate) fn parse_local_start_scene_scope(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<LocalSceneScope, ParseError> {
    let span = Span::from_pair(&pair);
    let mut scope = LocalSceneScope::start();
    scope.span = span;

    for inner in pair.into_inner() {
        parse_local_scene_item(inner, &mut scope, filename)?;
    }

    Ok(scope)
}

/// Parse local scene scope (with name).
pub(crate) fn parse_local_scene_scope(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<LocalSceneScope, ParseError> {
    let span = Span::from_pair(&pair);
    let mut scope = LocalSceneScope::start();
    scope.span = span;

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::local_scene_line {
            for scene_inner in inner.into_inner() {
                match scene_inner.as_rule() {
                    Rule::id => scope.name = Some(scene_inner.as_str().to_string()),
                    Rule::attr => scope.attrs.push(parse_attr(scene_inner, filename)?),
                    _ => {}
                }
            }
        } else {
            parse_local_scene_item(inner, &mut scope, filename)?;
        }
    }

    Ok(scope)
}

/// Parse a single local scene item and push it into the scope.
fn parse_local_scene_item(
    inner: Pair<Rule>,
    scope: &mut LocalSceneScope,
    filename: &str,
) -> Result<(), ParseError> {
    match inner.as_rule() {
        Rule::var_set_local
        | Rule::var_set_global
        | Rule::var_set_none
        | Rule::var_set_property => {
            scope
                .items
                .push(LocalSceneItem::VarSet(parse_var_set(inner, filename)?));
        }
        Rule::call_scene => {
            scope.items.push(LocalSceneItem::CallScene(parse_call_scene(
                inner, filename,
            )?));
        }
        Rule::action_line => {
            scope
                .items
                .push(LocalSceneItem::ActionLine(parse_action_line(
                    inner, filename,
                )?));
        }
        Rule::continue_action_line => {
            scope
                .items
                .push(LocalSceneItem::ContinueAction(parse_continue_action_line(
                    inner, filename,
                )?));
        }
        Rule::cue_cmd_line => {
            scope
                .items
                .push(LocalSceneItem::CueCommand(parse_cue_cmd_line(
                    inner, filename,
                )?));
        }
        Rule::choice_line => {
            scope
                .items
                .push(LocalSceneItem::Choice(parse_choice_line(inner)?));
        }
        Rule::code_block => {
            scope.code_blocks.push(parse_code_block(inner)?);
        }
        _ => {}
    }
    Ok(())
}

// ============================================================================
// Choice Line Parsing
// ============================================================================

/// Parse a `choice_line` pair into a `ChoiceNode`.
///
/// grammar.pest:
/// `choice_line = { pad ~ word_marker ~ question_marker ~ id ~ choice_label? ~ or_comment_eol }`
/// `choice_label = { slfence_ja1 ~ string_contents ~ strclose }`
pub(crate) fn parse_choice_line(pair: Pair<Rule>) -> Result<ChoiceNode, ParseError> {
    let span = Span::from_pair(&pair);
    let mut target = String::new();
    let mut label = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::id => {
                target = inner.as_str().to_string();
            }
            Rule::choice_label => {
                for label_inner in inner.into_inner() {
                    if label_inner.as_rule() == Rule::string_contents {
                        label = Some(label_inner.as_str().to_string());
                    }
                }
            }
            _ => {}
        }
    }

    Ok(ChoiceNode {
        target,
        label,
        span,
    })
}

// ============================================================================
// Cue Command Line Parsing
// ============================================================================

/// Parse a `cue_cmd_line` pair into a `CueCommandNode`.
///
/// grammar.pest:
/// `cue_cmd_line = { pad ~ cue_cmd_marker ~ cue_cmd_name ~ cue_cmd_scope? ~ cue_cmd_args? ~ or_comment_eol }`
pub(crate) fn parse_cue_cmd_line(
    pair: Pair<Rule>,
    filename: &str,
) -> Result<CueCommandNode, ParseError> {
    let span = Span::from_pair(&pair);
    let mut command = String::new();
    let mut scope = None;
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::cue_cmd_name => {
                command = inner.as_str().to_string();
            }
            Rule::cue_cmd_scope => {
                scope = Some(parse_cue_cmd_scope(inner)?);
            }
            Rule::cue_cmd_args => {
                args = parse_cue_cmd_args(inner, filename)?;
            }
            _ => {}
        }
    }

    Ok(CueCommandNode {
        command,
        scope,
        args,
        span,
    })
}

/// Parse a `cue_cmd_scope` pair into a `ScopedName`.
///
/// grammar.pest:
/// `cue_cmd_scope = { at ~ cue_scoped_ident }`
/// `cue_scoped_ident = @{ cue_ident_part ~ ( colon ~ cue_ident_part )? }`
///
/// `cue_scoped_ident` is atomic, so we get the full text and split on `:`.
fn parse_cue_cmd_scope(pair: Pair<Rule>) -> Result<ScopedName, ParseError> {
    let span = Span::from_pair(&pair);

    // `at` は黙る規則なので、子の文は `cue_scoped_ident` だけになる
    let raw = pair.into_inner().as_str();
    let (actor, name) = match raw.split_once([':', '：']) {
        Some((actor, name)) => (Some(actor.to_string()), name),
        None => (None, raw),
    };
    Ok(ScopedName {
        actor,
        name: name.to_string(),
        span,
    })
}

/// Parse `cue_cmd_args` into a `Vec<CueArgToken>`.
///
/// grammar.pest:
/// `cue_cmd_args = { lparen ~ s ~ cue_arg_list? ~ s ~ rparen }`
/// `cue_arg_list = _{ cue_arg ~ ( comma_sep ~ cue_arg )* }`
/// `cue_arg = _{ cue_arg_at_ref | number_literal | string_literal | cue_arg_id }`
///
/// Since `cue_arg_list` and `cue_arg` are silent rules, the inner pairs of
/// `cue_cmd_args` directly contain the matched argument rules.
fn parse_cue_cmd_args(pair: Pair<Rule>, filename: &str) -> Result<Vec<CueArgToken>, ParseError> {
    let mut tokens = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::cue_arg_at_ref => {
                // cue_arg_at_ref = { at ~ id }
                for id_pair in inner.into_inner() {
                    if id_pair.as_rule() == Rule::id {
                        tokens.push(CueArgToken::AtRef(id_pair.as_str().to_string()));
                    }
                }
            }
            Rule::number_literal => {
                tokens.push(match number_value(&inner, filename)? {
                    Number::Integer(i) => CueArgToken::Integer(i),
                    Number::Float(f) => CueArgToken::Float(f),
                });
            }
            Rule::string_contents => {
                tokens.push(CueArgToken::StringLiteral(inner.as_str().to_string()));
            }
            Rule::string_blank => {
                tokens.push(CueArgToken::StringLiteral(String::new()));
            }
            Rule::cue_arg_id => {
                tokens.push(CueArgToken::Ident(inner.as_str().to_string()));
            }
            _ => {}
        }
    }

    Ok(tokens)
}
