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
pub(crate) fn parse_local_start_scene_scope(pair: Pair<Rule>) -> Result<LocalSceneScope, ParseError> {
    let span = Span::from(&pair.as_span());
    let mut scope = LocalSceneScope::start();
    scope.span = span;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::var_set_local | Rule::var_set_global => {
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
            Rule::var_set_local | Rule::var_set_global => {
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
            Rule::code_block => {
                scope.code_blocks.push(parse_code_block(inner)?);
            }
            _ => {}
        }
    }

    Ok(scope)
}
