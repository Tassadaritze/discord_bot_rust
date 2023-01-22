use proc_macro::TokenStream;
use std::io::ErrorKind;
use std::path::Path;
use std::{fs, io};

// fn register_commands(commands: &mut serenity::builder::CreateApplicationCommands) -> &mut serenity::builder::CreateApplicationCommands {
// commands.create_application_command(|command| commands::foo::register(command));
// commands.create_application_command(|command| commands::bar::register(command));
// ...etc
// commands
// }
#[proc_macro]
pub fn register_commands(_: TokenStream) -> TokenStream {
    let mut out = String::from("fn register_commands(commands: &mut serenity::builder::CreateApplicationCommands) -> &mut serenity::builder::CreateApplicationCommands {\n");
    for command in read_modules("src/commands").expect("error reading command directory") {
        out += "commands.create_application_command(|command| crate::commands::";
        out += &command;
        out += "::register(command));\n";
    }
    out += "commands\n}";

    out.parse().unwrap()
}

// match command.data.name.as_str() {
// "foo" => commands::foo::run(&command.data.options),
// "bar" => commands::bar::run(&command.data.options),
// ...etc
// _ => "not implemented".to_string(),
// }
#[proc_macro]
pub fn run_commands(_: TokenStream) -> TokenStream {
    let mut out = String::from("match command.data.name.as_str() {");
    for command in read_modules("src/commands").expect("error reading command directory") {
        out += "\"";
        out += &command;
        out += "\" => crate::commands::";
        out += &(command + "::run(&command.data.options),\n");
    }
    out += "_ => \"not implemented\".to_string(),\n";
    out += "}";

    out.parse().unwrap()
}

// pub mod foo;
// pub mod bar;
// ...etc
#[proc_macro]
pub fn get_modules(input: TokenStream) -> TokenStream {
    let path = input.to_string();
    let path = Path::new(path.trim_matches('"'));

    let mut out = String::new();
    if path.is_dir() {
        for module in read_modules(path.to_str().unwrap()).expect("error reading command directory")
        {
            out += "pub mod ";
            out += &(module + ";\n");
        }
    } else {
        panic!("{:?} is not a valid directory", path);
    }

    out.parse().unwrap()
}

fn read_modules(dir: &str) -> io::Result<Vec<String>> {
    let mut modules = Vec::new();

    for entry in fs::read_dir(dir)? {
        let file_name = entry?
            .file_name()
            .to_str()
            .ok_or(ErrorKind::InvalidData)?
            .trim_end_matches(".rs")
            .to_string();
        if file_name == "mod" {
            continue;
        }
        modules.push(file_name);
    }

    Ok(modules)
}
