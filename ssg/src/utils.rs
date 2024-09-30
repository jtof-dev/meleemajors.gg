use serde_json::Value;

pub fn read_file(path: &str) -> String {
    if let Ok(file) = std::fs::File::open(path) {
        std::io::read_to_string(file).unwrap()
    } else {
        panic!("File not found: {}", path);
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
