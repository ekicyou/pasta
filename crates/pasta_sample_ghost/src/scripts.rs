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

    /// 8 段目はつつき・撫でをショートカットのシーンで書く。つつきは 4 種の段
    /// （アクター＋部位・アクターその他・部位だけ・その他）を 1 つ以上ずつ持ち、撫でのシーンがあり、
    /// 旧い書き方（`＊OnMouseDoubleClick`・`＞transfer_req_to_var`・`＄ｒ４`）を使わない。
    #[test]
    fn test_touch_pasta_contains_events() {
        const ACTORS: [&str; 2] = ["女の子", "男の子"];
        let touch = read_pasta_script("08-touch.pasta");
        // コメント行（＃）の説明ではなく、シーンの本文を調べる
        let body: Vec<&str> = touch.lines().filter(|l| !l.starts_with('＃')).collect();
        let pokes: Vec<&str> = body
            .iter()
            .filter_map(|l| l.trim_end().strip_prefix("＊つっつき反応"))
            .collect();
        // 段の種類: 0=アクター＋部位, 1=アクターその他, 2=部位だけ, 3=その他
        let mut tiers = [0usize; 4];
        for suffix in &pokes {
            let tier = match ACTORS.iter().find_map(|a| suffix.strip_prefix(a)) {
                Some("その他") => 1,
                Some(part) if !part.is_empty() => 0,
                Some(_) => panic!("部位の無い名前 `＊つっつき反応{suffix}`"),
                None if *suffix == "その他" => 3,
                None => 2,
            };
            tiers[tier] += 1;
        }
        assert!(
            tiers.iter().all(|&n| n >= 1),
            "つつきの 4 種の段が 1 つ以上ずつ必要（アクター＋部位・アクターその他・部位だけ・その他）: {tiers:?} / {pokes:?}"
        );
        assert!(
            body.iter().any(|l| l.starts_with("＊さわり反応")),
            "撫でのシーン（＊さわり反応…）がありません"
        );
        for old in ["＊OnMouseDoubleClick", "＞transfer_req_to_var", "＄ｒ４"] {
            assert!(
                body.iter().all(|l| !l.contains(old)),
                "8 段目に旧い書き方 {old} が残っています"
            );
        }
        let choice = read_pasta_script("09-choice.pasta");
        assert_eq!(
            count_lines(&choice, "＊つっつき反応その他"),
            1,
            "9 段目に ＊つっつき反応その他 がちょうど 1 つ必要"
        );
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
