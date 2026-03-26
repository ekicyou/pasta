mod copy;
mod nar;
mod release;
mod update_files;

use std::path::PathBuf;
use std::process;

/// コマンドライン引数
struct Args {
    command: Command,
}

/// サブコマンド
enum Command {
    Release(ReleaseArgs),
    Help,
    Version,
}

/// release サブコマンドの引数
pub struct ReleaseArgs {
    /// ゴースト開発フォルダーパス (--target)
    pub target: PathBuf,
    /// リリース出力先フォルダーパス (--release)
    pub release: PathBuf,
    /// NAR 出力ファイルパス (--nar)
    pub nar: PathBuf,
    /// 上書きコピー元フォルダーパス (--copy, 0回以上)
    pub copy_dirs: Vec<PathBuf>,
}

fn main() {
    match parse_args_from(lexopt::Parser::from_env()) {
        Ok(args) => match args.command {
            Command::Help => {
                print_usage();
            }
            Command::Version => {
                println!("pasta_check {}", env!("CARGO_PKG_VERSION"));
            }
            Command::Release(release_args) => {
                if let Err(e) = release::execute_release(&release_args) {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!();
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: pasta_check <command> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  release    Build release package (update files + NAR)");
    eprintln!();
    eprintln!("Options for 'release':");
    eprintln!("  --target <path>   Ghost development folder (required)");
    eprintln!("  --release <path>  Release output folder (required)");
    eprintln!("  --nar <path>      NAR output file path (required)");
    eprintln!("  --copy <path>     Overlay copy folder (repeatable)");
    eprintln!();
    eprintln!("Global options:");
    eprintln!("  --help             Show this help");
    eprintln!("  --version          Show version");
}

fn parse_args_from(mut parser: lexopt::Parser) -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut subcommand: Option<String> = None;
    let mut target: Option<PathBuf> = None;
    let mut release_dir: Option<PathBuf> = None;
    let mut nar: Option<PathBuf> = None;
    let mut copy_dirs: Vec<PathBuf> = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("help") | Short('h') => {
                return Ok(Args {
                    command: Command::Help,
                });
            }
            Long("version") | Short('V') => {
                return Ok(Args {
                    command: Command::Version,
                });
            }
            Long("target") => {
                target = Some(parser.value()?.into());
            }
            Long("release") => {
                release_dir = Some(parser.value()?.into());
            }
            Long("nar") => {
                nar = Some(parser.value()?.into());
            }
            Long("copy") => {
                copy_dirs.push(parser.value()?.into());
            }
            Value(val) if subcommand.is_none() => {
                subcommand = Some(val.to_string_lossy().into_owned());
            }
            _ => return Err(arg.unexpected()),
        }
    }

    match subcommand.as_deref() {
        Some("release") => {
            let target = target.ok_or_else(|| lexopt::Error::MissingValue {
                option: Some("--target".to_string()),
            })?;
            let release_dir = release_dir.ok_or_else(|| lexopt::Error::MissingValue {
                option: Some("--release".to_string()),
            })?;
            let nar = nar.ok_or_else(|| lexopt::Error::MissingValue {
                option: Some("--nar".to_string()),
            })?;
            Ok(Args {
                command: Command::Release(ReleaseArgs {
                    target,
                    release: release_dir,
                    nar,
                    copy_dirs,
                }),
            })
        }
        Some(other) => Err(lexopt::Error::UnexpectedValue {
            option: "command".to_string(),
            value: other.into(),
        }),
        None => Err(lexopt::Error::MissingValue {
            option: Some("command".to_string()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, lexopt::Error> {
        let parser = lexopt::Parser::from_args(args);
        parse_args_from(parser)
    }

    #[test]
    fn test_release_all_required_options() {
        let args = parse(&[
            "release",
            "--target",
            "ghost/dir",
            "--release",
            "release/dir",
            "--nar",
            "out.nar",
        ])
        .unwrap();
        match args.command {
            Command::Release(r) => {
                assert_eq!(r.target, PathBuf::from("ghost/dir"));
                assert_eq!(r.release, PathBuf::from("release/dir"));
                assert_eq!(r.nar, PathBuf::from("out.nar"));
                assert!(r.copy_dirs.is_empty());
            }
            _ => panic!("Expected Release command"),
        }
    }

    #[test]
    fn test_release_missing_target() {
        let result = parse(&["release", "--release", "r", "--nar", "n"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_release_missing_release() {
        let result = parse(&["release", "--target", "t", "--nar", "n"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_release_missing_nar() {
        let result = parse(&["release", "--target", "t", "--release", "r"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_release_multiple_copy() {
        let args = parse(&[
            "release",
            "--target",
            "t",
            "--release",
            "r",
            "--nar",
            "n",
            "--copy",
            "overlay1",
            "--copy",
            "overlay2",
            "--copy",
            "overlay3",
        ])
        .unwrap();
        match args.command {
            Command::Release(r) => {
                assert_eq!(r.copy_dirs.len(), 3);
                assert_eq!(r.copy_dirs[0], PathBuf::from("overlay1"));
                assert_eq!(r.copy_dirs[1], PathBuf::from("overlay2"));
                assert_eq!(r.copy_dirs[2], PathBuf::from("overlay3"));
            }
            _ => panic!("Expected Release command"),
        }
    }

    #[test]
    fn test_no_subcommand() {
        let result = parse(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_flag() {
        let args = parse(&["--help"]).unwrap();
        assert!(matches!(args.command, Command::Help));
    }

    #[test]
    fn test_version_flag() {
        let args = parse(&["--version"]).unwrap();
        assert!(matches!(args.command, Command::Version));
    }

    #[test]
    fn test_unknown_subcommand() {
        let result = parse(&["unknown"]);
        assert!(result.is_err());
    }
}
