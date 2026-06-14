use super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// 指定ディレクトリ配下の全ファイルの (相対パス, 内容バイト, mtime) スナップショットを取る。
/// 一致シナリオで「自己展開先へ一切書き込みが発生しない」ことの証跡に使う。
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    let mut map = BTreeMap::new();
    collect_snapshot(dir, dir, &mut map);
    map
}

fn collect_snapshot(
    root: &Path,
    current: &Path,
    out: &mut BTreeMap<PathBuf, (Vec<u8>, SystemTime)>,
) {
    let Ok(read) = std::fs::read_dir(current) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).expect("stat");
        if meta.is_dir() {
            collect_snapshot(root, &path, out);
        } else {
            let rel = path.strip_prefix(root).unwrap().to_path_buf();
            let bytes = std::fs::read(&path).expect("read");
            let mtime = meta.modified().expect("mtime");
            out.insert(rel, (bytes, mtime));
        }
    }
}

/// FAST PATH: マーカーが `EXPECTED_MD5` と一致 → `Skipped` を返し、
/// 自己展開先へ一切書き込みが発生しないこと（Req 1.2 / 1.5 / 1.6）。
#[test]
fn match_skips_without_any_write() {
    let base = tempfile::tempdir().expect("tempdir");
    let target = base.path().join(SELF_DEPLOY_REL);
    std::fs::create_dir_all(&target).expect("mkdir target");

    // 既存スクリプトを模した別ファイルも置き、これも不変であることを確認する。
    std::fs::write(target.join("existing.lua"), b"-- pre-existing\n").expect("write file");
    std::fs::write(target.join(MARKER_NAME), EXPECTED_MD5).expect("write marker");

    // 呼び出し前のスナップショット（内容＋mtime）。
    let before = snapshot(&target);
    assert!(
        before.contains_key(Path::new(MARKER_NAME)),
        "precondition: marker exists"
    );

    let outcome = sync_pasta_scripts(base.path()).expect("sync ok");

    match outcome {
        SyncOutcome::Skipped => {}
        SyncOutcome::Deployed => panic!("expected Skipped on marker match"),
    }

    // 書き込みが一切発生していないこと: ファイル集合・内容・mtime が完全一致。
    let after = snapshot(&target);
    assert_eq!(
        before, after,
        "no writes must occur on fast path (files/contents/mtimes unchanged)"
    );
}

/// FAST PATH の比較は trim 後の文字列一致: マーカーに前後空白・改行が
/// 付いていても一致と判定され、無書き込みで `Skipped` を返すこと（Req 1.5）。
#[test]
fn marker_with_surrounding_whitespace_still_skips() {
    let base = tempfile::tempdir().expect("tempdir");
    let target = base.path().join(SELF_DEPLOY_REL);
    std::fs::create_dir_all(&target).expect("mkdir target");
    std::fs::write(target.join("existing.lua"), b"-- pre-existing\n").expect("write file");
    std::fs::write(target.join(MARKER_NAME), format!("  {EXPECTED_MD5}\r\n"))
        .expect("write padded marker");

    let before = snapshot(&target);

    let outcome = sync_pasta_scripts(base.path()).expect("sync ok");
    match outcome {
        SyncOutcome::Skipped => {}
        SyncOutcome::Deployed => {
            panic!("expected Skipped: padded marker must match after trim")
        }
    }

    // 一致パスなので一切書き込みが発生しないこと（マーカーの正規化書き戻しも無し）。
    let after = snapshot(&target);
    assert_eq!(before, after, "no writes on trimmed-match fast path");
}

/// マーカーが一致しない（trim 後に異なる）→ 再展開（`Deployed`）。
/// 完全な展開検証は Task 5.1。ここでは判定分岐のみ軽く確認する。
#[test]
fn mismatch_marker_redeploys() {
    let base = tempfile::tempdir().expect("tempdir");
    let target = base.path().join(SELF_DEPLOY_REL);
    std::fs::create_dir_all(&target).expect("mkdir target");
    std::fs::write(target.join(MARKER_NAME), "deadbeef-not-the-expected-digest")
        .expect("write marker");

    let outcome = sync_pasta_scripts(base.path()).expect("sync ok");
    match outcome {
        SyncOutcome::Deployed => {
            // 展開後はマーカーが基準ダイジェストへ更新される。
            let marker =
                std::fs::read_to_string(target.join(MARKER_NAME)).expect("read marker");
            assert_eq!(marker.trim(), EXPECTED_MD5);
        }
        SyncOutcome::Skipped => panic!("expected Deployed on marker mismatch"),
    }
}

/// マーカー欠落（自己展開先が未生成を含む）→ 再展開（`Deployed`）（Req 1.3 / 5.5）。
#[test]
fn missing_marker_deploys() {
    let base = tempfile::tempdir().expect("tempdir");
    // 自己展開先を一切作らない（フレッシュ初回起動相当）。

    let outcome = sync_pasta_scripts(base.path()).expect("sync ok");
    match outcome {
        SyncOutcome::Deployed => {}
        SyncOutcome::Skipped => panic!("expected Deployed when marker missing"),
    }

    // 展開先とマーカーが生成されていること。
    let target = base.path().join(SELF_DEPLOY_REL);
    let marker = std::fs::read_to_string(target.join(MARKER_NAME)).expect("read marker");
    assert_eq!(marker.trim(), EXPECTED_MD5);
}

/// 展開内容が内蔵正本と一致し、旧版の orphan が残らないこと（Req 2.1 / 2.2）。
/// stale `.md5` と orphan ファイルを事前投入 → sync → orphan 消滅・既知エントリ存在・
/// `.md5`＝EXPECTED_MD5 を確認する。
#[test]
fn deploy_replaces_content_without_orphan() {
    let base = tempfile::tempdir().expect("tempdir");
    let target = base.path().join(SELF_DEPLOY_REL);
    std::fs::create_dir_all(&target).expect("mkdir target");

    // stale マーカー（不一致）と orphan ファイルを投入。
    std::fs::write(target.join(MARKER_NAME), "stale-digest-not-expected")
        .expect("write stale marker");
    std::fs::write(
        target.join("zzz_orphan.lua"),
        b"-- orphan from old version\n",
    )
    .expect("write orphan");
    // ネストした orphan も投入（サブツリーごと消えること）。
    std::fs::create_dir_all(target.join("ghost_subdir")).expect("mkdir orphan subdir");
    std::fs::write(target.join("ghost_subdir/old.lua"), b"-- nested orphan\n")
        .expect("write nested orphan");

    let outcome = sync_pasta_scripts(base.path()).expect("sync ok");
    match outcome {
        SyncOutcome::Deployed => {}
        SyncOutcome::Skipped => panic!("expected Deployed on stale marker"),
    }

    // orphan が消滅していること（内容＝zip エントリ集合）。
    assert!(
        !target.join("zzz_orphan.lua").exists(),
        "orphan file must be removed after deploy (no orphans)"
    );
    assert!(
        !target.join("ghost_subdir").exists(),
        "orphan subtree must be removed after deploy"
    );

    // 既知の内蔵エントリ（zip ルートの main.lua）が存在すること。
    assert!(
        target.join("main.lua").exists(),
        "known embedded entry main.lua must exist after deploy"
    );

    // `.md5` が基準ダイジェストへ更新されていること。
    let marker = std::fs::read_to_string(target.join(MARKER_NAME)).expect("read marker");
    assert_eq!(marker.trim(), EXPECTED_MD5);
}

/// `.md5` がスワップ成功後に最後に書かれ、基準ダイジェストと一致すること（Req 2.4）。
/// （並行して、自己展開先以外に temp/backup シブリングの残骸が残らないことも確認する。）
#[test]
fn marker_written_last_equals_digest() {
    let base = tempfile::tempdir().expect("tempdir");

    let outcome = sync_pasta_scripts(base.path()).expect("sync ok");
    assert!(matches!(outcome, SyncOutcome::Deployed));

    let target = base.path().join(SELF_DEPLOY_REL);
    let marker = std::fs::read_to_string(target.join(MARKER_NAME)).expect("read marker");
    assert_eq!(marker.trim(), EXPECTED_MD5);

    // temp/backup シブリングの残骸が profile/pasta/ 配下に残っていないこと。
    let pasta_dir = base.path().join("profile/pasta");
    for entry in std::fs::read_dir(&pasta_dir)
        .expect("read profile/pasta")
        .flatten()
    {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        assert!(
            !name.starts_with(".pasta_scripts.new.")
                && !name.starts_with(".pasta_scripts.old."),
            "no temp/backup leftover siblings; found: {name}"
        );
    }
}

/// `scripts/`（ユーザーカスタム層）と他のゴーストファイルが同期で不変であること（Req 2.5）。
#[test]
fn scripts_dir_untouched() {
    let base = tempfile::tempdir().expect("tempdir");

    // base_dir 直下に scripts/ と無関係ファイルを用意。
    let scripts = base.path().join("scripts");
    std::fs::create_dir_all(&scripts).expect("mkdir scripts");
    std::fs::write(scripts.join("main.lua"), b"-- user custom override\n").expect("write");
    std::fs::create_dir_all(scripts.join("sub")).expect("mkdir scripts/sub");
    std::fs::write(scripts.join("sub/util.lua"), b"-- nested user file\n").expect("write");
    let other = base.path().join("descript.txt");
    std::fs::write(&other, b"charset,UTF-8\n").expect("write other");

    let before = snapshot(&scripts);
    let before_other = std::fs::read(&other).expect("read other");

    let outcome = sync_pasta_scripts(base.path()).expect("sync ok");
    assert!(matches!(outcome, SyncOutcome::Deployed));

    // scripts/ がファイル集合・内容・mtime すべて不変であること。
    let after = snapshot(&scripts);
    assert_eq!(before, after, "scripts/ must be untouched by self-deploy");

    // 無関係ファイルも不変であること。
    assert_eq!(
        before_other,
        std::fs::read(&other).expect("read other after"),
        "unrelated ghost files must be untouched"
    );
}

/// 原子性: 展開／入れ替えが失敗したとき、自己展開先の直前状態を保全すること（Req 2.3）。
///
/// Windows での確実な失敗注入: live 自己展開先内のファイルへ排他（no-share）ハンドルを
/// 開いたまま `sync_pasta_scripts` を呼ぶと、production の swap 手順
/// `rename(live → backup)` が共有違反で失敗する。これにより deploy の失敗→保全パスを
/// 実際に通す。検証内容:
/// - 戻り値が `Err(LoaderError::SelfDeploy{..})` であること。
/// - live のセンチネルファイル群が元の内容のまま存在すること（直前状態の保全）。
/// - stale `.md5` が EXPECTED_MD5 へ書き換わっていない（＝マーカーは最後にしか書かれない）。
/// - `.pasta_scripts.new.` / `.pasta_scripts.old.` の残骸が残っていないこと（エラーパス掃除）。
/// - 同一 base_dir 下の `scripts/`（ユーザー層）が不変であること（Req 2.5 と整合）。
#[cfg(windows)]
#[test]
fn deploy_failure_preserves_prior_live_state() {
    use std::os::windows::fs::OpenOptionsExt;

    let base = tempfile::tempdir().expect("tempdir");
    let target = base.path().join(SELF_DEPLOY_REL);
    std::fs::create_dir_all(&target).expect("mkdir target");

    // 直前の「生きている」自己展開先を模す: センチネル＋stale マーカー（不一致→deploy 起動）。
    let sentinel_a = target.join("locked.lua");
    let sentinel_b = target.join("keep.lua");
    let nested = target.join("sub");
    std::fs::create_dir_all(&nested).expect("mkdir nested");
    let sentinel_c = nested.join("nested.lua");
    std::fs::write(&sentinel_a, b"-- prior live file (locked)\n").expect("write sentinel a");
    std::fs::write(&sentinel_b, b"-- prior live file (keep)\n").expect("write sentinel b");
    std::fs::write(&sentinel_c, b"-- prior live nested file\n").expect("write sentinel c");
    const STALE_MARKER: &str = "stale-digest-prior-live-state";
    std::fs::write(target.join(MARKER_NAME), STALE_MARKER).expect("write stale marker");

    // scripts/（ユーザー層）も置き、不変であることを確認する。
    let scripts = base.path().join("scripts");
    std::fs::create_dir_all(&scripts).expect("mkdir scripts");
    std::fs::write(scripts.join("main.lua"), b"-- user override\n").expect("write user");
    let scripts_before = snapshot(&scripts);

    // live 内のファイルへ排他ハンドル（share_mode 0 = 共有なし）を開く。
    // これを保持したまま sync を呼ぶと rename(live → backup) が共有違反で失敗する。
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0) // FILE_SHARE 無し: 他からの rename/削除を弾く
        .open(&sentinel_a)
        .expect("open exclusive handle inside live dir");

    let result = sync_pasta_scripts(base.path());

    // 失敗パスを実際に通っていること（Ok だったら保全検証は無意味＝テスト失敗）。
    match result {
        Err(LoaderError::SelfDeploy { .. }) => {}
        Err(other) => panic!("expected SelfDeploy error, got {other:?}"),
        Ok(outcome) => {
            let kind = match outcome {
                SyncOutcome::Skipped => "Skipped",
                SyncOutcome::Deployed => "Deployed",
            };
            panic!("expected Err(SelfDeploy) when swap is blocked, got Ok({kind})");
        }
    }

    // --- ハンドル保持中の検証（lock がまだ live を弾いている状態） ---

    // 直前状態の保全: ロック対象センチネルが依然存在すること（内容読取は drop 後に行う。
    // share_mode 0 のため、保持中は自分自身も再 open できないため）。
    assert!(
        sentinel_a.exists(),
        "locked sentinel must still exist after failed deploy (live preserved)"
    );

    // ロックしていない直前ファイルは内容まで不変であること（live が退避されていない証跡）。
    assert_eq!(
        std::fs::read(&sentinel_b).expect("read sentinel b"),
        b"-- prior live file (keep)\n",
        "sentinel b must survive failed deploy unchanged"
    );
    assert_eq!(
        std::fs::read(&sentinel_c).expect("read nested sentinel"),
        b"-- prior live nested file\n",
        "nested sentinel must survive failed deploy unchanged"
    );

    // マーカーは最後にしか書かれない: 失敗時は stale のまま（EXPECTED_MD5 へ書き換わらない）。
    let marker = std::fs::read_to_string(target.join(MARKER_NAME)).expect("read marker");
    assert_eq!(
        marker.trim(),
        STALE_MARKER,
        "stale .md5 must remain unchanged on failed deploy (marker written last)"
    );
    assert_ne!(
        marker.trim(),
        EXPECTED_MD5,
        "marker must NOT advance to expected digest when deploy failed"
    );

    // エラーパスの掃除: temp/backup シブリングの残骸が profile/pasta/ に残らないこと。
    let pasta_dir = base.path().join("profile/pasta");
    for entry in std::fs::read_dir(&pasta_dir)
        .expect("read profile/pasta")
        .flatten()
    {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        assert!(
            !name.starts_with(".pasta_scripts.new.")
                && !name.starts_with(".pasta_scripts.old."),
            "no temp/backup leftover after failed deploy; found: {name}"
        );
    }

    // scripts/（ユーザー層）は不変であること（Req 2.5）。
    let scripts_after = snapshot(&scripts);
    assert_eq!(
        scripts_before, scripts_after,
        "scripts/ must be untouched even on failed deploy"
    );

    // ハンドルは sync 呼び出しより長生きさせる（共有違反を成立させるため）。
    // ここで解放すると locked.lua を再 open できる。
    drop(lock);

    // drop 後にロック対象センチネルの内容も元のままであることを確認する。
    assert_eq!(
        std::fs::read(&sentinel_a).expect("read sentinel a after drop"),
        b"-- prior live file (locked)\n",
        "locked sentinel must survive failed deploy unchanged"
    );
}
