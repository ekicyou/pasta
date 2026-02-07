//! pasta DSL スクリプト
//!
//! サンプルゴースト用の pasta DSL スクリプトは
//! `dist-src/ghost/master/dic/` に実ファイルとして配置されています。
//! release.ps1 の robocopy ステップでコピーされます。

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    /// dist-src ディレクトリのパスを取得
    fn dist_src_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist-src")
    }

    /// dist-src からスクリプトファイルを読み込む
    fn read_pasta_script(name: &str) -> String {
        let path = dist_src_dir().join("ghost/master/dic").join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} の読み込みに失敗: {}", name, e))
    }

    /// グローバルアクター辞書定義（行頭の`％actor_name`）が含まれているかチェック
    /// シーン内アクタースコープ（インデント付き`　％actor_name`）は検出しない
    fn contains_global_actor_dictionary(content: &str, actor_name: &str) -> bool {
        let pattern = format!("％{}", actor_name);
        content.starts_with(&pattern) || content.contains(&format!("\n{}", pattern))
    }

    #[test]
    fn test_actors_pasta_contains_all_characters() {
        let actors = read_pasta_script("actors.pasta");
        assert!(
            actors.contains("％女の子"),
            "女の子アクターがありません"
        );
        assert!(
            actors.contains("％男の子"),
            "男の子アクターがありません"
        );
        assert!(actors.contains("＠笑顔"), "笑顔表情がありません");
        assert!(actors.contains("＠通常"), "通常表情がありません");
        assert!(actors.contains("＠怒り"), "怒り表情がありません");
    }

    #[test]
    fn test_boot_pasta_contains_events() {
        let boot = read_pasta_script("boot.pasta");
        assert!(boot.contains("＊OnBoot"));
        assert!(boot.contains("＊OnFirstBoot"));
        assert!(boot.contains("＊OnClose"));
    }

    #[test]
    fn test_talk_pasta_contains_events() {
        let talk = read_pasta_script("talk.pasta");
        assert!(talk.contains("＊OnTalk"));
        assert!(talk.contains("＊OnHour"));
        assert!(talk.contains("＄時"));
    }

    #[test]
    fn test_click_pasta_contains_events() {
        let click = read_pasta_script("click.pasta");
        assert!(click.contains("＊OnMouseDoubleClick"));
        let count = click.matches("＊OnMouseDoubleClick").count();
        assert!(count >= 7, "ダブルクリック反応は7種以上必要: {}", count);
    }

    #[test]
    fn test_script_expression_names_defined_in_actors() {
        let actors = read_pasta_script("actors.pasta");

        fn extract_expression_names(script: &str) -> Vec<&str> {
            let mut names = Vec::new();
            let mut in_scene = false;
            for line in script.lines() {
                if line.starts_with("＊") {
                    in_scene = true;
                    continue;
                }
                if !in_scene {
                    continue;
                }
                if let Some(rest) = line.strip_prefix('　') {
                    if let Some(after_colon) = rest.split_once('：').map(|(_, r)| r) {
                        if let Some(name) = after_colon.strip_prefix('＠') {
                            let expr_name = name.split('　').next().unwrap_or(name);
                            if !expr_name.is_empty() {
                                names.push(expr_name);
                            }
                        }
                    }
                }
            }
            names
        }

        let scripts = [
            ("boot.pasta", read_pasta_script("boot.pasta")),
            ("talk.pasta", read_pasta_script("talk.pasta")),
            ("click.pasta", read_pasta_script("click.pasta")),
        ];

        for (name, script) in &scripts {
            let expression_names = extract_expression_names(script);
            assert!(
                !expression_names.is_empty(),
                "{} から表情名が1つも抽出されませんでした",
                name
            );
            for expr_name in &expression_names {
                assert!(
                    actors.contains(&format!("＠{}：", expr_name)),
                    "{} 内の表情名「＠{}」が actors.pasta に定義されていません",
                    name,
                    expr_name
                );
            }
        }
    }

    #[test]
    fn test_event_files_do_not_contain_global_actor_dictionary() {
        let boot = read_pasta_script("boot.pasta");
        let talk = read_pasta_script("talk.pasta");
        let click = read_pasta_script("click.pasta");

        assert!(
            !contains_global_actor_dictionary(&boot, "女の子"),
            "boot.pasta にグローバルアクター辞書定義が含まれています"
        );
        assert!(
            !contains_global_actor_dictionary(&boot, "男の子"),
            "boot.pasta にグローバルアクター辞書定義が含まれています"
        );
        assert!(
            !contains_global_actor_dictionary(&talk, "女の子"),
            "talk.pasta にグローバルアクター辞書定義が含まれています"
        );
        assert!(
            !contains_global_actor_dictionary(&talk, "男の子"),
            "talk.pasta にグローバルアクター辞書定義が含まれています"
        );
        assert!(
            !contains_global_actor_dictionary(&click, "女の子"),
            "click.pasta にグローバルアクター辞書定義が含まれています"
        );
        assert!(
            !contains_global_actor_dictionary(&click, "男の子"),
            "click.pasta にグローバルアクター辞書定義が含まれています"
        );
    }
}
