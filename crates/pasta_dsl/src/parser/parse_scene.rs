//! Scene parsing functions for global and local scene scopes.

use super::*;

/// Parse global scene scope.
pub(crate) fn parse_global_scene_scope(
    pair: Pair<Rule>,
    last_name: &mut Option<String>,
    filename: &str,
) -> Result<GlobalSceneScope, ParseError> {
    let span = Span::from(&pair.as_span());
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
                let (name, cont) = parse_global_scene_start(inner, last_name, filename)?;
                scene_name = name.clone();
                is_continuation = cont;
                *last_name = Some(name);
            }
            Rule::global_scene_attr_line => {
                for attr_pair in inner.into_inner() {
                    if attr_pair.as_rule() == Rule::attr {
                        attrs.push(parse_attr(attr_pair)?);
                    }
                }
            }
            Rule::global_scene_word_line => {
                for kw_pair in inner.into_inner() {
                    if kw_pair.as_rule() == Rule::key_words {
                        words.push(parse_key_words(kw_pair)?);
                    }
                }
            }
            Rule::scene_actors_line => {
                let items = parse_scene_actors_line(inner, &mut next_actor_number)?;
                actors.extend(items);
            }
            Rule::code_block => {
                code_blocks.push(parse_code_block(inner)?);
            }
            Rule::local_start_scene_scope => {
                local_scenes.push(parse_local_start_scene_scope(inner)?);
            }
            Rule::local_scene_scope => {
                local_scenes.push(parse_local_scene_scope(inner)?);
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
pub(crate) fn parse_global_scene_start(
    pair: Pair<Rule>,
    last_name: &Option<String>,
    filename: &str,
) -> Result<(String, bool), ParseError> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::global_scene_line => {
                // Named scene
                for scene_inner in inner.into_inner() {
                    if scene_inner.as_rule() == Rule::id {
                        return Ok((scene_inner.as_str().to_string(), false));
                    }
                }
            }
            Rule::global_scene_continue_line => {
                // Continuation scene - inherit name from last
                if let Some(name) = last_name {
                    return Ok((name.clone(), true));
                } else {
                    let span = inner.as_span();
                    let (line, col) = span.start_pos().line_col();
                    return Err(ParseError::SyntaxError {
                        file: filename.to_string(),
                        line,
                        column: col,
                        message: "Unnamed global scene at start of file. A named global scene must appear before any unnamed scenes.".to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    Ok((String::new(), false))
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
///
/// # Returns
/// パースされたSceneActorItemのベクタ
pub(crate) fn parse_scene_actors_line(
    pair: Pair<Rule>,
    next_number: &mut u32,
) -> Result<Vec<SceneActorItem>, ParseError> {
    let mut items = Vec::new();

    // scene_actors_line = { pad ~ actor_marker ~ actors ~ or_comment_eol }
    // actors is a silent rule, so actors_item pairs appear directly
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::actors_item {
            let item = parse_actors_item(inner, next_number)?;
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
pub(crate) fn parse_actors_item(
    pair: Pair<Rule>,
    next_number: &mut u32,
) -> Result<SceneActorItem, ParseError> {
    let span = Span::from(&pair.as_span());
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
                explicit_number = normalized.parse::<u32>().ok();
            }
            _ => {}
        }
    }

    // C#のenum採番ルールを適用
    let number = if let Some(n) = explicit_number {
        *next_number = n + 1;
        n
    } else {
        let n = *next_number;
        *next_number += 1;
        n
    };

    Ok(SceneActorItem { name, number, span })
}

/// Parse local start scene scope (no name).
pub(crate) fn parse_local_start_scene_scope(
    pair: Pair<Rule>,
) -> Result<LocalSceneScope, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut scope = LocalSceneScope::start();
    scope.span = span;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::var_set_local
            | Rule::var_set_global
            | Rule::var_set_none
            | Rule::var_set_property => {
                scope
                    .items
                    .push(LocalSceneItem::VarSet(parse_var_set(inner)?));
            }
            Rule::call_scene => {
                scope
                    .items
                    .push(LocalSceneItem::CallScene(parse_call_scene(inner)?));
            }
            Rule::action_line => {
                scope
                    .items
                    .push(LocalSceneItem::ActionLine(parse_action_line(inner)?));
            }
            Rule::continue_action_line => {
                scope
                    .items
                    .push(LocalSceneItem::ContinueAction(parse_continue_action_line(
                        inner,
                    )?));
            }
            Rule::cue_cmd_line => {
                scope
                    .items
                    .push(LocalSceneItem::CueCommand(parse_cue_cmd_line(inner)?));
            }
            Rule::code_block => {
                scope.code_blocks.push(parse_code_block(inner)?);
            }
            _ => {}
        }
    }

    Ok(scope)
}

/// Parse local scene scope (with name).
pub(crate) fn parse_local_scene_scope(pair: Pair<Rule>) -> Result<LocalSceneScope, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut scope = LocalSceneScope::start();
    scope.span = span;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::local_scene_line => {
                for scene_inner in inner.into_inner() {
                    if scene_inner.as_rule() == Rule::id {
                        scope.name = Some(scene_inner.as_str().to_string());
                        break;
                    }
                }
            }
            Rule::var_set_local
            | Rule::var_set_global
            | Rule::var_set_none
            | Rule::var_set_property => {
                scope
                    .items
                    .push(LocalSceneItem::VarSet(parse_var_set(inner)?));
            }
            Rule::call_scene => {
                scope
                    .items
                    .push(LocalSceneItem::CallScene(parse_call_scene(inner)?));
            }
            Rule::action_line => {
                scope
                    .items
                    .push(LocalSceneItem::ActionLine(parse_action_line(inner)?));
            }
            Rule::continue_action_line => {
                scope
                    .items
                    .push(LocalSceneItem::ContinueAction(parse_continue_action_line(
                        inner,
                    )?));
            }
            Rule::cue_cmd_line => {
                scope
                    .items
                    .push(LocalSceneItem::CueCommand(parse_cue_cmd_line(inner)?));
            }
            Rule::code_block => {
                scope.code_blocks.push(parse_code_block(inner)?);
            }
            _ => {}
        }
    }

    Ok(scope)
}

// ============================================================================
// Cue Command Line Parsing
// ============================================================================

/// Parse a `cue_cmd_line` pair into a `CueCommandNode`.
///
/// grammar.pest:
/// `cue_cmd_line = { pad ~ cue_cmd_marker ~ cue_cmd_name ~ cue_cmd_scope? ~ cue_cmd_args? ~ or_comment_eol }`
pub(crate) fn parse_cue_cmd_line(pair: Pair<Rule>) -> Result<CueCommandNode, ParseError> {
    let span = Span::from(&pair.as_span());
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
                args = parse_cue_cmd_args(inner)?;
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
    let span = Span::from(&pair.as_span());

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::cue_scoped_ident {
            let raw = inner.as_str();
            // Split on ':' or '：' (full-width colon) to separate actor and name
            if let Some(colon_pos) = raw.find(|c: char| c == ':' || c == '：') {
                let actor = &raw[..colon_pos];
                let colon_char = raw[colon_pos..].chars().next().unwrap();
                let name = &raw[colon_pos + colon_char.len_utf8()..];
                return Ok(ScopedName {
                    actor: Some(actor.to_string()),
                    name: name.to_string(),
                    span,
                });
            } else {
                return Ok(ScopedName {
                    actor: None,
                    name: raw.to_string(),
                    span,
                });
            }
        }
    }

    // Fallback (should not be reached due to PEG grammar guarantees)
    Ok(ScopedName {
        actor: None,
        name: String::new(),
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
fn parse_cue_cmd_args(pair: Pair<Rule>) -> Result<Vec<CueArgToken>, ParseError> {
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
                let normalized = normalize_number_str(inner.as_str());
                if normalized.contains('.') {
                    tokens.push(CueArgToken::Float(normalized.parse().unwrap_or(0.0)));
                } else {
                    tokens.push(CueArgToken::Integer(normalized.parse().unwrap_or(0)));
                }
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
