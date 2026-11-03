//! ファイル属性の受け継ぎと宣言の重なりの決まり（この規則の唯一の実装）

use super::{Attr, AttrBody, FileItem, PastaFile};

/// `inherited` → `own` の順に並べる。宣言は同じ名前を先の位置に 1 つ残して値は後のもの、
/// 条件とフラグは畳まずに足す。
pub fn merge_attrs(inherited: &[Attr], own: &[Attr]) -> Vec<Attr> {
    let mut merged: Vec<Attr> = Vec::with_capacity(inherited.len() + own.len());
    for attr in inherited.iter().chain(own) {
        if matches!(attr.body, AttrBody::Decl(_))
            && let Some(prev) = merged
                .iter_mut()
                .find(|m| matches!(m.body, AttrBody::Decl(_)) && m.key == attr.key)
        {
            prev.body = attr.body.clone();
            continue;
        }
        merged.push(attr.clone());
    }
    merged
}

impl PastaFile {
    /// n 番目は items の中の n 番目の GlobalSceneScope の属性（受け継ぎと畳みの後・書いた順）
    pub fn global_scene_attrs(&self) -> Vec<Vec<Attr>> {
        let mut file_attrs: Vec<Attr> = Vec::new();
        // グローバルシーンの後に来た最初の FileAttr で組を捨てる
        let mut applied = false;
        let mut result = Vec::new();
        for item in &self.items {
            match item {
                FileItem::FileAttr(attr) => {
                    if applied {
                        file_attrs.clear();
                        applied = false;
                    }
                    file_attrs.push(attr.clone());
                }
                FileItem::GlobalSceneScope(scene) => {
                    applied = true;
                    result.push(merge_attrs(&file_attrs, &scene.attrs));
                }
                FileItem::GlobalWord(_) | FileItem::ActorScope(_) => {}
            }
        }
        result
    }
}
