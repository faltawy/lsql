use std::process::Command;

fn gen_for_grammar(antlr_path: &str, grammar_name: &str) {
    let output_dir = format!("./src/antlr/{}", grammar_name);
    let grammar_path = format!("./src/antlr/{}.g4", grammar_name);

    let output = Command::new("java")
        .arg("-jar")
        .arg(antlr_path)
        .arg("-Dlanguage=Rust")
        .arg("-o")
        .arg(&output_dir)
        .arg(&grammar_path)
        .output()
        .expect("Failed to generate parser");

    if !output.status.success() {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
}


fn main(){
    let antlr_path = "./antlr-4.13.1-complete.jar";
    gen_for_grammar(antlr_path, "./src/antlr/LSQLexer.g4");
}