use serde_json::Value;

/// Make all paths relative to `/ssg/src`, regardless of where `cargo run` is called from.
pub fn absolute_path(path: &str) -> String {
    // current_exe is in /target/debug when invoked with cargo run
    let current_exe = std::env::current_exe().unwrap();

    // from there, resolve relative path to /ssg/src
    let absolute_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src")
        .join(path);

    absolute_path.to_str().unwrap().to_string()
}

pub fn read_file(path: &str) -> String {
    let abs_path = absolute_path(path);
    if let Ok(file) = std::fs::File::open(&abs_path) {
        std::io::read_to_string(file).unwrap()
    } else {
        panic!("File not found: {}", &abs_path);
    }
}

/// Replaces all instances of `{{key}}` the template string with the JSON value from `data[key]`.
pub fn replace_placeholder_values(data: &Value, template: &str) -> String {
    data.as_object()
        .unwrap()
        .into_iter()
        .fold(template.to_string(), |acc, (key, value)| {
            acc.replace(
                &format!("{{{{{key}}}}}"), // 🤮
                &match value {
                    Value::String(file_type_string) => file_type_string.to_owned(),
                    Value::Number(file_type_number) => file_type_number.to_string(),
                    _ => panic!(),
                },
            )
        })
}

// Pretty logging
use ansi_term::{
    Color::{Cyan, Green, Red, Yellow, RGB},
    Style,
};

pub fn log_heading(heading: &str) {
    let style = Style::new().on(Cyan);
    println!("\n{}", style.paint(format!(" {} ", heading)));
}

pub fn log_error(label: &str, msg: &str) {
    eprint!("{}", "❌");
    if !label.is_empty() {
        eprint!(" {}", RGB(128, 128, 128).paint(format!("[{}]", label)));
    }
    eprint!(" {}\n", msg);
}

pub fn log_warn(label: &str, msg: &str) {
    print!("{}", "⚠️ ");
    if !label.is_empty() {
        print!(" {}", RGB(128, 128, 128).paint(format!("[{}]", label)));
    }
    print!(" {}\n", msg);
}

pub fn log_skip(label: &str, msg: &str) {
    print!("{}", "➖");
    if !label.is_empty() {
        print!(" {}", RGB(128, 128, 128).paint(format!("[{}]", label)));
    }
    print!(" {}\n", msg);
}

pub fn log_success(label: &str, msg: &str) {
    print!("{}", "✅");
    if !label.is_empty() {
        print!(" {}", RGB(128, 128, 128).paint(format!("[{}]", label)));
    }
    print!(" {}\n", msg);
}

pub fn log_info(label: &str, msg: &str) {
    if !label.is_empty() {
        print!(" {}", RGB(128, 128, 128).paint(format!("[{}]", label)));
    }
    print!(" {}\n", RGB(128, 128, 128).paint(msg));
}

pub fn log_red(msg: &str) {
    eprintln!("{}", Red.paint(msg));
}

pub fn log_yellow(msg: &str) {
    println!("{}", Yellow.paint(msg));
}

pub fn log_green(msg: &str) {
    println!("{}", Green.paint(msg));
}

pub fn log_grey(msg: &str) {
    println!("{}", RGB(128, 128, 128).paint(msg));
}
