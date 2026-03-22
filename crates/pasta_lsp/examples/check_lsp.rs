use pasta_lsp::analysis::AnalysisEngine;

fn main() {
    let files = &[
        (r"c:\home\maz\git\pasta\.kiro\specs\vscode-trailing-newline\talk.pasta", "talk.pasta"),
        (r"c:\home\maz\git\pasta\.kiro\specs\vscode-trailing-newline\ochi.pasta", "ochi.pasta"),
    ];
    for (path, label) in files {
        let content = std::fs::read_to_string(path).unwrap();
        let result = AnalysisEngine::analyze(&content);
        println!("=== {} ===", label);
        println!("  Tokens: {}", result.tokens.len());
        println!("  Diagnostics: {}", result.diagnostics.len());
        for (i, d) in result.diagnostics.iter().enumerate() {
            if i < 5 {
                println!("    [{i}] L{}:{} - {}", d.range.start.line, d.range.start.character, &d.message[..d.message.len().min(100)]);
            }
        }
        if result.diagnostics.len() > 5 {
            println!("    ... and {} more", result.diagnostics.len() - 5);
        }
    }
}
