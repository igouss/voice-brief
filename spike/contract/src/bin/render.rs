use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let [_, idea_md, template] = args.as_slice() else {
        eprintln!("usage: render <idea.md> <system-template.md>");
        return ExitCode::from(2);
    };
    let idea: String = fs::read_to_string(idea_md).expect("read idea.md");
    let Some(table) = vb_contract::format_table(&idea) else {
        eprintln!("no format table in {idea_md}");
        return ExitCode::FAILURE;
    };
    let template: String = fs::read_to_string(template).expect("read template");
    print!("{}", vb_contract::render(&template, table));
    ExitCode::SUCCESS
}
