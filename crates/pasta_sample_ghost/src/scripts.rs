//! pasta DSL スクリプト
//!
//! サンプルゴースト用の pasta DSL スクリプトは
//! `ghosts/hello-pasta/ghost/master/dic/` に実ファイルとして配置されています。

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    /// ghosts/hello-pasta/ghost/master/dic ディレクトリのパスを取得
    fn dic_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ghosts/hello-pasta/ghost/master/dic")
    }

    /// dic/ からスクリプトファイルを読み込む
    fn read_pasta_script(name: &str) -> String {
        let path = dic_dir().join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} の読み込みに失敗: {}", name, e))
    }

    /// dic/ の全 *.pasta を（ファイル名, 内容）で辞書順に返す
    fn all_pasta_scripts() -> Vec<(String, String)> {
        let mut names: Vec<String> = std::fs::read_dir(dic_dir())
            .expect("dic/ の読み込みに失敗")
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .filter(|n| n.ends_with(".pasta"))
            .collect();
        names.sort();
        names
            .into_iter()
            .map(|n| {
                let content = read_pasta_script(&n);
                (n, content)
            })
            .collect()
    }

    /// `line` と完全一致する行の数
    fn count_lines(content: &str, line: &str) -> usize {
        content.lines().filter(|l| l.trim_end() == line).count()
    }

    #[test]
    fn test_face_pasta_contains_all_characters() {
        let face = read_pasta_script("03-face.pasta");
        assert!(face.contains("％女の子"), "女の子アクターがありません");
        assert!(face.contains("％男の子"), "男の子アクターがありません");
        assert!(face.contains("＠笑顔"), "笑顔表情がありません");
        assert!(face.contains("＠通常"), "通常表情がありません");
        assert!(face.contains("＠怒り"), "怒り表情がありません");
    }

    /// 表情名とサーフェス番号の対応（女の子 0〜8・男の子 10〜18）が揃っている
    #[test]
    fn test_face_pasta_expression_surface_mapping() {
        const NAMES: [&str; 9] = [
            "笑顔",
            "通常",
            "照れ",
            "驚き",
            "泣き",
            "困惑",
            "キラキラ",
            "眠い",
            "怒り",
        ];
        let mut expected = Vec::new();
        for (actor, base) in [("女の子", 0), ("男の子", 10)] {
            for (i, name) in NAMES.iter().enumerate() {
                expected.push(format!(r"{}　＠{}：\s[{}]", actor, name, base + i));
            }
        }

        let face = read_pasta_script("03-face.pasta");
        let mut actor = "";
        let mut actual = Vec::new();
        for line in face.lines() {
            if let Some(name) = line.strip_prefix('％') {
                actor = name.trim_end();
            } else if line.starts_with("　＠") {
                actual.push(format!("{}{}", actor, line.trim_end()));
            }
        }
        assert_eq!(
            actual, expected,
            "03-face.pasta の表情とサーフェスの対応が違います"
        );
    }

    #[test]
    fn test_onboot_defined_once_in_boot_pasta() {
        let boot = read_pasta_script("01-boot.pasta");
        assert_eq!(
            count_lines(&boot, "＊OnBoot"),
            1,
            "01-boot.pasta の ＊OnBoot は 1 つ"
        );

        let total: usize = all_pasta_scripts()
            .iter()
            .map(|(_, c)| count_lines(c, "＊OnBoot"))
            .sum();
        assert_eq!(total, 1, "dic/ 全体で ＊OnBoot は 1 つ: {}", total);
    }

    #[test]
    fn test_greeting_pasta_contains_events() {
        let greeting = read_pasta_script("07-greeting.pasta");
        for scene in [
            "＊OnFirstBoot",
            "＊OnClose",
            "＊OnGhostChanged",
            "＊OnGhostChanging",
        ] {
            assert!(
                count_lines(&greeting, scene) >= 1,
                "07-greeting.pasta に {} がありません",
                scene
            );
        }
    }

    #[test]
    fn test_talk_scene_count() {
        let count: usize = all_pasta_scripts()
            .iter()
            .map(|(_, c)| count_lines(c, "＊会話"))
            .sum();
        assert!(count >= 11, "＊会話 は 11 個以上必要: {}", count);
    }

    #[test]
    fn test_hour_pasta_contains_chimes() {
        let hour = read_pasta_script("06-hour.pasta");
        assert!(hour.contains("＊時報12"), "時報12 シーンがありません");
        assert!(
            hour.contains("＊時報その他"),
            "時報その他 シーンがありません"
        );
        assert!(hour.contains("＄時１２"), "＄時１２ 変数参照がありません");
    }

    #[test]
    fn test_touch_pasta_contains_events() {
        let touch = read_pasta_script("08-touch.pasta");
        let count = count_lines(&touch, "＊OnMouseDoubleClick");
        assert!(count >= 3, "ダブルクリック反応は 3 種以上必要: {}", count);
        assert!(
            touch.contains("＞transfer_req_to_var"),
            "＞transfer_req_to_var がありません"
        );
        assert!(touch.contains("＄ｒ４"), "＄ｒ４ 変数参照がありません");
    }

    #[test]
    fn test_script_expression_names_defined_in_face() {
        let face = read_pasta_script("03-face.pasta");

        /// シーン内のアクション行の先頭の `＠表情` を抜き出す
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
                if let Some(rest) = line.strip_prefix('　')
                    && let Some(after_colon) = rest.split_once('：').map(|(_, r)| r)
                    && let Some(name) = after_colon.strip_prefix('＠')
                {
                    let expr_name = name.split('　').next().unwrap_or(name);
                    if !expr_name.is_empty() {
                        names.push(expr_name);
                    }
                }
            }
            names
        }

        let scripts = all_pasta_scripts();
        let mut total = 0;
        for (name, script) in &scripts {
            for expr_name in extract_expression_names(script) {
                total += 1;
                assert!(
                    face.contains(&format!("＠{}：", expr_name)),
                    "{} 内の表情名「＠{}」が 03-face.pasta に定義されていません",
                    name,
                    expr_name
                );
            }
        }
        assert!(total > 0, "dic/ から表情名が 1 つも抽出されませんでした");
    }

    /// 行頭のアクター辞書（`％アクター名`）は 03-face.pasta にだけある
    /// シーン内アクタースコープ（インデント付き`　％actor_name`）は対象外
    #[test]
    fn test_global_actor_dictionary_only_in_face_pasta() {
        for (name, script) in all_pasta_scripts() {
            let has_actor_dic = script.lines().any(|l| l.starts_with('％'));
            assert_eq!(
                has_actor_dic,
                name == "03-face.pasta",
                "行頭のアクター辞書の置き場所が違います: {}",
                name
            );
        }
    }
}
