//! 検査の核。ファイルの一覧と中身を受け取り、報告を返す（ファイルシステムに触れない）。

mod report;

pub use report::{Finding, FindingKind, Report, Severity, Stats, msg};
