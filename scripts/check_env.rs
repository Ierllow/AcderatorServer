use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

const ENV_FILE: &str = ".env";

fn main() {
    println!("cargo:rerun-if-changed={ENV_FILE}");
    println!("cargo:rerun-if-env-changed=CI");

    if env::var("CI").as_deref() == Ok("true") {
        return;
    }

    let env_path = Path::new(ENV_FILE);
    let env = fs::read_to_string(env_path).unwrap_or_else(|err| {
        panic!("failed to read {ENV_FILE}: {err}");
    });

    let values = parse_env(&env);

    if values.is_empty() {
        panic!("{ENV_FILE} must contain at least one env key");
    }

    for (key, value) in values {
        let value = Some(value.as_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| panic!("{key} must be set in {ENV_FILE}"));

        println!("cargo:rustc-env={key}={value}");
    }
}

fn parse_env(input: &str) -> HashMap<String, String> {
    input
        .lines()
        .filter_map(|line| {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            let (key, value) = line
                .split_once('=')
                .unwrap_or_else(|| panic!("invalid .env line: {line}"));

            Some((key.trim().to_owned(), trim_quotes(value.trim()).to_owned()))
        })
        .collect()
}

fn trim_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}
