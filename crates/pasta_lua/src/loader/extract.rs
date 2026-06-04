//! 起動時自己展開（self-deploy）。
//!
//! pasta-scripts-self-deploy (Req 1.x, 2.x, 5.4–5.5): pasta.dll はフレームワーク
//! スクリプト（標準ランタイム Lua 一式）の唯一の正本源であり、起動時にディスク上の
//! 自己展開先（`base_dir` 相対 `profile/pasta/pasta_scripts/`）を内蔵 zip 正本へ整合する。
//!
//! 内蔵正本（`EMBEDDED_ZIP`）と基準ダイジェスト（`EXPECTED_MD5`）は build.rs が生成し、
//! `include_bytes!` / `env!` でコンパイル時定数として参照する。
//!
//! # 判定（このモジュールの中心）
//! - 自己展開先直下の `.md5` マーカーを読み、`EXPECTED_MD5` と**文字列比較のみ**で判定する
//!   （Req 1.5: ディスクファイルの再ハッシュは行わない）。
//! - 一致 → 一切書き込まず `Skipped` を返し、使用中の版を DEBUG ログに記録する
//!   （Req 1.2 / Req 1.6）。
//! - 欠落・読込失敗・不一致 → 内蔵 zip を自己展開先へ展開し `Deployed` を返す
//!   （Req 1.3 / Req 1.4 / Req 5.5）。
//!
//! # 展開（Task 2.3 でハードニング済み）
//! 不一致/欠落時は内蔵 zip を**準アトミック**に自己展開する：
//! 1. 自己展開先と同一ボリューム（`profile/pasta/` 配下のシブリング）の一時ディレクトリへ
//!    全エントリを展開し、成功を確認する（Req 2.1）。
//! 2. 旧自己展開先があれば退避（rename）→ 一時→自己展開先へ rename → 旧を削除、という
//!    Windows 安全順（退避→差し込み→旧削除）でスワップする（Req 2.2 / design Risks）。
//! 3. スワップ成功後にのみ `.md5` マーカーを書き込む（Req 2.4）。
//! 4. 一時展開／差し込みが失敗したら自己展開先の直前状態を保全し、残骸を掃除して
//!    `LoaderError::SelfDeploy` を返す（Req 2.3 / フォールバック）。
//! 操作対象は自己展開先＋同一領域の一時／退避シブリングのみで、`scripts/` や他のゴースト
//! ファイルには一切触れない（Req 2.5）。展開物は解凍済みの生テキストとして配置する（Req 2.6）。

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::{debug, info};

use super::LoaderError;

/// 内蔵正本（決定論的 zip）。build.rs が `OUT_DIR/pasta_scripts.zip` を生成する。
const EMBEDDED_ZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/pasta_scripts.zip"));

/// 基準ダイジェスト（内蔵 zip の MD5）。build.rs が `cargo:rustc-env` で公開する。
const EXPECTED_MD5: &str = env!("PASTA_SCRIPTS_MD5");

/// 自己展開先（`base_dir` 相対）。
const SELF_DEPLOY_REL: &str = "profile/pasta/pasta_scripts";

/// マーカーファイル名（自己展開先直下）。
const MARKER_NAME: &str = ".md5";

/// プロセス内でユニークなサフィックスを得るための単調増加カウンタ。
/// `process::id()` + nanos + counter で一時/退避ディレクトリ名の衝突確率を下げる
/// （衝突は自己修復で吸収可能だが、可能な限り回避する）。
static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 起動時自己展開の結果。
pub(crate) enum SyncOutcome {
    /// マーカー一致（高速パス）。書き込みは発生していない。
    Skipped { digest: String },
    /// 再展開を実施した。
    Deployed { digest: String },
}

/// 起動時自己展開。Phase 2.5 で `base_dir` を受けて呼ばれる。
///
/// 自己展開先の `.md5` マーカーと `EXPECTED_MD5` を文字列比較し、一致時は無書き込みで
/// `Skipped` を返す。欠落・不一致・読込失敗時は内蔵 zip を展開し `Deployed` を返す。
///
/// 失敗時は `LoaderError::SelfDeploy` を返すが、呼び出し側（Phase 2.5）で握り潰し
/// 起動を継続する（非致命）。
pub(crate) fn sync_pasta_scripts(base_dir: &Path) -> Result<SyncOutcome, LoaderError> {
    let target_dir = base_dir.join(SELF_DEPLOY_REL);
    let marker_path = target_dir.join(MARKER_NAME);

    // FAST PATH: マーカー文字列のみで比較（Req 1.5: 再ハッシュしない）。
    // 読込失敗・欠落は不一致（再展開）として扱う（Req 1.3）。
    if let Ok(marker) = std::fs::read_to_string(&marker_path)
        && marker.trim() == EXPECTED_MD5
    {
        // 一致: 一切書き込まず即 return（Req 1.2）。使用中の版を DEBUG ログ（Req 1.6）。
        debug!(
            digest = EXPECTED_MD5,
            path = %target_dir.display(),
            "pasta_scripts up-to-date (fast path); using embedded version"
        );
        return Ok(SyncOutcome::Skipped {
            digest: EXPECTED_MD5.to_string(),
        });
    }

    // 不一致/欠落: 再展開（Req 1.3 / Req 1.4 / Req 5.5）。
    deploy(&target_dir, &marker_path)
}

/// 内蔵 zip を自己展開先へ**準アトミック**に展開し、最後に `.md5` マーカーを書き込む。
///
/// 手順（Req 2.1–2.4 / design Risks: Windows rename 制約への対応）:
/// 1. 同一ボリュームの一時ディレクトリ（`profile/pasta/` 配下のシブリング）へ全展開し、
///    成功を確認する（この時点までは live を一切触らない＝Req 2.3 の保全）。
/// 2. live があれば退避ディレクトリへ rename（旧退避）。
/// 3. 一時ディレクトリを live へ rename（新差し込み）。失敗時は退避を戻す。
/// 4. 旧退避を削除（旧削除）。
/// 5. `.md5` マーカーを最後に書き込む（Req 2.4）。
///
/// いずれの失敗でも live は直前状態へ復元し、一時／退避の残骸を掃除して
/// `LoaderError::SelfDeploy` を返す（Req 2.3）。
fn deploy(target_dir: &Path, marker_path: &Path) -> Result<SyncOutcome, LoaderError> {
    // 同一ボリュームに一時／退避シブリングを置くため、親（profile/pasta/）を確保する。
    // 親が決定できない場合は target_dir 自身を基準にする（防御的）。
    let parent = target_dir.parent().unwrap_or(target_dir);
    std::fs::create_dir_all(parent).map_err(|e| LoaderError::self_deploy(parent, e))?;

    let suffix = unique_suffix();
    let temp_dir = parent.join(format!(".pasta_scripts.new.{suffix}"));
    let backup_dir = parent.join(format!(".pasta_scripts.old.{suffix}"));

    // 前回失敗の残骸が万一同名で残っていたら掃除しておく（衝突回避）。
    let _ = std::fs::remove_dir_all(&temp_dir);
    let _ = std::fs::remove_dir_all(&backup_dir);

    // --- 1. 一時ディレクトリへ全展開（live はまだ触らない） ---
    if let Err(e) = extract_zip_into(&temp_dir) {
        // 一時展開失敗: live は無傷。一時残骸のみ掃除して返す（Req 2.3）。
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(e);
    }

    // --- 2. live があれば退避（旧退避） ---
    let had_live = target_dir.exists();
    if had_live {
        if let Err(e) = std::fs::rename(target_dir, &backup_dir) {
            // 退避に失敗: live は無傷のまま。一時を掃除して返す（Req 2.3）。
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(LoaderError::self_deploy(target_dir, e));
        }
    }

    // --- 3. 一時を live へ差し込み（新差し込み） ---
    if let Err(e) = std::fs::rename(&temp_dir, target_dir) {
        // 差し込み失敗: 退避した旧を戻して直前状態を復元する（Req 2.3）。
        if had_live {
            let _ = std::fs::rename(&backup_dir, target_dir);
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(LoaderError::self_deploy(target_dir, e));
    }

    // --- 4. 旧退避を削除（旧削除）。失敗しても致命ではない（残骸掃除のみ） ---
    if had_live {
        let _ = std::fs::remove_dir_all(&backup_dir);
    }

    // --- 5. `.md5` マーカーを最後に書き込む（Req 2.4） ---
    std::fs::write(marker_path, EXPECTED_MD5)
        .map_err(|e| LoaderError::self_deploy(marker_path, e))?;

    // 同期実施＋更新後の版を INFO ログに記録（Req 2.7）。
    info!(
        digest = EXPECTED_MD5,
        path = %target_dir.display(),
        "pasta_scripts self-deployed (extracted embedded version)"
    );

    Ok(SyncOutcome::Deployed {
        digest: EXPECTED_MD5.to_string(),
    })
}

/// 内蔵 zip の全エントリを `dest` へ解凍済み生ファイルとして展開する（Req 2.6）。
///
/// `enclosed_name` で path traversal を防止する。成功時は zip エントリ集合と一致する
/// ファイル集合が `dest` 配下に生成される（orphan なし＝空ディレクトリへの新規展開のため）。
fn extract_zip_into(dest: &Path) -> Result<(), LoaderError> {
    std::fs::create_dir_all(dest).map_err(|e| LoaderError::self_deploy(dest, e))?;

    let mut archive = zip::ZipArchive::new(Cursor::new(EMBEDDED_ZIP))
        .map_err(|e| LoaderError::self_deploy(dest, zip_io_error(e)))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| LoaderError::self_deploy(dest, zip_io_error(e)))?;

        // zip 内の安全な相対パス。enclosed_name で path traversal を防ぐ。
        let rel = match entry.enclosed_name() {
            Some(p) => p,
            None => continue,
        };
        let out_path = dest.join(&rel);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| LoaderError::self_deploy(&out_path, e))?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| LoaderError::self_deploy(parent, e))?;
        }

        // 解凍済み生バイトとして書き出す（再圧縮しない＝Req 2.6）。
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| LoaderError::self_deploy(&out_path, e))?;
        std::fs::write(&out_path, &bytes).map_err(|e| LoaderError::self_deploy(&out_path, e))?;
    }

    Ok(())
}

/// 一時／退避ディレクトリ名のためのユニークなサフィックスを生成する。
/// 外部 crate を使わず、`process::id()` + nanos + プロセス内カウンタで構成する。
fn unique_suffix() -> String {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{pid}.{nanos}.{counter}")
}

/// zip クレートのエラーを `std::io::Error` へ変換する（`LoaderError::self_deploy` 用）。
fn zip_io_error(e: zip::result::ZipError) -> std::io::Error {
    match e {
        zip::result::ZipError::Io(io) => io,
        other => std::io::Error::other(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
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
            SyncOutcome::Skipped { digest } => assert_eq!(digest, EXPECTED_MD5),
            SyncOutcome::Deployed { .. } => panic!("expected Skipped on marker match"),
        }

        // 書き込みが一切発生していないこと: ファイル集合・内容・mtime が完全一致。
        let after = snapshot(&target);
        assert_eq!(
            before, after,
            "no writes must occur on fast path (files/contents/mtimes unchanged)"
        );
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
            SyncOutcome::Deployed { digest } => {
                assert_eq!(digest, EXPECTED_MD5);
                // 展開後はマーカーが基準ダイジェストへ更新される。
                let marker = std::fs::read_to_string(target.join(MARKER_NAME)).expect("read marker");
                assert_eq!(marker.trim(), EXPECTED_MD5);
            }
            SyncOutcome::Skipped { .. } => panic!("expected Deployed on marker mismatch"),
        }
    }

    /// マーカー欠落（自己展開先が未生成を含む）→ 再展開（`Deployed`）（Req 1.3 / 5.5）。
    #[test]
    fn missing_marker_deploys() {
        let base = tempfile::tempdir().expect("tempdir");
        // 自己展開先を一切作らない（フレッシュ初回起動相当）。

        let outcome = sync_pasta_scripts(base.path()).expect("sync ok");
        match outcome {
            SyncOutcome::Deployed { digest } => assert_eq!(digest, EXPECTED_MD5),
            SyncOutcome::Skipped { .. } => panic!("expected Deployed when marker missing"),
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
        std::fs::write(target.join("zzz_orphan.lua"), b"-- orphan from old version\n")
            .expect("write orphan");
        // ネストした orphan も投入（サブツリーごと消えること）。
        std::fs::create_dir_all(target.join("ghost_subdir")).expect("mkdir orphan subdir");
        std::fs::write(target.join("ghost_subdir/old.lua"), b"-- nested orphan\n")
            .expect("write nested orphan");

        let outcome = sync_pasta_scripts(base.path()).expect("sync ok");
        match outcome {
            SyncOutcome::Deployed { digest } => assert_eq!(digest, EXPECTED_MD5),
            SyncOutcome::Skipped { .. } => panic!("expected Deployed on stale marker"),
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
        assert!(matches!(outcome, SyncOutcome::Deployed { .. }));

        let target = base.path().join(SELF_DEPLOY_REL);
        let marker = std::fs::read_to_string(target.join(MARKER_NAME)).expect("read marker");
        assert_eq!(marker.trim(), EXPECTED_MD5);

        // temp/backup シブリングの残骸が profile/pasta/ 配下に残っていないこと。
        let pasta_dir = base.path().join("profile/pasta");
        for entry in std::fs::read_dir(&pasta_dir).expect("read profile/pasta").flatten() {
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
        assert!(matches!(outcome, SyncOutcome::Deployed { .. }));

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
                    SyncOutcome::Skipped { .. } => "Skipped",
                    SyncOutcome::Deployed { .. } => "Deployed",
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
        for entry in std::fs::read_dir(&pasta_dir).expect("read profile/pasta").flatten() {
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
}
