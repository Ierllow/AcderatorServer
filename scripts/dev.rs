use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let root = repository_root()?;
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("setup") => {
            let arguments: Vec<_> = args.collect();
            if arguments
                .iter()
                .any(|argument| matches!(argument.as_str(), "-h" | "--help"))
            {
                print_setup_help();
                Ok(())
            } else {
                setup(&root, SetupOptions::parse(arguments)?)
            }
        }
        Some("cleanup") => cleanup(&root),
        Some("help" | "-h" | "--help") | None => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!(
            "unknown command: {command}\n\nRun with --help for usage."
        )),
    }
}

#[derive(Debug, PartialEq)]
struct SetupOptions {
    db_name: String,
    db_user: String,
    db_pass: String,
    db_root_user: String,
    db_root_pass: Option<String>,
    master_data_path: String,
    skip_packages: bool,
    skip_service: bool,
    no_run: bool,
    debug_ui: bool,
}

impl Default for SetupOptions {
    fn default() -> Self {
        Self {
            db_name: "acderator".into(),
            db_user: "acderator".into(),
            db_pass: "acderatorpass".into(),
            db_root_user: "root".into(),
            db_root_pass: None,
            master_data_path: "static/master/data.json".into(),
            skip_packages: false,
            skip_service: false,
            no_run: false,
            debug_ui: false,
        }
    }
}

impl SetupOptions {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self::default();
        let mut args = args.into_iter();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--db-name" => options.db_name = next_value(&mut args, &argument)?,
                "--db-user" => options.db_user = next_value(&mut args, &argument)?,
                "--db-pass" => options.db_pass = next_value(&mut args, &argument)?,
                "--db-root-user" => options.db_root_user = next_value(&mut args, &argument)?,
                "--db-root-pass" => options.db_root_pass = Some(next_value(&mut args, &argument)?),
                "--master-data-path" => {
                    options.master_data_path = next_value(&mut args, &argument)?
                }
                "--skip-packages" => options.skip_packages = true,
                "--skip-service" => options.skip_service = true,
                "--no-run" => options.no_run = true,
                "--debug-ui" => options.debug_ui = true,
                _ => return Err(format!("unknown setup option: {argument}")),
            }
        }

        validate_identifier("database name", &options.db_name)?;
        validate_identifier("database user", &options.db_user)?;
        validate_identifier("database root user", &options.db_root_user)?;
        if options.db_pass.is_empty() {
            return Err("database password cannot be empty".into());
        }
        if options.master_data_path.contains(['\r', '\n']) {
            return Err("master data path cannot contain a line break".into());
        }
        Ok(options)
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{option} requires a value"))
}

fn validate_identifier(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(format!(
            "{label} must contain only ASCII letters, numbers, and underscores"
        ));
    }
    Ok(())
}

fn setup(root: &Path, options: SetupOptions) -> Result<(), String> {
    println!("Preparing AcderatorServer in {}", root.display());
    if !options.skip_packages {
        install_system_packages()?;
    }
    require_command(
        "mysql",
        "Install a MySQL-compatible client or rerun without --skip-packages.",
    )?;

    if !options.skip_service {
        ensure_database_service(&options)?;
    }
    initialize_database(&options)?;
    write_env_file(root, &options)?;
    apply_migrations(root, &options)?;

    if options.no_run {
        println!("Setup complete. Server start skipped (--no-run).");
        return Ok(());
    }

    println!("Starting AcderatorServer...");
    let mut command = Command::new("cargo");
    command.arg("run").current_dir(root);
    if options.debug_ui {
        command.args(["--features", "debug-ui"]);
    }
    run_status(&mut command, "start the server")
}

fn cleanup(root: &Path) -> Result<(), String> {
    let commands: &[&[&str]] = &[
        &["fix", "--allow-dirty", "--allow-staged", "--all-targets"],
        &[
            "fix",
            "--allow-dirty",
            "--allow-staged",
            "--all-targets",
            "--features",
            "debug-ui",
        ],
        &["fmt"],
    ];
    for arguments in commands {
        let mut command = Command::new("cargo");
        command.args(*arguments).current_dir(root);
        run_status(&mut command, &format!("cargo {}", arguments.join(" ")))?;
    }
    Ok(())
}

fn install_system_packages() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if command_exists("apt-get") {
            run_privileged("apt-get", &["update"], "update the apt package index")?;
            return run_privileged(
                "apt-get",
                &[
                    "install",
                    "-y",
                    "build-essential",
                    "pkg-config",
                    "libssl-dev",
                    "mariadb-server",
                    "mariadb-client",
                ],
                "install system packages",
            );
        }
        if command_exists("dnf") {
            return run_privileged(
                "dnf",
                &[
                    "install",
                    "-y",
                    "gcc",
                    "pkgconf-pkg-config",
                    "openssl-devel",
                    "mariadb-server",
                    "mariadb",
                ],
                "install system packages",
            );
        }
        if command_exists("pacman") {
            return run_privileged(
                "pacman",
                &[
                    "-Sy",
                    "--needed",
                    "--noconfirm",
                    "base-devel",
                    "openssl",
                    "mariadb",
                ],
                "install system packages",
            );
        }
        println!("No supported Linux package manager found; using existing system packages.");
    }

    #[cfg(target_os = "macos")]
    {
        if command_exists("brew") {
            let mut command = Command::new("brew");
            command.args(["install", "mariadb", "pkg-config", "openssl"]);
            return run_status(&mut command, "install Homebrew packages");
        }
        println!("Homebrew was not found; using existing system packages.");
    }

    #[cfg(target_os = "windows")]
    println!("Automatic package installation is not used on Windows.");

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_privileged(program: &str, args: &[&str], context: &str) -> Result<(), String> {
    let is_root = Command::new("id")
        .arg("-u")
        .output()
        .map(|output| output.status.success() && output.stdout == b"0\n")
        .unwrap_or(false);
    let mut command = if is_root {
        Command::new(program)
    } else if command_exists("sudo") {
        let mut command = Command::new("sudo");
        command.arg(program);
        command
    } else {
        return Err(format!("{context} requires root privileges or sudo"));
    };
    command.args(args);
    run_status(&mut command, context)
}

fn ensure_database_service(options: &SetupOptions) -> Result<(), String> {
    if database_is_ready(options) {
        println!("MariaDB is already accepting connections.");
        return Ok(());
    }

    println!("Starting MariaDB...");
    let candidates: &[(&str, &[&str])] = if cfg!(target_os = "windows") {
        &[("sc", &["start", "MariaDB"]), ("sc", &["start", "MySQL80"])]
    } else if cfg!(target_os = "macos") {
        &[("brew", &["services", "start", "mariadb"])]
    } else {
        &[
            ("service", &["mariadb", "start"]),
            ("service", &["mysql", "start"]),
            ("systemctl", &["start", "mariadb"]),
            ("systemctl", &["start", "mysql"]),
        ]
    };

    for (program, arguments) in candidates {
        if !command_exists(program) {
            continue;
        }
        let status = service_command(program, arguments).status();
        if status.is_ok_and(|status| status.success()) && database_is_ready(options) {
            return Ok(());
        }
    }

    Err(
        "MariaDB could not be started. Start it manually or pass --skip-service if it is remote."
            .into(),
    )
}

fn service_command(program: &str, arguments: &[&str]) -> Command {
    #[cfg(target_os = "linux")]
    {
        let is_root = Command::new("id")
            .arg("-u")
            .output()
            .map(|output| output.status.success() && output.stdout == b"0\n")
            .unwrap_or(false);
        if !is_root && command_exists("sudo") {
            let mut command = Command::new("sudo");
            command.arg(program).args(arguments);
            return command;
        }
    }

    let mut command = Command::new(program);
    command.args(arguments);
    command
}

fn database_is_ready(options: &SetupOptions) -> bool {
    if !command_exists("mysqladmin") {
        return false;
    }
    let mut command = Command::new("mysqladmin");
    command
        .args(["--user", &options.db_root_user, "ping", "--silent"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(password) = &options.db_root_pass {
        command.env("MYSQL_PWD", password);
    }
    command.status().is_ok_and(|status| status.success())
}

fn initialize_database(options: &SetupOptions) -> Result<(), String> {
    let password = sql_string(&options.db_pass);
    let sql = format!(
        "CREATE DATABASE IF NOT EXISTS `{}`;\n\
         CREATE USER IF NOT EXISTS '{}'@'localhost' IDENTIFIED BY '{}';\n\
         GRANT ALL PRIVILEGES ON `{}`.* TO '{}'@'localhost';\n\
         FLUSH PRIVILEGES;",
        options.db_name, options.db_user, password, options.db_name, options.db_user
    );
    mysql_input(
        &options.db_root_user,
        options.db_root_pass.as_deref(),
        None,
        sql.as_bytes(),
        "initialize the database",
    )
}

fn apply_migrations(root: &Path, options: &SetupOptions) -> Result<(), String> {
    mysql_input(
        &options.db_user,
        Some(&options.db_pass),
        Some(&options.db_name),
        b"CREATE TABLE IF NOT EXISTS _acderator_migrations (\
          version VARCHAR(255) PRIMARY KEY,\
          applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP\
        );",
        "initialize migration tracking",
    )?;

    let migration_directory = root.join("migrations");
    let mut migrations = fs::read_dir(&migration_directory)
        .map_err(|error| {
            format!(
                "failed to read migration directory {}: {error}",
                migration_directory.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("sql")))
        .collect::<Vec<_>>();
    migrations.sort();

    for path in migrations {
        let version = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| format!("invalid migration path: {}", path.display()))?;
        let check = format!(
            "SELECT 1 FROM _acderator_migrations WHERE version = '{}' LIMIT 1;",
            sql_string(version)
        );
        if mysql_query(options, &check)?.trim() == "1" {
            continue;
        }

        let mut sql = fs::read(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        sql.extend_from_slice(
            format!(
                "\nINSERT INTO _acderator_migrations (version) VALUES ('{}');\n",
                sql_string(version)
            )
            .as_bytes(),
        );
        mysql_input(
            &options.db_user,
            Some(&options.db_pass),
            Some(&options.db_name),
            &sql,
            &format!("apply migration {version}"),
        )?;
        println!("Applied migration {version}");
    }
    Ok(())
}

fn mysql_query(options: &SetupOptions, sql: &str) -> Result<String, String> {
    let output = Command::new("mysql")
        .args([
            "--user",
            &options.db_user,
            "--batch",
            "--skip-column-names",
            &options.db_name,
            "--execute",
            sql,
        ])
        .env("MYSQL_PWD", &options.db_pass)
        .output()
        .map_err(|error| format!("failed to query migration state: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to query migration state: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("migration state was not valid UTF-8: {error}"))
}

fn mysql_input(
    user: &str,
    password: Option<&str>,
    database: Option<&str>,
    input: &[u8],
    context: &str,
) -> Result<(), String> {
    let mut command = Command::new("mysql");
    command.arg("--user").arg(user);
    if let Some(database) = database {
        command.arg(database);
    }
    if let Some(password) = password {
        command.env("MYSQL_PWD", password);
    }
    command.stdin(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to {context}: {error}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| format!("failed to open mysql input while trying to {context}"))?
        .write_all(input)
        .map_err(|error| format!("failed to send SQL while trying to {context}: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("failed to wait for mysql while trying to {context}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "mysql exited with {status} while trying to {context}"
        ))
    }
}

fn write_env_file(root: &Path, options: &SetupOptions) -> Result<(), String> {
    let contents = format!(
        "DATABASE_URL=mysql://{}:{}@127.0.0.1:3306/{}\nMASTER_DATA_PATH={}\n",
        url_component(&options.db_user),
        url_component(&options.db_pass),
        options.db_name,
        options.master_data_path
    );
    let path = root.join(".env");
    fs::write(&path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn sql_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "''")
}

fn url_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn repository_root() -> Result<PathBuf, String> {
    let root =
        env::current_dir().map_err(|error| format!("could not read current directory: {error}"))?;
    if root.join("Cargo.toml").is_file() && root.join("migrations").is_dir() {
        Ok(root)
    } else {
        Err("run this tool from the AcderatorServer repository root".into())
    }
}

fn command_exists(command: impl AsRef<OsStr>) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn require_command(command: &str, hint: &str) -> Result<(), String> {
    if command_exists(command) {
        Ok(())
    } else {
        Err(format!(
            "required command `{command}` was not found. {hint}"
        ))
    }
}

fn run_status(command: &mut Command, context: &str) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("failed to {context}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "command exited with {status} while trying to {context}"
        ))
    }
}

fn print_help() {
    println!(
        "Acderator project scripts\n\n\
         Usage:\n  project-scripts setup [options]\n  project-scripts cleanup\n\n\
         Run `project-scripts setup --help` for setup options."
    );
}

fn print_setup_help() {
    println!(
        "Usage: project-scripts setup [options]\n\n\
         Options:\n\
           --db-name NAME          Database name (default: acderator)\n\
           --db-user USER          Application database user (default: acderator)\n\
           --db-pass PASSWORD      Application database password\n\
           --db-root-user USER     Administrative database user (default: root)\n\
           --db-root-pass PASSWORD Administrative database password\n\
           --master-data-path PATH Master data JSON path\n\
           --skip-packages         Do not install OS packages\n\
           --skip-service          Do not try to start MariaDB\n\
           --no-run                Prepare the project without starting the server\n\
           --debug-ui              Start the server with the debug console enabled\n\
           -h, --help              Show this help"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_setup_options() {
        let options = SetupOptions::parse(
            [
                "--db-name",
                "music_test",
                "--db-pass",
                "p@ss word",
                "--debug-ui",
                "--no-run",
            ]
            .map(str::to_owned),
        )
        .unwrap();

        assert_eq!(options.db_name, "music_test");
        assert_eq!(options.db_pass, "p@ss word");
        assert!(options.debug_ui);
        assert!(options.no_run);
    }

    #[test]
    fn rejects_unsafe_database_identifier() {
        let error =
            SetupOptions::parse(["--db-name", "music; DROP DATABASE music"].map(str::to_owned))
                .unwrap_err();
        assert!(error.contains("database name"));
    }

    #[test]
    fn encodes_database_url_components() {
        assert_eq!(url_component("p@ss word:/"), "p%40ss%20word%3A%2F");
    }

    #[test]
    fn escapes_sql_strings() {
        assert_eq!(sql_string("it's\\fine"), "it''s\\\\fine");
    }
}
