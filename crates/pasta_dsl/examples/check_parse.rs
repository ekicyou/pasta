use pasta_dsl::parser::parse_file;
use std::path::Path;

fn main() {
    let files = &[
        r"c:\home\maz\git\pasta\.kiro\specs\vscode-trailing-newline\talk.pasta",
        r"c:\home\maz\git\pasta\.kiro\specs\vscode-trailing-newline\ochi.pasta",
    ];
    for f in files {
        let p = Path::new(f);
        match parse_file(p) {
            Ok(ast) => println!("{}: OK ({} items)", f, ast.items.len()),
            Err(e) => println!("{}: ERROR\n{}", f, e),
        }
    }
}
