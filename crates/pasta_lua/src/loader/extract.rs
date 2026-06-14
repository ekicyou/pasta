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
//!
//! 操作対象は自己展開先＋同一領域の一時／退避シブリングのみで、`scripts/` や他のゴースト
//! ファイルには一切触れない（Req 2.5）。展開物は解凍済みの生テキストとして配置する（Req 2.6）。

use std::io::{Cursor, Read};
use std::path::Path;
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
///
/// 使用された版のダイジェストは常にコンパイル時定数 `EXPECTED_MD5` であり、
/// 各分岐内のログ（DEBUG/INFO）に記録される（バリアントでは保持しない）。
pub(crate) enum SyncOutcome {
    /// マーカー一致（高速パス）。書き込みは発生していない。
    Skipped,
    /// 再展開を実施した。
    Deployed,
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
        return Ok(SyncOutcome::Skipped);
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
    if had_live && let Err(e) = std::fs::rename(target_dir, &backup_dir) {
        // 退避に失敗: live は無傷のまま。一時を掃除して返す（Req 2.3）。
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(LoaderError::self_deploy(target_dir, e));
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

    Ok(SyncOutcome::Deployed)
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
#[path = "extract_tests.rs"]
mod tests;
