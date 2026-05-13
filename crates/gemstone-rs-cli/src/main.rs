use gemstone_rs::{
    browser::{Browser, ALL_PROTOCOLS},
    codegen::{self, DEFAULT_CONFIG_PATH},
    gci_library_path, gci_library_resolution, profiles, BridgeKeyType, BridgeValue, Config,
    Error as GemStoneError, Oop, Session, Value, DEFAULT_BRIDGE_ROOT,
};
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Usage(message)) => {
            eprintln!("{message}");
            eprintln!();
            eprintln!("{}", usage());
            ExitCode::from(2)
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    let (args, global_env_file) = extract_env_file_option(&args)?;
    apply_env_file_option(global_env_file.as_deref())?;

    match parse_command(&args)? {
        Command::Help => {
            println!("{}", usage());
            Ok(())
        }
        Command::Doctor {
            live,
            strict,
            env_file,
            format,
        } => {
            apply_env_file_option(env_file.as_deref())?;
            run_doctor(live, strict, format)
        }
        Command::EnvSample => {
            print!("{}", env_template_source());
            Ok(())
        }
        Command::EnvWrite { path, force } => {
            write_env_template(&path, force)?;
            println!("wrote {}", path.display());
            Ok(())
        }
        Command::ExamplesList { format } => {
            print_examples_list(format);
            Ok(())
        }
        Command::ExamplesShow { name, format } => {
            let example = find_example(&name).ok_or_else(|| {
                CliError::CodegenCheck(format!(
                    "example {name} not found; run `gemstone-rs examples list`"
                ))
            })?;
            print_example_details(example, format);
            Ok(())
        }
        Command::Eval { source, env_file } => {
            apply_env_file_option(env_file.as_deref())?;
            let mut session = login()?;
            let value = session.eval(&source)?;
            print_value(&mut session, value)?;
            Ok(())
        }
        Command::BrowseDictionaries => {
            let mut session = login()?;
            print_lines(Browser::new(&mut session).dictionaries()?);
            Ok(())
        }
        Command::BrowseClasses { dictionary } => {
            let mut session = login()?;
            print_lines(Browser::new(&mut session).classes(&dictionary)?);
            Ok(())
        }
        Command::BrowseProtocols {
            class_name,
            dictionary,
            meta,
        } => {
            let mut session = login()?;
            println!("{ALL_PROTOCOLS}");
            print_lines(Browser::new(&mut session).protocols(&class_name, meta, &dictionary)?);
            Ok(())
        }
        Command::BrowseMethods {
            class_name,
            protocol,
            dictionary,
            meta,
        } => {
            let mut session = login()?;
            print_lines(Browser::new(&mut session).methods(
                &class_name,
                &protocol,
                meta,
                &dictionary,
            )?);
            Ok(())
        }
        Command::BrowseSource {
            class_name,
            selector,
            dictionary,
            meta,
        } => {
            let mut session = login()?;
            println!(
                "{}",
                Browser::new(&mut session).source(&class_name, &selector, meta, &dictionary)?
            );
            Ok(())
        }
        Command::InspectOop { oop } => {
            let mut session = login()?;
            print_oop_inspection(&mut session, oop)?;
            Ok(())
        }
        Command::BridgeRoot { root } => {
            let mut session = login()?;
            let (name, oop, identity_id) = {
                let bridge_root = session.bridge_root_named(root)?;
                (
                    bridge_root.name().to_string(),
                    bridge_root.oop(),
                    bridge_root.identity_id(),
                )
            };
            println!("root: {name}");
            println!("oop: {}", oop.raw());
            println!("identity_id: {identity_id}");
            println!("identity_map_size: {}", session.identity_map_len());
            Ok(())
        }
        Command::BridgeKeys { root } => {
            let mut session = login()?;
            let keys = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.keys()?
            };
            for key in keys {
                println!(
                    "{}\toop={}\tclass_oop={}\tidentity_id={}",
                    key.print_string,
                    key.oop.raw(),
                    key.class_oop.raw(),
                    key.identity_id
                );
            }
            Ok(())
        }
        Command::BridgeGet {
            root,
            key,
            key_type,
        } => {
            let mut session = login()?;
            let value = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.get_value_with_key_type(&key, key_type)?
            };
            print_value(&mut session, value)?;
            Ok(())
        }
        Command::BridgeInspect {
            root,
            key,
            key_type,
        } => {
            let mut session = login()?;
            let oop = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.get_oop_with_key_type(&key, key_type)?
            };
            print_oop_inspection(&mut session, oop)?;
            Ok(())
        }
        Command::BridgePut {
            root,
            key,
            key_type,
            value,
        } => {
            let mut session = login()?;
            let value = value.to_bridge_value()?;
            let (root_name, oop) = {
                let mut bridge_root = session.bridge_root_named(root)?;
                let oop = bridge_root.put_with_key_type(&key, key_type, value)?;
                bridge_root.commit()?;
                (bridge_root.name().to_string(), oop)
            };
            println!("stored {key}\troot={root_name}\toop={}", oop.raw());
            Ok(())
        }
        Command::BridgeRemove {
            root,
            key,
            key_type,
        } => {
            let mut session = login()?;
            let (root_name, oop) = {
                let mut bridge_root = session.bridge_root_named(root)?;
                let oop = bridge_root.remove_with_key_type(&key, key_type)?;
                bridge_root.commit()?;
                (bridge_root.name().to_string(), oop)
            };
            println!("removed {key}\troot={root_name}\toop={}", oop.raw());
            Ok(())
        }
        Command::BridgeSampleConfig { mapped_name } => {
            print!("{}", codegen::sample_mapping_config(&mapped_name));
            Ok(())
        }
        Command::ProfileSample => {
            print!("{}", profiles::sample_source());
            Ok(())
        }
        Command::ProfileInit { path } => {
            if path.exists() {
                return Err(CliError::CodegenCheck(format!(
                    "{} already exists",
                    path.display()
                )));
            }
            fs::write(&path, profiles::sample_source())?;
            println!("wrote {}", path.display());
            Ok(())
        }
        Command::ProfileValidate { path, format } => {
            let report = profiles::validate_file(&path)?;
            match format {
                OutputFormat::Human => println!(
                    "profile ok: {} ({} profiles: {})",
                    path.display(),
                    report.profile_count,
                    if report.profile_names.is_empty() {
                        "-".to_string()
                    } else {
                        report.profile_names.join(", ")
                    }
                ),
                OutputFormat::Json => println!(
                    r#"{{"ok":true,"path":"{}","profileCount":{},"profileNames":[{}]}}"#,
                    escape_json(&path.display().to_string()),
                    report.profile_count,
                    json_string_array(&report.profile_names)
                ),
            }
            Ok(())
        }
        Command::ProfileList { path, format } => {
            let project = profiles::load_file(&path)?;
            match format {
                OutputFormat::Human => print_profile_list(&path, &project),
                OutputFormat::Json => println!("{}", project_profiles_json(&path, &project)),
            }
            Ok(())
        }
        Command::ProfileShow { path, name, format } => {
            let project = profiles::load_file(&path)?;
            let profile = project.get(&name).ok_or_else(|| {
                CliError::CodegenCheck(format!("profile {name} not found in {}", path.display()))
            })?;
            match format {
                OutputFormat::Human => print_project_profile(&path, profile),
                OutputFormat::Json => println!(
                    r#"{{"path":"{}","profile":{}}}"#,
                    escape_json(&path.display().to_string()),
                    codegen_profile_json(profile)
                ),
            }
            Ok(())
        }
        Command::ProfileResolve { path, name, format } => {
            let project = profiles::load_file(&path)?;
            let profile = project.get(&name).ok_or_else(|| {
                CliError::CodegenCheck(format!("profile {name} not found in {}", path.display()))
            })?;
            let resolved = profile.resolved_config_path()?;
            match format {
                OutputFormat::Human => print_resolved_project_profile(&path, profile, &resolved),
                OutputFormat::Json => println!(
                    r#"{{"path":"{}","profile":{},"resolvedConfig":"{}"}}"#,
                    escape_json(&path.display().to_string()),
                    codegen_profile_json(profile),
                    escape_json(&resolved.display().to_string())
                ),
            }
            Ok(())
        }
        Command::ProfileCheck { path, format } => run_profile_check(&path, format),
        Command::CodegenInit { config } => {
            if config.exists() {
                return Err(CliError::CodegenCheck(format!(
                    "{} already exists",
                    config.display()
                )));
            }
            fs::write(&config, codegen::sample_config())?;
            println!("wrote {}", config.display());
            Ok(())
        }
        Command::CodegenPreview { config } => {
            let config = codegen::Config::from_file(config)?;
            print!("{}", codegen::generate(&config).source);
            Ok(())
        }
        Command::CodegenPreviewProfile { name, profiles } => {
            let config_path = profile_codegen_config_path(&profiles, &name)?;
            let config = codegen::Config::from_file(config_path)?;
            print!("{}", codegen::generate(&config).source);
            Ok(())
        }
        Command::CodegenDiff { config } => {
            let config = codegen::Config::from_file(&config)?;
            let report = codegen::diff(&config)?;
            if report.up_to_date {
                println!("codegen ok: {} is up to date", report.output.display());
                Ok(())
            } else {
                print!("{}", report.diff);
                Err(CliError::CodegenCheck(format!(
                    "{} is stale or missing",
                    report.output.display()
                )))
            }
        }
        Command::CodegenDiffProfile { name, profiles } => {
            let config_path = profile_codegen_config_path(&profiles, &name)?;
            run_codegen_diff(&config_path)
        }
        Command::CodegenCheck { config } => {
            let config_path = config;
            run_codegen_check(&config_path)
        }
        Command::CodegenExplain { config, format } => {
            let config = codegen::Config::from_file(config)?;
            match format {
                OutputFormat::Human => print!("{}", codegen::explain(&config)),
                OutputFormat::Json => println!("{}", codegen::explain_json(&config)),
            }
            Ok(())
        }
        Command::CodegenExplainProfile {
            name,
            profiles,
            format,
        } => {
            let config_path = profile_codegen_config_path(&profiles, &name)?;
            let config = codegen::Config::from_file(config_path)?;
            match format {
                OutputFormat::Human => print!("{}", codegen::explain(&config)),
                OutputFormat::Json => println!("{}", codegen::explain_json(&config)),
            }
            Ok(())
        }
        Command::CodegenCheckProfile { name, profiles } => {
            let config_path = profile_codegen_config_path(&profiles, &name)?;
            run_codegen_check(&config_path)
        }
        Command::CodegenGenerate { config } => {
            let config = codegen::Config::from_file(config)?;
            let generated = codegen::generate_to_file(&config)?;
            println!("generated {}", generated.output.display());
            Ok(())
        }
        Command::CodegenGenerateProfile { name, profiles } => {
            let config_path = profile_codegen_config_path(&profiles, &name)?;
            let config = codegen::Config::from_file(config_path)?;
            let generated = codegen::generate_to_file(&config)?;
            println!("generated {}", generated.output.display());
            Ok(())
        }
        Command::CodegenDiscover { config, classes } => {
            let class_refs: Vec<codegen::ClassRef> = classes
                .iter()
                .map(|class| codegen::ClassRef::parse(class))
                .collect::<Result<_, _>>()
                .map_err(CliError::usage)?;
            let mut session = login()?;
            let discovered =
                codegen::discover(&mut session, default_codegen_output(&config), &class_refs)?;
            codegen::write_config(&config, &discovered)?;
            println!(
                "wrote {} with {} classes",
                config.display(),
                discovered.classes.len()
            );
            Ok(())
        }
        Command::CodegenDiscoverMapping {
            config,
            mapped_name,
            class_name,
        } => {
            let class_ref = codegen::ClassRef::parse(&class_name).map_err(CliError::usage)?;
            let mut session = login()?;
            let discovered = codegen::discover_mapping(
                &mut session,
                default_codegen_output(&config),
                &mapped_name,
                &class_ref,
            )?;
            codegen::write_config(&config, &discovered)?;
            println!(
                "wrote {} with mapping {} from {}",
                config.display(),
                discovered
                    .mapped
                    .first()
                    .map(|mapped| mapped.name.as_str())
                    .unwrap_or("<none>"),
                class_ref.display_name()
            );
            Ok(())
        }
    }
}

fn login() -> Result<Session, CliError> {
    Ok(Session::login(Config::from_env()?)?)
}

fn env_template_source() -> String {
    env_template_source_with(|name| env::var(name).ok())
}

fn write_env_template(path: &Path, force: bool) -> Result<(), CliError> {
    if path.exists() && !force {
        return Err(CliError::CodegenCheck(format!(
            "{} already exists; pass --force to overwrite",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, env_template_source())?;
    Ok(())
}

fn apply_env_file_option(path: Option<&Path>) -> Result<(), CliError> {
    if let Some(path) = path {
        apply_env_file(path)?;
    }
    Ok(())
}

fn extract_env_file_option(args: &[String]) -> Result<(Vec<String>, Option<PathBuf>), CliError> {
    let mut cleaned = Vec::with_capacity(args.len());
    let mut env_file = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--env-file" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::usage("missing path after --env-file"))?;
                env_file = Some(PathBuf::from(value));
            }
            option if option.starts_with("--env-file=") => {
                let (_, value) = option.split_once('=').unwrap_or_default();
                if value.is_empty() {
                    return Err(CliError::usage("missing path after --env-file="));
                }
                env_file = Some(PathBuf::from(value));
            }
            other => cleaned.push(other.to_string()),
        }
        index += 1;
    }
    Ok((cleaned, env_file))
}

fn apply_env_file(path: &Path) -> Result<usize, CliError> {
    let source = fs::read_to_string(path)?;
    let values = parse_env_file_source(&source, path)?;
    let count = values.len();
    for (name, value) in values {
        env::set_var(name, value);
    }
    Ok(count)
}

fn parse_env_file_source(source: &str, path: &Path) -> Result<Vec<(String, String)>, CliError> {
    let mut values = Vec::new();
    for (index, raw_line) in source.lines().enumerate() {
        if let Some((name, value)) = parse_env_file_line(raw_line, path, index + 1)? {
            values.push((name, value));
        }
    }
    Ok(values)
}

fn parse_env_file_line(
    raw_line: &str,
    path: &Path,
    line_no: usize,
) -> Result<Option<(String, String)>, CliError> {
    let mut line = raw_line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    if let Some(rest) = line.strip_prefix("export ") {
        line = rest.trim_start();
    }
    let Some((name, raw_value)) = line.split_once('=') else {
        return Err(env_file_error(path, line_no, "expected NAME=value"));
    };
    let name = name.trim();
    validate_env_file_name(name).map_err(|message| env_file_error(path, line_no, message))?;
    let value = parse_env_file_value(raw_value)
        .map_err(|message| env_file_error(path, line_no, message))?;
    if is_placeholder_env_value(name, &value) {
        return Ok(None);
    }
    Ok(Some((name.to_string(), value)))
}

fn validate_env_file_name(name: &str) -> Result<(), &'static str> {
    if name != "GEMSTONE" && !name.starts_with("GS_") {
        return Err("only GEMSTONE and GS_* variables are allowed in --env-file");
    }
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err("invalid environment variable name");
    }
    Ok(())
}

fn parse_env_file_value(raw_value: &str) -> Result<String, &'static str> {
    let mut value = String::new();
    let mut chars = raw_value.trim_start().chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\'' => loop {
                match chars.next() {
                    Some('\'') => break,
                    Some(ch) => value.push(ch),
                    None => return Err("unterminated single-quoted value"),
                }
            },
            '"' => loop {
                match chars.next() {
                    Some('"') => break,
                    Some('\\') => {
                        if let Some(ch) = chars.next() {
                            value.push(ch);
                        }
                    }
                    Some(ch) => value.push(ch),
                    None => return Err("unterminated double-quoted value"),
                }
            },
            '\\' => {
                if let Some(ch) = chars.next() {
                    value.push(ch);
                }
            }
            '#' if value.chars().last().is_none_or(char::is_whitespace) => break,
            ch if ch.is_whitespace() => {
                while matches!(chars.peek(), Some(next) if next.is_whitespace()) {
                    chars.next();
                }
                if matches!(chars.peek(), Some('#')) {
                    break;
                }
                value.push(ch);
            }
            ch => value.push(ch),
        }
    }
    Ok(value.trim_end().to_string())
}

fn is_placeholder_env_value(name: &str, value: &str) -> bool {
    matches!(
        (name, value),
        ("GS_PASSWORD", "<set-your-password>")
            | ("GS_HOST_PASSWORD", "<set-host-password-if-needed>")
            | ("GS_LIB_PATH", "/full/path/to/libgcirpc.dylib")
    )
}

fn env_file_error(path: &Path, line_no: usize, message: impl Into<String>) -> CliError {
    CliError::EnvFile(format!("{}:{line_no}: {}", path.display(), message.into()))
}

fn env_template_source_with(lookup: impl Fn(&str) -> Option<String>) -> String {
    let stone = env_value(&lookup, "GS_STONE")
        .or_else(|| env_value(&lookup, "GS_STONE_NAME"))
        .unwrap_or_else(|| "gs64stone".to_string());
    let stone_alias = env_value(&lookup, "GS_STONE_NAME").unwrap_or_else(|| stone.clone());

    let mut output = String::new();
    output.push_str("# gemstone-rs GemStone/S environment template\n");
    output.push_str("# Source this file or paste these exports into your shell.\n");
    output.push_str(
        "# Password values are placeholders and are never copied from the current environment.\n\n",
    );

    append_export(
        &mut output,
        "GS_LIB",
        env_value(&lookup, "GS_LIB")
            .as_deref()
            .unwrap_or("/opt/gemstone/product/lib"),
    );
    append_optional_export(
        &mut output,
        "GS_LIB_PATH",
        env_value(&lookup, "GS_LIB_PATH").as_deref(),
        "/full/path/to/libgcirpc.dylib",
    );
    append_export(&mut output, "GS_STONE", &stone);
    append_export(&mut output, "GS_STONE_NAME", &stone_alias);
    append_export(
        &mut output,
        "GS_HOST",
        env_value(&lookup, "GS_HOST")
            .as_deref()
            .unwrap_or("localhost"),
    );
    append_export(
        &mut output,
        "GS_NETLDI",
        env_value(&lookup, "GS_NETLDI")
            .as_deref()
            .unwrap_or("netldi"),
    );
    append_export(
        &mut output,
        "GS_GEM_SERVICE",
        env_value(&lookup, "GS_GEM_SERVICE")
            .as_deref()
            .unwrap_or("gemnetobject"),
    );
    append_export(
        &mut output,
        "GS_USERNAME",
        env_value(&lookup, "GS_USERNAME")
            .as_deref()
            .unwrap_or("DataCurator"),
    );
    append_secret_export(
        &mut output,
        "GS_PASSWORD",
        lookup("GS_PASSWORD").as_deref(),
        "<set-your-password>",
    );
    append_optional_export(
        &mut output,
        "GS_HOST_USERNAME",
        env_value(&lookup, "GS_HOST_USERNAME").as_deref(),
        "host-user-if-needed",
    );
    append_optional_secret_export(
        &mut output,
        "GS_HOST_PASSWORD",
        lookup("GS_HOST_PASSWORD").as_deref(),
        "<set-host-password-if-needed>",
    );
    output.push_str("\n# Verify after editing:\n");
    output.push_str("# gemstone-rs doctor\n");
    output.push_str("# gemstone-rs doctor --live\n");
    output
}

fn env_value(lookup: &impl Fn(&str) -> Option<String>, name: &str) -> Option<String> {
    lookup(name).filter(|value| !value.is_empty())
}

fn append_export(output: &mut String, name: &str, value: &str) {
    output.push_str("export ");
    output.push_str(name);
    output.push('=');
    output.push_str(&shell_quote(value));
    output.push('\n');
}

fn append_optional_export(output: &mut String, name: &str, value: Option<&str>, placeholder: &str) {
    match value.filter(|value| !value.is_empty()) {
        Some(value) => append_export(output, name, value),
        None => {
            output.push_str("# export ");
            output.push_str(name);
            output.push('=');
            output.push_str(&shell_quote(placeholder));
            output.push('\n');
        }
    }
}

fn append_secret_export(output: &mut String, name: &str, current: Option<&str>, placeholder: &str) {
    if current.is_some_and(|value| !value.is_empty()) {
        output.push_str("# ");
        output.push_str(name);
        output.push_str(" is set in this shell; the real value is not printed.\n");
    }
    append_export(output, name, placeholder);
}

fn append_optional_secret_export(
    output: &mut String,
    name: &str,
    current: Option<&str>,
    placeholder: &str,
) {
    if current.is_some_and(|value| !value.is_empty()) {
        output.push_str("# ");
        output.push_str(name);
        output.push_str(" is set in this shell; the real value is not printed.\n");
        append_export(output, name, placeholder);
    } else {
        append_optional_export(output, name, None, placeholder);
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn run_doctor(live: bool, strict: bool, format: OutputFormat) -> Result<(), CliError> {
    if format == OutputFormat::Json {
        return run_doctor_json(live, strict);
    }

    let mut ok = true;
    let mut hints = Vec::new();
    println!("gemstone-rs doctor");
    println!("environment:");
    print_env_value("GS_LIB_PATH", false);
    print_env_value("GS_LIB", false);
    print_env_value("GEMSTONE", false);
    print_env_value("GS_STONE", false);
    print_env_value("GS_STONE_NAME", false);
    print_env_value("GS_HOST", false);
    print_env_value("GS_NETLDI", false);
    print_env_value("GS_GEM_SERVICE", false);
    print_env_value("GS_USERNAME", false);
    print_env_value("GS_PASSWORD", true);
    print_env_value("GS_HOST_USERNAME", false);
    print_env_value("GS_HOST_PASSWORD", true);
    add_missing_credential_hints(&mut hints);
    if strict {
        println!("strict: enabled");
        if env_var_missing_or_empty("GS_LIB_PATH")
            && env_var_missing_or_empty("GS_LIB")
            && env_var_missing_or_empty("GEMSTONE")
        {
            ok = false;
            println!(
                "strict: error: set GS_LIB_PATH, GS_LIB, or GEMSTONE for deterministic GCI loading"
            );
            add_hint(
                &mut hints,
                "Set GS_LIB_PATH to the full libgcirpc path, or set GS_LIB/GEMSTONE to the GemStone product location.",
            );
        }
        if env_var_missing_or_empty("GS_STONE") && env_var_missing_or_empty("GS_STONE_NAME") {
            ok = false;
            println!("strict: error: set GS_STONE or GS_STONE_NAME explicitly");
            add_hint(
                &mut hints,
                "Set GS_STONE or GS_STONE_NAME so CI does not depend on the default stone name.",
            );
        }
    }

    let config = match Config::from_env() {
        Ok(config) => {
            println!("config: ok");
            println!("  stone_nrs: {}", config.stone_nrs());
            Some(config)
        }
        Err(err) => {
            ok = false;
            println!("config: error: {err}");
            add_error_hints(&mut hints, &err);
            None
        }
    };

    if let Some(config) = config {
        match gci_library_resolution(&config) {
            Ok(resolution) => match gci_library_path(&config) {
                Ok(path) => {
                    println!("gci_library: ok: {}", path.display());
                    println!("  source: {}", resolution.source);
                    print_searched_paths(&resolution.searched);
                    print_path_diagnostics("  ", &path);
                    add_gci_architecture_hints(&mut hints, &path);
                }
                Err(err) => {
                    ok = false;
                    println!("gci_library: error: {err}");
                    println!("  source: {}", resolution.source);
                    println!("  path: {}", resolution.path.display());
                    print_searched_paths(&resolution.searched);
                    print_path_diagnostics("  ", &resolution.path);
                    add_error_hints(&mut hints, &err);
                    add_gci_architecture_hints(&mut hints, &resolution.path);
                }
            },
            Err(err) => {
                ok = false;
                println!("gci_library: error: {err}");
                add_error_hints(&mut hints, &err);
                add_gs_lib_directory_hint(&mut hints);
            }
        }

        if live {
            match Session::login(config).and_then(|mut session| session.eval("3 + 4")) {
                Ok(Value::SmallInt(7)) => println!("live: ok: 3 + 4 == 7"),
                Ok(value) => {
                    ok = false;
                    println!("live: error: expected 7, got {value:?}");
                    add_hint(
                        &mut hints,
                        "Confirm the stone can evaluate simple Smalltalk expressions and rerun gemstone-rs doctor --live.",
                    );
                }
                Err(err) => {
                    ok = false;
                    println!("live: error: {err}");
                    add_error_hints(&mut hints, &err);
                }
            }
        } else {
            println!("live: skipped; pass --live to login and evaluate 3 + 4");
        }
    } else if live {
        println!("live: skipped because config is incomplete");
    }

    print_hints(&hints);

    if ok {
        Ok(())
    } else {
        Err(CliError::Doctor(
            "doctor found setup problems; see report above".to_string(),
        ))
    }
}

fn run_doctor_json(live: bool, strict: bool) -> Result<(), CliError> {
    let mut ok = true;
    let mut hints = Vec::new();
    add_missing_credential_hints(&mut hints);
    let mut gci_json = String::from(r#"{"ok":false,"checked":false}"#);
    let mut live_json = if live {
        String::from(r#"{"ok":false,"checked":true,"error":"config incomplete"}"#)
    } else {
        String::from(r#"{"ok":true,"checked":false}"#)
    };
    let mut strict_errors = Vec::new();
    if strict {
        if env_var_missing_or_empty("GS_LIB_PATH")
            && env_var_missing_or_empty("GS_LIB")
            && env_var_missing_or_empty("GEMSTONE")
        {
            ok = false;
            strict_errors.push(
                "set GS_LIB_PATH, GS_LIB, or GEMSTONE for deterministic GCI loading".to_string(),
            );
            add_hint(
                &mut hints,
                "Set GS_LIB_PATH to the full libgcirpc path, or set GS_LIB/GEMSTONE to the GemStone product location.",
            );
        }
        if env_var_missing_or_empty("GS_STONE") && env_var_missing_or_empty("GS_STONE_NAME") {
            ok = false;
            strict_errors.push("set GS_STONE or GS_STONE_NAME explicitly".to_string());
            add_hint(
                &mut hints,
                "Set GS_STONE or GS_STONE_NAME so CI does not depend on the default stone name.",
            );
        }
    }

    let (config_json, config) = match Config::from_env() {
        Ok(config) => {
            let config_json = format!(
                r#"{{"ok":true,"stone":"{}","stoneNrs":"{}","host":"{}","netldi":"{}","username":"{}","gemService":"{}"}}"#,
                escape_json(&config.stone),
                escape_json(&config.stone_nrs()),
                escape_json(&config.host),
                escape_json(&config.netldi),
                escape_json(&config.username),
                escape_json(&config.gem_service)
            );
            (config_json, Some(config))
        }
        Err(err) => {
            ok = false;
            add_error_hints(&mut hints, &err);
            (
                format!(
                    r#"{{"ok":false,"error":"{}"}}"#,
                    escape_json(&err.to_string())
                ),
                None,
            )
        }
    };

    if let Some(config) = config {
        match gci_library_resolution(&config) {
            Ok(resolution) => match gci_library_path(&config) {
                Ok(path) => {
                    add_gci_architecture_hints(&mut hints, &path);
                    gci_json = format!(
                        r#"{{"ok":true,"checked":true,"source":"{}","path":"{}","searched":[{}],"pathDiagnostics":{}}}"#,
                        escape_json(resolution.source),
                        escape_json(&path.display().to_string()),
                        json_path_array(&resolution.searched),
                        path_diagnostics_json(&path)
                    );
                }
                Err(err) => {
                    ok = false;
                    add_error_hints(&mut hints, &err);
                    add_gci_architecture_hints(&mut hints, &resolution.path);
                    gci_json = format!(
                        r#"{{"ok":false,"checked":true,"source":"{}","path":"{}","searched":[{}],"pathDiagnostics":{},"error":"{}"}}"#,
                        escape_json(resolution.source),
                        escape_json(&resolution.path.display().to_string()),
                        json_path_array(&resolution.searched),
                        path_diagnostics_json(&resolution.path),
                        escape_json(&err.to_string())
                    );
                }
            },
            Err(err) => {
                ok = false;
                add_error_hints(&mut hints, &err);
                add_gs_lib_directory_hint(&mut hints);
                gci_json = format!(
                    r#"{{"ok":false,"checked":true,"error":"{}"}}"#,
                    escape_json(&err.to_string())
                );
            }
        }

        if live {
            match Session::login(config).and_then(|mut session| session.eval("3 + 4")) {
                Ok(Value::SmallInt(7)) => {
                    live_json = r#"{"ok":true,"checked":true,"result":7}"#.to_string();
                }
                Ok(value) => {
                    ok = false;
                    add_hint(
                        &mut hints,
                        "Confirm the stone can evaluate simple Smalltalk expressions and rerun gemstone-rs doctor --live.",
                    );
                    live_json = format!(
                        r#"{{"ok":false,"checked":true,"error":"expected 7","actual":"{}"}}"#,
                        escape_json(&format!("{value:?}"))
                    );
                }
                Err(err) => {
                    ok = false;
                    add_error_hints(&mut hints, &err);
                    live_json = format!(
                        r#"{{"ok":false,"checked":true,"error":"{}"}}"#,
                        escape_json(&err.to_string())
                    );
                }
            }
        }
    }

    println!(
        r#"{{"success":{},"environment":{},"strict":{},"config":{},"gciLibrary":{},"live":{},"hints":[{}]}}"#,
        ok,
        environment_json(),
        strict_json(strict, &strict_errors),
        config_json,
        gci_json,
        live_json,
        json_hint_array(&hints)
    );

    if ok {
        Ok(())
    } else {
        Err(CliError::Doctor(
            "doctor found setup problems; see report above".to_string(),
        ))
    }
}

fn print_env_value(name: &str, secret: bool) {
    match env::var(name) {
        Ok(value) if secret && !value.is_empty() => println!("  {name}: <set>"),
        Ok(value) if value.is_empty() => println!("  {name}: <empty>"),
        Ok(value) => println!("  {name}: {value}"),
        Err(_) => println!("  {name}: <unset>"),
    }
}

fn environment_json() -> String {
    let entries = [
        ("GS_LIB_PATH", false),
        ("GS_LIB", false),
        ("GEMSTONE", false),
        ("GS_STONE", false),
        ("GS_STONE_NAME", false),
        ("GS_HOST", false),
        ("GS_NETLDI", false),
        ("GS_GEM_SERVICE", false),
        ("GS_USERNAME", false),
        ("GS_PASSWORD", true),
        ("GS_HOST_USERNAME", false),
        ("GS_HOST_PASSWORD", true),
    ];
    let mut output = String::from("{");
    for (index, (name, secret)) in entries.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('"');
        output.push_str(name);
        output.push_str(r#"":"#);
        match env::var(name) {
            Ok(value) if value.is_empty() => output.push_str(r#"{"status":"empty"}"#),
            Ok(_) if *secret => output.push_str(r#"{"status":"set","masked":true}"#),
            Ok(value) => output.push_str(&format!(
                r#"{{"status":"set","value":"{}"}}"#,
                escape_json(&value)
            )),
            Err(_) => output.push_str(r#"{"status":"unset"}"#),
        }
    }
    output.push('}');
    output
}

fn escape_json(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            ch if ch.is_control() => result.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => result.push(ch),
        }
    }
    result
}

fn json_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!(r#""{}""#, escape_json(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn json_path_array(values: &[PathBuf]) -> String {
    values
        .iter()
        .map(|value| format!(r#""{}""#, escape_json(&value.display().to_string())))
        .collect::<Vec<_>>()
        .join(",")
}

fn json_hint_array(values: &[&'static str]) -> String {
    values
        .iter()
        .map(|value| format!(r#""{}""#, escape_json(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn strict_json(enabled: bool, errors: &[String]) -> String {
    format!(
        r#"{{"enabled":{},"ok":{},"errors":[{}]}}"#,
        enabled,
        errors.is_empty(),
        json_string_array(errors)
    )
}

fn path_diagnostics_json(path: &Path) -> String {
    let diagnostics = path_diagnostics(path);
    format!(
        r#"{{"exists":{},"isFile":{},"readable":{},"architecture":"{}"}}"#,
        diagnostics.exists,
        diagnostics.is_file,
        diagnostics.readable,
        escape_json(diagnostics.architecture)
    )
}

fn print_searched_paths(paths: &[PathBuf]) {
    if paths.is_empty() {
        return;
    }
    println!("  searched:");
    for path in paths {
        println!("    {}", path.display());
    }
}

fn print_path_diagnostics(prefix: &str, path: &Path) {
    let diagnostics = path_diagnostics(path);
    println!("{prefix}exists: {}", diagnostics.exists);
    println!("{prefix}is_file: {}", diagnostics.is_file);
    println!("{prefix}readable: {}", diagnostics.readable);
    println!("{prefix}architecture: {}", diagnostics.architecture);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PathDiagnostics {
    exists: bool,
    is_file: bool,
    readable: bool,
    architecture: &'static str,
}

fn path_diagnostics(path: &Path) -> PathDiagnostics {
    let metadata = fs::metadata(path);
    let exists = metadata.is_ok();
    let is_file = metadata.as_ref().is_ok_and(|metadata| metadata.is_file());
    let readable = fs::File::open(path).is_ok();
    PathDiagnostics {
        exists,
        is_file,
        readable,
        architecture: architecture_hint(path),
    }
}

fn architecture_hint(path: &Path) -> &'static str {
    let lower = path.display().to_string().to_ascii_lowercase();
    if lower.contains("arm64") || lower.contains("aarch64") {
        "arm64"
    } else if lower.contains("x86_64") || lower.contains("amd64") || lower.contains("x64") {
        "x86_64"
    } else {
        "unknown"
    }
}

fn add_error_hints(hints: &mut Vec<&'static str>, err: &GemStoneError) {
    match err {
        GemStoneError::MissingEnvironment("GS_USERNAME") => add_hint(
            hints,
            "Set GS_USERNAME to the GemStone account name before running live GemStone commands.",
        ),
        GemStoneError::MissingEnvironment("GS_PASSWORD") => add_hint(
            hints,
            "Set GS_PASSWORD for the GemStone account before running live GemStone commands.",
        ),
        GemStoneError::MissingConfig(_) => add_hint(
            hints,
            "Use Config::builder() to provide all required fields when environment variables are not used.",
        ),
        GemStoneError::Gci(_) => {
            add_hint(
                hints,
                "Set GS_LIB_PATH to the full libgcirpc path, or set GS_LIB/GEMSTONE to the GemStone product location.",
            );
            add_hint(
                hints,
                "If libgcirpc is found but will not load, check CPU architecture, file permissions, and dependent dynamic libraries.",
            );
        }
        GemStoneError::GemStone { .. } => add_hint(
            hints,
            "Check GS_STONE or GS_STONE_NAME, GS_HOST, GS_NETLDI, credentials, and stone status, then rerun gemstone-rs doctor --live.",
        ),
        GemStoneError::Nul(_)
        | GemStoneError::NotLoggedIn
        | GemStoneError::IllegalOop { .. }
        | GemStoneError::UnexpectedType { .. }
        | GemStoneError::Mapping { .. }
        | GemStoneError::NegativeSize(_)
        | GemStoneError::ArgumentCountTooLarge(_) => {}
        GemStoneError::MissingEnvironment(_) => add_hint(
            hints,
            "Set the missing GemStone environment variable or pass the value through Config::builder().",
        ),
    }
}

fn add_gs_lib_directory_hint(hints: &mut Vec<&'static str>) {
    if env_value(&|name| env::var(name).ok(), "GS_LIB").is_some() {
        add_hint(
            hints,
            "If GS_LIB is set, it must point at the GemStone lib directory that contains libgcirpc. Use GS_LIB_PATH for one exact library file.",
        );
    }
}

fn add_gci_architecture_hints(hints: &mut Vec<&'static str>, path: &Path) {
    let architecture = architecture_hint(path);
    let current = env::consts::ARCH;
    if architecture == "arm64" && current != "aarch64" {
        add_hint(
            hints,
            "The selected libgcirpc path looks arm64, but this Rust process is not aarch64. Use a matching GemStone installation or Rust toolchain.",
        );
    } else if architecture == "x86_64" && current != "x86_64" {
        add_hint(
            hints,
            "The selected libgcirpc path looks x86_64, but this Rust process is not x86_64. Use a matching GemStone installation or Rust toolchain.",
        );
    }
}

fn add_missing_credential_hints(hints: &mut Vec<&'static str>) {
    if env_var_missing_or_empty("GS_USERNAME") {
        add_hint(
            hints,
            "Set GS_USERNAME to the GemStone account name before running live GemStone commands.",
        );
    }
    if env_var_missing_or_empty("GS_PASSWORD") {
        add_hint(
            hints,
            "Set GS_PASSWORD for the GemStone account before running live GemStone commands.",
        );
    }
}

fn env_var_missing_or_empty(name: &str) -> bool {
    env::var(name).map(|value| value.is_empty()).unwrap_or(true)
}

fn add_hint(hints: &mut Vec<&'static str>, hint: &'static str) {
    if !hints.contains(&hint) {
        hints.push(hint);
    }
}

fn print_hints(hints: &[&'static str]) {
    if hints.is_empty() {
        return;
    }
    println!("hints:");
    for hint in hints {
        println!("  - {hint}");
    }
}

fn project_profiles_json(path: &Path, project: &profiles::ProjectProfiles) -> String {
    let profiles = project
        .profiles
        .iter()
        .map(codegen_profile_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"path":"{}","kind":"{}","version":{},"profiles":[{}]}}"#,
        escape_json(&path.display().to_string()),
        escape_json(&project.kind),
        project.version,
        profiles
    )
}

fn codegen_profile_json(profile: &profiles::CodegenProfile) -> String {
    format!(
        r#"{{"name":"{}","config":{},"root":{},"mapped":{},"className":{}}}"#,
        escape_json(&profile.name),
        optional_json_string(profile.config.as_deref()),
        optional_json_string(profile.root.as_deref()),
        optional_json_string(profile.mapped.as_deref()),
        optional_json_string(profile.class_name.as_deref())
    )
}

fn optional_json_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!(r#""{}""#, escape_json(value)),
        None => "null".to_string(),
    }
}

fn print_value(session: &mut Session, value: Value) -> Result<(), CliError> {
    match value {
        Value::Nil => println!("nil"),
        Value::Bool(value) => println!("{value}"),
        Value::SmallInt(value) => println!("{value}"),
        Value::Char(value) => println!("{value}"),
        Value::String(value) => println!("{value}"),
        Value::Oop(oop) => {
            let printed = session.perform_oop(oop, "printString", &[])?;
            println!("{}", session.fetch_string(printed)?);
        }
    }
    Ok(())
}

fn print_oop_inspection(session: &mut Session, oop: Oop) -> Result<(), CliError> {
    let class = session.fetch_class(oop)?;
    let printed = session.perform_oop(oop, "printString", &[])?;
    println!("oop: {}", oop.raw());
    println!("class_oop: {}", class.raw());
    println!("printString: {}", session.fetch_string(printed)?);
    Ok(())
}

fn print_lines(values: impl IntoIterator<Item = String>) {
    for value in values {
        println!("{value}");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ExampleInfo {
    name: &'static str,
    title: &'static str,
    command: &'static str,
    category: &'static str,
    requires_live: bool,
    description: &'static str,
}

const EXAMPLES: &[ExampleInfo] = &[
    ExampleInfo {
        name: "hello_gemstone",
        title: "Hello GemStone",
        command: "cargo run -p gemstone-rs --example hello_gemstone",
        category: "quickstart",
        requires_live: true,
        description: "Verify environment loading, login, session id, and a tiny eval.",
    },
    ExampleInfo {
        name: "quickstart",
        title: "Quickstart",
        command: "cargo run -p gemstone-rs --example quickstart",
        category: "quickstart",
        requires_live: true,
        description: "Smallest live eval plus UserGlobals write/read round trip.",
    },
    ExampleInfo {
        name: "eval",
        title: "Eval",
        command: "cargo run -p gemstone-rs --example eval",
        category: "quickstart",
        requires_live: true,
        description: "Minimal Session::eval(\"3 + 4\") shape.",
    },
    ExampleInfo {
        name: "browser",
        title: "Browser",
        command: "cargo run -p gemstone-rs --example browser",
        category: "browser",
        requires_live: true,
        description: "Dictionaries, protocols, methods, and method source.",
    },
    ExampleInfo {
        name: "live_smoke_cookbook",
        title: "Live smoke cookbook",
        command: "cargo run -p gemstone-rs --example live_smoke_cookbook",
        category: "smoke",
        requires_live: true,
        description: "Login, eval, global round-trip, perform, and transaction checks.",
    },
    ExampleInfo {
        name: "transactions",
        title: "Transactions",
        command: "cargo run -p gemstone-rs --example transactions",
        category: "transactions",
        requires_live: true,
        description: "Commit-on-success and abort-on-error behavior.",
    },
    ExampleInfo {
        name: "oop_values",
        title: "OOP values",
        command: "cargo run -p gemstone-rs --example oop_values",
        category: "values",
        requires_live: true,
        description: "Explicit OOP/value conversion and export-set retention.",
    },
    ExampleInfo {
        name: "bridge_root_mapping",
        title: "BridgeRoot mapping",
        command: "cargo run -p gemstone-rs --example bridge_root_mapping",
        category: "mapping",
        requires_live: true,
        description: "MagLev-style bridge-root storage with explicit Rust value mapping.",
    },
    ExampleInfo {
        name: "derive_mapping",
        title: "Derive mapping",
        command: "cargo run -p gemstone-rs --example derive_mapping",
        category: "mapping",
        requires_live: true,
        description: "#[derive(BridgeMapped)] with nested structs, vectors, maps, optionals, and symbol keys.",
    },
    ExampleInfo {
        name: "codegen_preview",
        title: "Codegen preview",
        command: "cargo run -p gemstone-rs --example codegen_preview",
        category: "codegen",
        requires_live: false,
        description: "Offline wrapper generation preview without writing files.",
    },
    ExampleInfo {
        name: "codegen_workflow",
        title: "Codegen workflow",
        command: "cargo run -p gemstone-rs --example codegen_workflow",
        category: "codegen",
        requires_live: false,
        description: "Config, preview, diff, check, and generate in one offline run.",
    },
    ExampleInfo {
        name: "codegen_discover",
        title: "Codegen discovery",
        command: "cargo run -p gemstone-rs --example codegen_discover",
        category: "codegen",
        requires_live: true,
        description: "Discover a starter wrapper config from live GemStone classes.",
    },
    ExampleInfo {
        name: "codegen_discover_mapping",
        title: "Mapping discovery",
        command: "cargo run -p gemstone-rs --example codegen_discover_mapping",
        category: "codegen",
        requires_live: true,
        description: "Discover a starter BridgeRoot mapping config from a live class.",
    },
    ExampleInfo {
        name: "generated_wrapper_app",
        title: "Generated wrapper app",
        command: "cargo run -p gemstone-rs --example generated_wrapper_app",
        category: "codegen",
        requires_live: true,
        description: "Call Object>>printString through checked-in generated Rust code.",
    },
    ExampleInfo {
        name: "generated_mapping_app",
        title: "Generated mapping app",
        command: "cargo run -p gemstone-rs --example generated_mapping_app",
        category: "mapping",
        requires_live: true,
        description: "Use generated BridgeMapped structs stored under BridgeRoot.",
    },
];

fn find_example(name: &str) -> Option<&'static ExampleInfo> {
    EXAMPLES.iter().find(|example| {
        example.name.eq_ignore_ascii_case(name) || example.title.eq_ignore_ascii_case(name)
    })
}

fn print_examples_list(format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("gemstone-rs examples");
            println!("Use `gemstone-rs examples show <name>` for one command and details.");
            println!();
            for example in EXAMPLES {
                println!(
                    "{}\t{}\tlive={}\t{}",
                    example.name, example.category, example.requires_live, example.command
                );
                println!("  {}", example.description);
            }
        }
        OutputFormat::Json => {
            let examples = EXAMPLES
                .iter()
                .map(example_json)
                .collect::<Vec<_>>()
                .join(",");
            println!(r#"{{"examples":[{}]}}"#, examples);
        }
    }
}

fn print_example_details(example: &ExampleInfo, format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("example: {}", example.name);
            println!("title: {}", example.title);
            println!("category: {}", example.category);
            println!("requires_live: {}", example.requires_live);
            println!("command: {}", example.command);
            println!("description: {}", example.description);
        }
        OutputFormat::Json => println!("{}", example_json(example)),
    }
}

fn example_json(example: &ExampleInfo) -> String {
    format!(
        r#"{{"name":"{}","title":"{}","command":"{}","category":"{}","requiresLive":{},"description":"{}"}}"#,
        escape_json(example.name),
        escape_json(example.title),
        escape_json(example.command),
        escape_json(example.category),
        example.requires_live,
        escape_json(example.description)
    )
}

fn print_profile_list(path: &Path, project: &profiles::ProjectProfiles) {
    println!("profiles: {} ({})", path.display(), project.profiles.len());
    for profile in &project.profiles {
        println!(
            "{}\tconfig={}\troot={}\tmapped={}\tclassName={}",
            profile.name,
            profile_field(profile.config.as_deref()),
            profile_field(profile.root.as_deref()),
            profile_field(profile.mapped.as_deref()),
            profile_field(profile.class_name.as_deref())
        );
    }
}

fn print_project_profile(path: &Path, profile: &profiles::CodegenProfile) {
    println!("profile: {}", profile.name);
    println!("path: {}", path.display());
    println!("config: {}", profile_field(profile.config.as_deref()));
    println!("root: {}", profile_field(profile.root.as_deref()));
    println!("mapped: {}", profile_field(profile.mapped.as_deref()));
    println!(
        "className: {}",
        profile_field(profile.class_name.as_deref())
    );
}

fn print_resolved_project_profile(
    path: &Path,
    profile: &profiles::CodegenProfile,
    resolved: &Path,
) {
    print_project_profile(path, profile);
    println!("resolvedConfig: {}", resolved.display());
}

fn profile_field(value: Option<&str>) -> String {
    match value {
        Some("") => "\"\"".to_string(),
        Some(value) => value.to_string(),
        None => "-".to_string(),
    }
}

fn run_profile_check(path: &Path, format: OutputFormat) -> Result<(), CliError> {
    let report = profiles::check_file(path, None)?;
    match format {
        OutputFormat::Human => print_profile_check(&report),
        OutputFormat::Json => println!("{}", report.to_json()),
    }
    if report.ok() {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "one or more profiles in {} are stale or invalid",
            path.display()
        )))
    }
}

fn print_profile_check(report: &profiles::ProfileCheckReport) {
    let counts = report.counts();
    println!(
        "profile check: {} ({} profiles)",
        report.path.display(),
        counts.profile_count
    );
    println!(
        "summary: {} ok, {} stale, {} errors, {} total",
        counts.ok_count, counts.stale_count, counts.error_count, counts.profile_count
    );
    for entry in &report.entries {
        let mut line = format!(
            "{}\t{}\tconfig={}\toutput={}",
            entry.status(),
            entry.name,
            path_field(entry.config.as_deref()),
            path_field(entry.output.as_deref())
        );
        if let Some(error) = &entry.error {
            line.push_str("\terror=");
            line.push_str(error);
        }
        println!("{line}");
    }
}

fn path_field(value: Option<&Path>) -> String {
    value
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "-".to_string())
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Help,
    Doctor {
        live: bool,
        strict: bool,
        env_file: Option<PathBuf>,
        format: OutputFormat,
    },
    EnvSample,
    EnvWrite {
        path: PathBuf,
        force: bool,
    },
    ExamplesList {
        format: OutputFormat,
    },
    ExamplesShow {
        name: String,
        format: OutputFormat,
    },
    Eval {
        source: String,
        env_file: Option<PathBuf>,
    },
    BrowseDictionaries,
    BrowseClasses {
        dictionary: String,
    },
    BrowseProtocols {
        class_name: String,
        dictionary: String,
        meta: bool,
    },
    BrowseMethods {
        class_name: String,
        protocol: String,
        dictionary: String,
        meta: bool,
    },
    BrowseSource {
        class_name: String,
        selector: String,
        dictionary: String,
        meta: bool,
    },
    InspectOop {
        oop: Oop,
    },
    BridgeRoot {
        root: String,
    },
    BridgeKeys {
        root: String,
    },
    BridgeGet {
        root: String,
        key: String,
        key_type: BridgeKeyType,
    },
    BridgeInspect {
        root: String,
        key: String,
        key_type: BridgeKeyType,
    },
    BridgePut {
        root: String,
        key: String,
        key_type: BridgeKeyType,
        value: BridgeCliValue,
    },
    BridgeRemove {
        root: String,
        key: String,
        key_type: BridgeKeyType,
    },
    BridgeSampleConfig {
        mapped_name: String,
    },
    ProfileSample,
    ProfileInit {
        path: PathBuf,
    },
    ProfileValidate {
        path: PathBuf,
        format: OutputFormat,
    },
    ProfileList {
        path: PathBuf,
        format: OutputFormat,
    },
    ProfileShow {
        path: PathBuf,
        name: String,
        format: OutputFormat,
    },
    ProfileResolve {
        path: PathBuf,
        name: String,
        format: OutputFormat,
    },
    ProfileCheck {
        path: PathBuf,
        format: OutputFormat,
    },
    CodegenInit {
        config: PathBuf,
    },
    CodegenPreview {
        config: PathBuf,
    },
    CodegenPreviewProfile {
        name: String,
        profiles: PathBuf,
    },
    CodegenDiff {
        config: PathBuf,
    },
    CodegenDiffProfile {
        name: String,
        profiles: PathBuf,
    },
    CodegenCheck {
        config: PathBuf,
    },
    CodegenExplain {
        config: PathBuf,
        format: OutputFormat,
    },
    CodegenExplainProfile {
        name: String,
        profiles: PathBuf,
        format: OutputFormat,
    },
    CodegenCheckProfile {
        name: String,
        profiles: PathBuf,
    },
    CodegenGenerate {
        config: PathBuf,
    },
    CodegenGenerateProfile {
        name: String,
        profiles: PathBuf,
    },
    CodegenDiscover {
        config: PathBuf,
        classes: Vec<String>,
    },
    CodegenDiscoverMapping {
        config: PathBuf,
        mapped_name: String,
        class_name: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Human,
    Json,
}

fn parse_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::Help);
    };

    match command {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "doctor" => parse_doctor_command(&args[1..]),
        "env" => parse_env_command(&args[1..]),
        "examples" | "example" => parse_examples_command(&args[1..]),
        "eval" => parse_eval_command(&args[1..]),
        "browse" => parse_browse_command(&args[1..]),
        "bridge" => parse_bridge_command(&args[1..]),
        "profile" => parse_profile_command(&args[1..]),
        "inspect" => match (args.get(1).map(String::as_str), args.get(2)) {
            (Some("oop"), Some(raw)) => Ok(Command::InspectOop {
                oop: Oop(parse_u64(raw)?),
            }),
            _ => Err(CliError::usage("expected: inspect oop <raw-oop>")),
        },
        "codegen" => match args.get(1).map(String::as_str) {
            Some("init") => Ok(Command::CodegenInit {
                config: optional_path(args.get(2)),
            }),
            Some("preview") => Ok(Command::CodegenPreview {
                config: optional_path(args.get(2)),
            }),
            Some("preview-profile") => parse_codegen_profile_command(&args[2..], |name, profiles| {
                Command::CodegenPreviewProfile { name, profiles }
            }),
            Some("diff") => Ok(Command::CodegenDiff {
                config: optional_path(args.get(2)),
            }),
            Some("diff-profile") => parse_codegen_profile_command(&args[2..], |name, profiles| {
                Command::CodegenDiffProfile { name, profiles }
            }),
            Some("check") => Ok(Command::CodegenCheck {
                config: optional_path(args.get(2)),
            }),
            Some("explain") => parse_codegen_explain_command(&args[2..]),
            Some("explain-profile") => parse_codegen_explain_profile_command(&args[2..]),
            Some("check-profile") => parse_codegen_profile_command(&args[2..], |name, profiles| {
                Command::CodegenCheckProfile { name, profiles }
            }),
            Some("generate") => Ok(Command::CodegenGenerate {
                config: optional_path(args.get(2)),
            }),
            Some("generate-profile") => {
                parse_codegen_profile_command(&args[2..], |name, profiles| {
                    Command::CodegenGenerateProfile { name, profiles }
                })
            }
            Some("discover") => Ok(Command::CodegenDiscover {
                config: optional_path(args.get(2)),
                classes: args.iter().skip(3).cloned().collect(),
            }),
            Some("discover-mapping") => Ok(Command::CodegenDiscoverMapping {
                config: optional_path(args.get(2)),
                mapped_name: args
                    .get(3)
                    .cloned()
                    .ok_or_else(|| CliError::usage("missing mapped struct name"))?,
                class_name: args
                    .get(4)
                    .cloned()
                    .ok_or_else(|| CliError::usage("missing GemStone class"))?,
            }),
            _ => Err(CliError::usage(
                "expected: codegen init|preview|preview-profile|diff|diff-profile|check|check-profile|explain|explain-profile|generate|generate-profile|discover|discover-mapping",
            )),
        },
        _ => Err(CliError::usage(format!("unknown command: {command}"))),
    }
}

fn parse_env_command(args: &[String]) -> Result<Command, CliError> {
    match args.first().map(String::as_str) {
        None | Some("sample") => {
            if args.len() > 1 {
                return Err(CliError::usage("expected: env sample"));
            }
            Ok(Command::EnvSample)
        }
        Some("write") => parse_env_write_command(&args[1..]),
        Some("-h" | "--help") => Err(CliError::usage("expected: env sample")),
        Some(command) => Err(CliError::usage(format!(
            "unknown env command: {command}; expected env sample|write"
        ))),
    }
}

fn parse_env_write_command(args: &[String]) -> Result<Command, CliError> {
    let mut force = false;
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "--force" => force = true,
            "-h" | "--help" => return Err(CliError::usage("expected: env write [path] [--force]")),
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown env write option: {option}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected env write argument: {value}"
                )))
            }
        }
    }
    Ok(Command::EnvWrite {
        path: path.unwrap_or_else(|| PathBuf::from(".env.gemstone-rs")),
        force,
    })
}

fn parse_examples_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::ExamplesList {
            format: OutputFormat::Human,
        });
    };
    match command {
        "list" => parse_examples_list_command(&args[1..]),
        "show" => parse_examples_show_command(&args[1..]),
        "--json" => Ok(Command::ExamplesList {
            format: OutputFormat::Json,
        }),
        "-h" | "--help" => Err(CliError::usage(
            "expected: examples [list [--json] | show <name> [--json]]",
        )),
        name => {
            if args.len() > 2 {
                return Err(CliError::usage(
                    "expected: examples [list [--json] | show <name> [--json]]",
                ));
            }
            let format = if args.get(1).is_some_and(|arg| arg == "--json") {
                OutputFormat::Json
            } else if args.len() == 1 {
                OutputFormat::Human
            } else {
                return Err(CliError::usage(format!(
                    "unexpected examples argument: {}",
                    args[1]
                )));
            };
            Ok(Command::ExamplesShow {
                name: name.to_string(),
                format,
            })
        }
    }
}

fn parse_examples_list_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => {
                return Err(CliError::usage("expected: examples list [--json]"));
            }
            other => {
                return Err(CliError::usage(format!(
                    "unexpected examples argument: {other}"
                )))
            }
        }
    }
    Ok(Command::ExamplesList { format })
}

fn parse_examples_show_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut name = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => {
                return Err(CliError::usage("expected: examples show <name> [--json]"));
            }
            value if name.is_none() => name = Some(value.to_string()),
            other => {
                return Err(CliError::usage(format!(
                    "unexpected examples argument: {other}"
                )))
            }
        }
    }
    Ok(Command::ExamplesShow {
        name: name.ok_or_else(|| CliError::usage("missing example name"))?,
        format,
    })
}

fn parse_eval_command(args: &[String]) -> Result<Command, CliError> {
    let mut source = None;
    let mut env_file = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--env-file" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::usage("missing path after --env-file"))?;
                env_file = Some(PathBuf::from(value));
            }
            option if option.starts_with("--env-file=") => {
                let (_, value) = option.split_once('=').unwrap_or_default();
                if value.is_empty() {
                    return Err(CliError::usage("missing path after --env-file="));
                }
                env_file = Some(PathBuf::from(value));
            }
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: eval [--env-file <path>] <smalltalk>",
                ));
            }
            value if source.is_none() => source = Some(value.to_string()),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected eval argument: {value}"
                )))
            }
        }
        index += 1;
    }
    Ok(Command::Eval {
        source: source.ok_or_else(|| CliError::usage("missing Smalltalk source for eval"))?,
        env_file,
    })
}

fn parse_profile_command(args: &[String]) -> Result<Command, CliError> {
    match args.first().map(String::as_str) {
        Some("sample") => {
            if args.len() > 1 {
                return Err(CliError::usage("expected: profile sample"));
            }
            Ok(Command::ProfileSample)
        }
        Some("init") => {
            if args.len() > 2 {
                return Err(CliError::usage("expected: profile init [path]"));
            }
            Ok(Command::ProfileInit {
                path: args
                    .get(1)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(profiles::DEFAULT_PROFILE_PATH)),
            })
        }
        Some("validate") => parse_profile_validate_command(&args[1..]),
        Some("list") => parse_profile_path_format_command(
            &args[1..],
            "profile list [--json] [path]",
            |path, format| Command::ProfileList { path, format },
        ),
        Some("show") => parse_profile_show_command(&args[1..]),
        Some("resolve") => parse_profile_name_path_format_command(
            &args[1..],
            "profile resolve <name> [--json] [path]",
            |name, path, format| Command::ProfileResolve { path, name, format },
        ),
        Some("check") => parse_profile_path_format_command(
            &args[1..],
            "profile check [--json] [path]",
            |path, format| Command::ProfileCheck { path, format },
        ),
        _ => Err(CliError::usage(
            "expected: profile sample | profile init [path] | profile validate [--json] [path] | profile list [--json] [path] | profile show <name> [--json] [path] | profile resolve <name> [--json] [path] | profile check [--json] [path]",
        )),
    }
}

fn parse_profile_validate_command(args: &[String]) -> Result<Command, CliError> {
    parse_profile_path_format_command(args, "profile validate [--json] [path]", |path, format| {
        Command::ProfileValidate { path, format }
    })
}

fn parse_profile_path_format_command(
    args: &[String],
    usage: &'static str,
    build: impl FnOnce(PathBuf, OutputFormat) -> Command,
) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => return Err(CliError::usage(format!("expected: {usage}"))),
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!("unknown profile option: {option}")));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected profile argument: {value}"
                )));
            }
        }
    }

    Ok(build(
        path.unwrap_or_else(|| PathBuf::from(profiles::DEFAULT_PROFILE_PATH)),
        format,
    ))
}

fn parse_profile_show_command(args: &[String]) -> Result<Command, CliError> {
    parse_profile_name_path_format_command(
        args,
        "profile show <name> [--json] [path]",
        |name, path, format| Command::ProfileShow { path, name, format },
    )
}

fn parse_profile_name_path_format_command(
    args: &[String],
    usage: &'static str,
    build: impl FnOnce(String, PathBuf, OutputFormat) -> Command,
) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut name = None;
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => return Err(CliError::usage(format!("expected: {usage}"))),
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!("unknown profile option: {option}")));
            }
            value if name.is_none() => name = Some(value.to_string()),
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected profile show argument: {value}"
                )));
            }
        }
    }

    Ok(build(
        name.ok_or_else(|| CliError::usage("missing profile name"))?,
        path.unwrap_or_else(|| PathBuf::from(profiles::DEFAULT_PROFILE_PATH)),
        format,
    ))
}

fn parse_codegen_profile_command(
    args: &[String],
    build: impl FnOnce(String, PathBuf) -> Command,
) -> Result<Command, CliError> {
    if args.len() > 2 {
        return Err(CliError::usage(
            "expected: codegen <action>-profile <profile-name> [profile-file]",
        ));
    }
    let name = args
        .first()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| CliError::usage("missing profile name"))?;
    let profiles = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(profiles::DEFAULT_PROFILE_PATH));
    Ok(build(name, profiles))
}

fn parse_codegen_explain_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut config = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: codegen explain [--json] [config]",
                ));
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown codegen explain option: {option}"
                )));
            }
            value if config.is_none() => config = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected codegen explain argument: {value}"
                )));
            }
        }
    }
    Ok(Command::CodegenExplain {
        config: config.unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH)),
        format,
    })
}

fn parse_codegen_explain_profile_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut name = None;
    let mut profiles = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: codegen explain-profile [--json] <profile-name> [profile-file]",
                ));
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown codegen explain-profile option: {option}"
                )));
            }
            value if name.is_none() => name = Some(value.to_string()),
            value if profiles.is_none() => profiles = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected codegen explain-profile argument: {value}"
                )));
            }
        }
    }
    Ok(Command::CodegenExplainProfile {
        name: name.ok_or_else(|| CliError::usage("missing profile name"))?,
        profiles: profiles.unwrap_or_else(|| PathBuf::from(profiles::DEFAULT_PROFILE_PATH)),
        format,
    })
}

fn parse_u64(value: &str) -> Result<u64, CliError> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
    } else {
        value.parse()
    }
    .map_err(|_| CliError::usage(format!("invalid OOP: {value}")))
}

fn parse_doctor_command(args: &[String]) -> Result<Command, CliError> {
    let mut live = false;
    let mut strict = false;
    let mut env_file = None;
    let mut format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--live" => live = true,
            "--strict" => strict = true,
            "--env-file" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| CliError::usage("missing path after --env-file"))?;
                env_file = Some(PathBuf::from(value));
            }
            option if option.starts_with("--env-file=") => {
                let (_, value) = option.split_once('=').unwrap_or_default();
                if value.is_empty() {
                    return Err(CliError::usage("missing path after --env-file="));
                }
                env_file = Some(PathBuf::from(value));
            }
            "--json" => format = OutputFormat::Json,
            other => return Err(CliError::usage(format!("unknown doctor option: {other}"))),
        }
        index += 1;
    }
    Ok(Command::Doctor {
        live,
        strict,
        env_file,
        format,
    })
}

fn parse_browse_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::usage(
            "expected: browse dictionaries|classes|protocols|methods|source",
        ));
    };
    let (values, meta) = split_meta_flag(&args[1..]);
    match command {
        "dictionaries" => Ok(Command::BrowseDictionaries),
        "classes" => Ok(Command::BrowseClasses {
            dictionary: values
                .first()
                .cloned()
                .unwrap_or_else(|| "UserGlobals".to_string()),
        }),
        "protocols" => Ok(Command::BrowseProtocols {
            class_name: required_value(&values, 0, "missing class for browse protocols")?,
            dictionary: values.get(1).cloned().unwrap_or_default(),
            meta,
        }),
        "methods" => Ok(Command::BrowseMethods {
            class_name: required_value(&values, 0, "missing class for browse methods")?,
            protocol: values
                .get(1)
                .cloned()
                .unwrap_or_else(|| ALL_PROTOCOLS.to_string()),
            dictionary: values.get(2).cloned().unwrap_or_default(),
            meta,
        }),
        "source" => Ok(Command::BrowseSource {
            class_name: required_value(&values, 0, "missing class for browse source")?,
            selector: values.get(1).cloned().unwrap_or_default(),
            dictionary: values.get(2).cloned().unwrap_or_default(),
            meta,
        }),
        _ => Err(CliError::usage(
            "expected: browse dictionaries|classes|protocols|methods|source",
        )),
    }
}

fn parse_bridge_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::usage(
            "expected: bridge root|keys|get|inspect|put|put-string|put-symbol|put-smallint|put-bool|remove|sample-config",
        ));
    };
    match command {
        "root" => {
            let options = parse_bridge_options(&args[1..])?;
            Ok(Command::BridgeRoot { root: options.root })
        }
        "keys" => {
            let options = parse_bridge_options(&args[1..])?;
            Ok(Command::BridgeKeys { root: options.root })
        }
        "get" => {
            let key = args
                .get(1)
                .cloned()
                .ok_or_else(|| CliError::usage("missing key for bridge get"))?;
            let options = parse_bridge_options(&args[2..])?;
            Ok(Command::BridgeGet {
                root: options.root,
                key,
                key_type: options.key_type,
            })
        }
        "inspect" => {
            let key = args
                .get(1)
                .cloned()
                .ok_or_else(|| CliError::usage("missing key for bridge inspect"))?;
            let options = parse_bridge_options(&args[2..])?;
            Ok(Command::BridgeInspect {
                root: options.root,
                key,
                key_type: options.key_type,
            })
        }
        "put" => parse_bridge_put_command(&args[1..], "bridge put", None),
        "put-string" => parse_bridge_put_command(
            &args[1..],
            "bridge put-string",
            Some(BridgeValueType::String),
        ),
        "put-symbol" => parse_bridge_put_command(
            &args[1..],
            "bridge put-symbol",
            Some(BridgeValueType::Symbol),
        ),
        "put-smallint" => parse_bridge_put_command(
            &args[1..],
            "bridge put-smallint",
            Some(BridgeValueType::SmallInt),
        ),
        "put-bool" => {
            parse_bridge_put_command(&args[1..], "bridge put-bool", Some(BridgeValueType::Bool))
        }
        "remove" => {
            let key = args
                .get(1)
                .cloned()
                .ok_or_else(|| CliError::usage("missing key for bridge remove"))?;
            let options = parse_bridge_options(&args[2..])?;
            Ok(Command::BridgeRemove {
                root: options.root,
                key,
                key_type: options.key_type,
            })
        }
        "sample-config" => Ok(Command::BridgeSampleConfig {
            mapped_name: args
                .get(1)
                .cloned()
                .unwrap_or_else(|| "BookingDraft".to_string()),
        }),
        _ => Err(CliError::usage(
            "expected: bridge root|keys|get|inspect|put|put-string|put-symbol|put-smallint|put-bool|remove|sample-config",
        )),
    }
}

fn parse_bridge_put_command(
    args: &[String],
    command_name: &'static str,
    fixed_value_type: Option<BridgeValueType>,
) -> Result<Command, CliError> {
    let key = args
        .first()
        .cloned()
        .ok_or_else(|| CliError::usage(format!("missing key for {command_name}")))?;
    let raw_value = args
        .get(1)
        .cloned()
        .ok_or_else(|| CliError::usage(format!("missing value for {command_name}")))?;
    let options = parse_bridge_options(&args[2..])?;
    if fixed_value_type.is_some() && options.value_type.is_some() {
        return Err(CliError::usage(format!(
            "{command_name} has a fixed value type; use `bridge put --type ...` for explicit type selection"
        )));
    }
    Ok(Command::BridgePut {
        root: options.root,
        key,
        key_type: options.key_type,
        value: BridgeCliValue {
            raw: raw_value,
            value_type: fixed_value_type
                .or(options.value_type)
                .unwrap_or(BridgeValueType::String),
        },
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BridgeCliValue {
    raw: String,
    value_type: BridgeValueType,
}

impl BridgeCliValue {
    fn to_bridge_value(&self) -> Result<BridgeValue, CliError> {
        self.value_type.parse_value(&self.raw)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BridgeOptions {
    root: String,
    key_type: BridgeKeyType,
    value_type: Option<BridgeValueType>,
}

fn parse_bridge_options(args: &[String]) -> Result<BridgeOptions, CliError> {
    let mut root = DEFAULT_BRIDGE_ROOT.to_string();
    let mut key_type = BridgeKeyType::String;
    let mut value_type = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                index += 1;
                root = args
                    .get(index)
                    .cloned()
                    .ok_or_else(|| CliError::usage("missing value for --root"))?;
            }
            option if option.starts_with("--root=") => {
                let (_, value) = option.split_once('=').unwrap_or_default();
                if value.is_empty() {
                    return Err(CliError::usage("missing value for --root="));
                }
                root = value.to_string();
            }
            "--symbol" => key_type = BridgeKeyType::Symbol,
            "--string" => key_type = BridgeKeyType::String,
            "--key-type" => {
                index += 1;
                key_type = parse_bridge_key_type(
                    args.get(index)
                        .ok_or_else(|| CliError::usage("missing value for --key-type"))?,
                )?;
            }
            option if option.starts_with("--key-type=") => {
                let (_, value) = option.split_once('=').unwrap_or_default();
                if value.is_empty() {
                    return Err(CliError::usage("missing value for --key-type="));
                }
                key_type = parse_bridge_key_type(value)?;
            }
            "--type" | "--value-type" => {
                index += 1;
                value_type = Some(parse_bridge_value_type(
                    args.get(index)
                        .ok_or_else(|| CliError::usage("missing value for --type"))?,
                )?);
            }
            option if option.starts_with("--type=") || option.starts_with("--value-type=") => {
                let (name, value) = option.split_once('=').unwrap_or_default();
                if value.is_empty() {
                    return Err(CliError::usage(format!("missing value for {name}=")));
                }
                value_type = Some(parse_bridge_value_type(value)?);
            }
            other => {
                return Err(CliError::usage(format!("unknown bridge option: {other}")));
            }
        }
        index += 1;
    }
    Ok(BridgeOptions {
        root,
        key_type,
        value_type,
    })
}

fn parse_bridge_key_type(value: &str) -> Result<BridgeKeyType, CliError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "string" => Ok(BridgeKeyType::String),
        "symbol" => Ok(BridgeKeyType::Symbol),
        _ => Err(CliError::usage(format!(
            "expected bridge key type String or Symbol, got {value}"
        ))),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BridgeValueType {
    String,
    Symbol,
    SmallInt,
    Bool,
}

impl BridgeValueType {
    fn parse_value(self, value: &str) -> Result<BridgeValue, CliError> {
        match self {
            Self::String => Ok(BridgeValue::from(value.to_string())),
            Self::Symbol => Ok(BridgeValue::Symbol(value.to_string())),
            Self::SmallInt => value
                .parse::<i64>()
                .map(BridgeValue::from)
                .map_err(|_| CliError::usage(format!("expected SmallInt value, got {value}"))),
            Self::Bool => parse_bool_value(value).map(BridgeValue::from),
        }
    }
}

fn parse_bridge_value_type(value: &str) -> Result<BridgeValueType, CliError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "string" | "str" => Ok(BridgeValueType::String),
        "symbol" | "sym" => Ok(BridgeValueType::Symbol),
        "smallint" | "small_int" | "int" | "integer" => Ok(BridgeValueType::SmallInt),
        "bool" | "boolean" => Ok(BridgeValueType::Bool),
        _ => Err(CliError::usage(format!(
            "expected bridge value type String, Symbol, SmallInt, or Bool, got {value}"
        ))),
    }
}

fn parse_bool_value(value: &str) -> Result<bool, CliError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(CliError::usage(format!("expected Bool value, got {value}"))),
    }
}

fn split_meta_flag(args: &[String]) -> (Vec<String>, bool) {
    let mut meta = false;
    let mut values = Vec::new();
    for value in args {
        if value == "--meta" || value == "--class-side" {
            meta = true;
        } else {
            values.push(value.clone());
        }
    }
    (values, meta)
}

fn required_value(
    values: &[String],
    index: usize,
    message: &'static str,
) -> Result<String, CliError> {
    values
        .get(index)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| CliError::usage(message))
}

fn optional_path(value: Option<&String>) -> PathBuf {
    value
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH))
}

fn default_codegen_output(config: &std::path::Path) -> PathBuf {
    config
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("generated/gemstone_wrappers.rs")
}

fn profile_codegen_config_path(
    profile_file: &Path,
    profile_name: &str,
) -> Result<PathBuf, CliError> {
    let project = profiles::load_file(profile_file)?;
    let profile = project.get(profile_name).ok_or_else(|| {
        CliError::CodegenCheck(format!(
            "profile {profile_name} not found in {}",
            profile_file.display()
        ))
    })?;
    profile.resolved_config_path().map_err(CliError::Profiles)
}

fn run_codegen_diff(config_path: &Path) -> Result<(), CliError> {
    let config = codegen::Config::from_file(config_path)?;
    let report = codegen::diff(&config)?;
    if report.up_to_date {
        println!("codegen ok: {} is up to date", report.output.display());
        Ok(())
    } else {
        print!("{}", report.diff);
        Err(CliError::CodegenCheck(format!(
            "{} is stale or missing",
            report.output.display()
        )))
    }
}

fn run_codegen_check(config_path: &Path) -> Result<(), CliError> {
    let config = codegen::Config::from_file(config_path)?;
    let report = codegen::check(&config)?;
    if report.up_to_date {
        println!("codegen ok: {} is up to date", report.output.display());
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "{} is stale or missing; run `gemstone-rs codegen generate {}`",
            report.output.display(),
            config_path.display()
        )))
    }
}

fn usage() -> &'static str {
    "usage:
  gemstone-rs [--env-file <path>] <command>
  gemstone-rs doctor [--env-file <path>] [--live] [--strict] [--json]
  gemstone-rs env sample
  gemstone-rs env write [path] [--force]
  gemstone-rs examples list [--json]
  gemstone-rs examples show <name> [--json]
  gemstone-rs eval [--env-file <path>] <smalltalk>
  gemstone-rs browse dictionaries [--env-file <path>]
  gemstone-rs browse classes [dictionary] [--env-file <path>]
  gemstone-rs browse protocols <class> [dictionary] [--meta] [--env-file <path>]
  gemstone-rs browse methods <class> [protocol] [dictionary] [--meta] [--env-file <path>]
  gemstone-rs browse source <class> [selector] [dictionary] [--meta] [--env-file <path>]
  gemstone-rs inspect oop <raw-oop> [--env-file <path>]
  gemstone-rs bridge root [--root <name>]
  gemstone-rs bridge keys [--root <name>]
  gemstone-rs bridge get <key> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge inspect <key> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge put <key> <value> [--type String|Symbol|SmallInt|Bool] [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge put-string <key> <value> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge put-symbol <key> <value> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge put-smallint <key> <value> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge put-bool <key> <value> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge remove <key> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge sample-config [mapped-name]
  gemstone-rs profile sample
  gemstone-rs profile init [path]
  gemstone-rs profile validate [--json] [path]
  gemstone-rs profile list [--json] [path]
  gemstone-rs profile show <name> [--json] [path]
  gemstone-rs profile resolve <name> [--json] [path]
  gemstone-rs profile check [--json] [path]
  gemstone-rs codegen init [config]
  gemstone-rs codegen preview [config] [--env-file <path>]
  gemstone-rs codegen preview-profile <profile-name> [profile-file]
  gemstone-rs codegen diff [config] [--env-file <path>]
  gemstone-rs codegen diff-profile <profile-name> [profile-file]
  gemstone-rs codegen check [config] [--env-file <path>]
  gemstone-rs codegen check-profile <profile-name> [profile-file]
  gemstone-rs codegen explain [--json] [config]
  gemstone-rs codegen explain-profile [--json] <profile-name> [profile-file]
  gemstone-rs codegen generate [config] [--env-file <path>]
  gemstone-rs codegen generate-profile <profile-name> [profile-file]
  gemstone-rs codegen discover [config] [class ...]
  gemstone-rs codegen discover-mapping [config] <mapped-name> <class>"
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    GemStone(GemStoneError),
    Codegen(codegen::Error),
    Profiles(profiles::Error),
    CodegenCheck(String),
    Doctor(String),
    EnvFile(String),
    Io(std::io::Error),
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::GemStone(err) => write!(f, "{err}"),
            Self::Codegen(err) => write!(f, "{err}"),
            Self::Profiles(err) => write!(f, "{err}"),
            Self::CodegenCheck(message) => write!(f, "{message}"),
            Self::Doctor(message) => write!(f, "{message}"),
            Self::EnvFile(message) => write!(f, "{message}"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl StdError for CliError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Usage(_) => None,
            Self::GemStone(err) => Some(err),
            Self::Codegen(err) => Some(err),
            Self::Profiles(err) => Some(err),
            Self::CodegenCheck(_) | Self::Doctor(_) | Self::EnvFile(_) => None,
            Self::Io(err) => Some(err),
        }
    }
}

impl From<GemStoneError> for CliError {
    fn from(value: GemStoneError) -> Self {
        Self::GemStone(value)
    }
}

impl From<codegen::Error> for CliError {
    fn from(value: codegen::Error) -> Self {
        Self::Codegen(value)
    }
}

impl From<profiles::Error> for CliError {
    fn from(value: profiles::Error) -> Self {
        Self::Profiles(value)
    }
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_help_without_live_gemstone() {
        assert_eq!(parse_command(&args(&[])).unwrap(), Command::Help);
        assert_eq!(parse_command(&args(&["--help"])).unwrap(), Command::Help);
    }

    #[test]
    fn parses_doctor_command() {
        assert_eq!(
            parse_command(&args(&["doctor"])).unwrap(),
            Command::Doctor {
                live: false,
                strict: false,
                env_file: None,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["doctor", "--live"])).unwrap(),
            Command::Doctor {
                live: true,
                strict: false,
                env_file: None,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "doctor",
                "--json",
                "--live",
                "--strict",
                "--env-file",
                ".env.gemstone-rs"
            ]))
            .unwrap(),
            Command::Doctor {
                live: true,
                strict: true,
                env_file: Some(PathBuf::from(".env.gemstone-rs")),
                format: OutputFormat::Json,
            }
        );
    }

    #[test]
    fn parses_env_sample_command() {
        assert_eq!(parse_command(&args(&["env"])).unwrap(), Command::EnvSample);
        assert_eq!(
            parse_command(&args(&["env", "sample"])).unwrap(),
            Command::EnvSample
        );
        assert_eq!(
            parse_command(&args(&["env", "write"])).unwrap(),
            Command::EnvWrite {
                path: PathBuf::from(".env.gemstone-rs"),
                force: false,
            }
        );
        assert_eq!(
            parse_command(&args(&["env", "write", "demo.env", "--force"])).unwrap(),
            Command::EnvWrite {
                path: PathBuf::from("demo.env"),
                force: true,
            }
        );
    }

    #[test]
    fn parses_examples_commands() {
        assert_eq!(
            parse_command(&args(&["examples"])).unwrap(),
            Command::ExamplesList {
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["examples", "list", "--json"])).unwrap(),
            Command::ExamplesList {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["examples", "show", "quickstart"])).unwrap(),
            Command::ExamplesShow {
                name: "quickstart".to_string(),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["examples", "quickstart", "--json"])).unwrap(),
            Command::ExamplesShow {
                name: "quickstart".to_string(),
                format: OutputFormat::Json,
            }
        );
    }

    #[test]
    fn examples_index_contains_quickstart_and_codegen_workflow() {
        let quickstart = find_example("quickstart").unwrap();
        assert!(quickstart.requires_live);
        assert!(quickstart.command.contains("--example quickstart"));

        let workflow = find_example("codegen workflow").unwrap();
        assert!(!workflow.requires_live);
        assert!(example_json(workflow).contains(r#""requiresLive":false"#));
    }

    #[test]
    fn env_sample_uses_current_non_secret_values_without_leaking_passwords() {
        let output = env_template_source_with(|name| match name {
            "GS_LIB" => Some("/opt/GemStone lib".to_string()),
            "GS_STONE_NAME" => Some("demoStone".to_string()),
            "GS_USERNAME" => Some("Data Curator".to_string()),
            "GS_PASSWORD" => Some("secret-password".to_string()),
            "GS_HOST_PASSWORD" => Some("host-secret".to_string()),
            _ => None,
        });

        assert!(output.contains("export GS_LIB='/opt/GemStone lib'"));
        assert!(output.contains("export GS_STONE='demoStone'"));
        assert!(output.contains("export GS_STONE_NAME='demoStone'"));
        assert!(output.contains("export GS_USERNAME='Data Curator'"));
        assert!(output.contains("# GS_PASSWORD is set in this shell"));
        assert!(output.contains("export GS_PASSWORD='<set-your-password>'"));
        assert!(output.contains("# GS_HOST_PASSWORD is set in this shell"));
        assert!(output.contains("export GS_HOST_PASSWORD='<set-host-password-if-needed>'"));
        assert!(!output.contains("secret-password"));
        assert!(!output.contains("host-secret"));

        let default_output = env_template_source_with(|_| None);
        assert!(
            default_output.contains("# export GS_HOST_PASSWORD='<set-host-password-if-needed>'")
        );
    }

    #[test]
    fn env_write_refuses_existing_file_without_force() {
        let path = std::env::temp_dir().join(format!(
            "gemstone-rs-env-write-test-{}.env",
            std::process::id()
        ));
        fs::write(&path, "existing").unwrap();

        let err = write_env_template(&path, false).unwrap_err().to_string();
        assert!(err.contains("already exists"));
        write_env_template(&path, true).unwrap();
        let rewritten = fs::read_to_string(&path).unwrap();
        assert!(rewritten.contains("export GS_PASSWORD='<set-your-password>'"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn env_file_parser_reads_safe_template_values() {
        let path = Path::new(".env.gemstone-rs");
        let values = parse_env_file_source(
            r#"
# comment
export GS_USERNAME='Data'"'"'Curator'
export GS_PASSWORD='<set-your-password>'
GS_STONE_NAME="demoStone"
GEMSTONE=/opt/gemstone # product root
"#,
            path,
        )
        .unwrap();

        assert_eq!(
            values,
            vec![
                ("GS_USERNAME".to_string(), "Data'Curator".to_string()),
                ("GS_STONE_NAME".to_string(), "demoStone".to_string()),
                ("GEMSTONE".to_string(), "/opt/gemstone".to_string()),
            ]
        );
    }

    #[test]
    fn env_file_parser_rejects_non_gs_variables() {
        let err = parse_env_file_source("PATH=/tmp\n", Path::new(".env.gemstone-rs")).unwrap_err();
        assert!(err.to_string().contains("only GEMSTONE and GS_*"));
    }

    #[test]
    fn shell_quote_handles_single_quotes() {
        assert_eq!(shell_quote("Data'Curator"), r#"'Data'"'"'Curator'"#);
    }

    #[test]
    fn doctor_hints_are_deduplicated_and_json_escaped() {
        let mut hints = Vec::new();
        add_hint(
            &mut hints,
            "Set GS_USERNAME before running \"live\" checks.",
        );
        add_hint(
            &mut hints,
            "Set GS_USERNAME before running \"live\" checks.",
        );

        assert_eq!(
            hints,
            vec!["Set GS_USERNAME before running \"live\" checks."]
        );
        assert_eq!(
            json_hint_array(&hints),
            r#""Set GS_USERNAME before running \"live\" checks.""#
        );
    }

    #[test]
    fn doctor_hints_cover_missing_environment() {
        let mut hints = Vec::new();
        add_error_hints(
            &mut hints,
            &GemStoneError::MissingEnvironment("GS_USERNAME"),
        );
        add_error_hints(
            &mut hints,
            &GemStoneError::MissingEnvironment("GS_PASSWORD"),
        );

        assert_eq!(
            hints,
            vec![
                "Set GS_USERNAME to the GemStone account name before running live GemStone commands.",
                "Set GS_PASSWORD for the GemStone account before running live GemStone commands."
            ]
        );
    }

    #[test]
    fn parses_eval_command() {
        assert_eq!(
            parse_command(&args(&["eval", "3 + 4"])).unwrap(),
            Command::Eval {
                source: "3 + 4".to_string(),
                env_file: None,
            }
        );
        assert_eq!(
            parse_command(&args(&["eval", "--env-file=.env.gemstone-rs", "3 + 4"])).unwrap(),
            Command::Eval {
                source: "3 + 4".to_string(),
                env_file: Some(PathBuf::from(".env.gemstone-rs")),
            }
        );
        assert_eq!(
            parse_command(&args(&["eval", "-1 + 2"])).unwrap(),
            Command::Eval {
                source: "-1 + 2".to_string(),
                env_file: None,
            }
        );
    }

    #[test]
    fn extracts_global_env_file_option_for_any_command() {
        assert_eq!(
            extract_env_file_option(&args(&[
                "--env-file",
                ".env.gemstone-rs",
                "browse",
                "dictionaries"
            ]))
            .unwrap(),
            (
                args(&["browse", "dictionaries"]),
                Some(PathBuf::from(".env.gemstone-rs"))
            )
        );
        assert_eq!(
            extract_env_file_option(&args(&[
                "codegen",
                "check",
                "--env-file=.env.gemstone-rs",
                "demo.codegen"
            ]))
            .unwrap(),
            (
                args(&["codegen", "check", "demo.codegen"]),
                Some(PathBuf::from(".env.gemstone-rs"))
            )
        );
    }

    #[test]
    fn parses_browse_commands() {
        assert_eq!(
            parse_command(&args(&["browse", "dictionaries"])).unwrap(),
            Command::BrowseDictionaries
        );
        assert_eq!(
            parse_command(&args(&["browse", "classes"])).unwrap(),
            Command::BrowseClasses {
                dictionary: "UserGlobals".to_string()
            }
        );
        assert_eq!(
            parse_command(&args(&["browse", "protocols", "Object"])).unwrap(),
            Command::BrowseProtocols {
                class_name: "Object".to_string(),
                dictionary: String::new(),
                meta: false,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "browse",
                "methods",
                "Object",
                "-- all --",
                "UserGlobals",
                "--meta"
            ]))
            .unwrap(),
            Command::BrowseMethods {
                class_name: "Object".to_string(),
                protocol: "-- all --".to_string(),
                dictionary: "UserGlobals".to_string(),
                meta: true,
            }
        );
    }

    #[test]
    fn parses_inspect_oop_decimal_and_hex() {
        assert_eq!(
            parse_command(&args(&["inspect", "oop", "20"])).unwrap(),
            Command::InspectOop { oop: Oop(20) }
        );
        assert_eq!(
            parse_command(&args(&["inspect", "oop", "0x14"])).unwrap(),
            Command::InspectOop { oop: Oop(20) }
        );
    }

    #[test]
    fn parses_bridge_commands() {
        assert_eq!(
            parse_command(&args(&["bridge", "root"])).unwrap(),
            Command::BridgeRoot {
                root: DEFAULT_BRIDGE_ROOT.to_string()
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "root", "--root", "DemoRoot"])).unwrap(),
            Command::BridgeRoot {
                root: "DemoRoot".to_string()
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "keys", "--root", "DemoRoot"])).unwrap(),
            Command::BridgeKeys {
                root: "DemoRoot".to_string()
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "get", "BookingDraft", "--symbol"])).unwrap(),
            Command::BridgeGet {
                root: DEFAULT_BRIDGE_ROOT.to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::Symbol,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "inspect",
                "BookingDraft",
                "--key-type",
                "String",
                "--root",
                "DemoRoot"
            ]))
            .unwrap(),
            Command::BridgeInspect {
                root: "DemoRoot".to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::String,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "put",
                "BookingDraft",
                "42",
                "--type=SmallInt",
                "--root=DemoRoot"
            ]))
            .unwrap(),
            Command::BridgePut {
                root: "DemoRoot".to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::String,
                value: BridgeCliValue {
                    raw: "42".to_string(),
                    value_type: BridgeValueType::SmallInt,
                },
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "put-string", "BookingStatus", "ready"])).unwrap(),
            Command::BridgePut {
                root: DEFAULT_BRIDGE_ROOT.to_string(),
                key: "BookingStatus".to_string(),
                key_type: BridgeKeyType::String,
                value: BridgeCliValue {
                    raw: "ready".to_string(),
                    value_type: BridgeValueType::String,
                },
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "put-symbol",
                "BookingState",
                "confirmed",
                "--key-type=Symbol"
            ]))
            .unwrap(),
            Command::BridgePut {
                root: DEFAULT_BRIDGE_ROOT.to_string(),
                key: "BookingState".to_string(),
                key_type: BridgeKeyType::Symbol,
                value: BridgeCliValue {
                    raw: "confirmed".to_string(),
                    value_type: BridgeValueType::Symbol,
                },
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "put",
                "BookingState",
                "confirmed",
                "--type",
                "Symbol"
            ]))
            .unwrap(),
            Command::BridgePut {
                root: DEFAULT_BRIDGE_ROOT.to_string(),
                key: "BookingState".to_string(),
                key_type: BridgeKeyType::String,
                value: BridgeCliValue {
                    raw: "confirmed".to_string(),
                    value_type: BridgeValueType::Symbol,
                },
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "put-smallint",
                "BookingAmount",
                "100",
                "--symbol"
            ]))
            .unwrap(),
            Command::BridgePut {
                root: DEFAULT_BRIDGE_ROOT.to_string(),
                key: "BookingAmount".to_string(),
                key_type: BridgeKeyType::Symbol,
                value: BridgeCliValue {
                    raw: "100".to_string(),
                    value_type: BridgeValueType::SmallInt,
                },
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "put-bool",
                "BookingApproved",
                "true",
                "--root",
                "DemoRoot"
            ]))
            .unwrap(),
            Command::BridgePut {
                root: "DemoRoot".to_string(),
                key: "BookingApproved".to_string(),
                key_type: BridgeKeyType::String,
                value: BridgeCliValue {
                    raw: "true".to_string(),
                    value_type: BridgeValueType::Bool,
                },
            }
        );
        assert!(parse_command(&args(&[
            "bridge",
            "put-bool",
            "BookingApproved",
            "true",
            "--type",
            "String"
        ]))
        .is_err());
        assert_eq!(
            parse_command(&args(&["bridge", "remove", "BookingDraft", "--symbol"])).unwrap(),
            Command::BridgeRemove {
                root: DEFAULT_BRIDGE_ROOT.to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::Symbol,
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "sample-config", "CustomerDraft"])).unwrap(),
            Command::BridgeSampleConfig {
                mapped_name: "CustomerDraft".to_string()
            }
        );
    }

    #[test]
    fn parses_codegen_commands() {
        assert_eq!(
            parse_command(&args(&["codegen", "init"])).unwrap(),
            Command::CodegenInit {
                config: PathBuf::from(DEFAULT_CONFIG_PATH)
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "preview", "demo.codegen"])).unwrap(),
            Command::CodegenPreview {
                config: PathBuf::from("demo.codegen")
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "preview-profile", "default"])).unwrap(),
            Command::CodegenPreviewProfile {
                name: "default".to_string(),
                profiles: PathBuf::from(profiles::DEFAULT_PROFILE_PATH)
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "check"])).unwrap(),
            Command::CodegenCheck {
                config: PathBuf::from(DEFAULT_CONFIG_PATH)
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "explain", "demo.codegen"])).unwrap(),
            Command::CodegenExplain {
                config: PathBuf::from("demo.codegen"),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "explain", "--json", "demo.codegen"])).unwrap(),
            Command::CodegenExplain {
                config: PathBuf::from("demo.codegen"),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "explain-profile", "--json", "default"])).unwrap(),
            Command::CodegenExplainProfile {
                name: "default".to_string(),
                profiles: PathBuf::from(profiles::DEFAULT_PROFILE_PATH),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "codegen",
                "explain-profile",
                "default",
                "examples/codegen/gemstone-rs.codegen-profiles.json"
            ]))
            .unwrap(),
            Command::CodegenExplainProfile {
                name: "default".to_string(),
                profiles: PathBuf::from("examples/codegen/gemstone-rs.codegen-profiles.json"),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "codegen",
                "check-profile",
                "default",
                "examples/codegen/gemstone-rs.codegen-profiles.json"
            ]))
            .unwrap(),
            Command::CodegenCheckProfile {
                name: "default".to_string(),
                profiles: PathBuf::from("examples/codegen/gemstone-rs.codegen-profiles.json")
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "diff", "demo.codegen"])).unwrap(),
            Command::CodegenDiff {
                config: PathBuf::from("demo.codegen")
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "codegen",
                "diff-profile",
                "default",
                "profiles.json"
            ]))
            .unwrap(),
            Command::CodegenDiffProfile {
                name: "default".to_string(),
                profiles: PathBuf::from("profiles.json")
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "generate", "demo.codegen"])).unwrap(),
            Command::CodegenGenerate {
                config: PathBuf::from("demo.codegen")
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "generate-profile", "default"])).unwrap(),
            Command::CodegenGenerateProfile {
                name: "default".to_string(),
                profiles: PathBuf::from(profiles::DEFAULT_PROFILE_PATH)
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "discover", "demo.codegen", "Object"])).unwrap(),
            Command::CodegenDiscover {
                config: PathBuf::from("demo.codegen"),
                classes: vec!["Object".to_string()]
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "codegen",
                "discover-mapping",
                "mapping.codegen",
                "BookingDraft",
                "UserGlobals:Booking"
            ]))
            .unwrap(),
            Command::CodegenDiscoverMapping {
                config: PathBuf::from("mapping.codegen"),
                mapped_name: "BookingDraft".to_string(),
                class_name: "UserGlobals:Booking".to_string()
            }
        );
    }

    #[test]
    fn parses_profile_commands() {
        assert_eq!(
            parse_command(&args(&["profile", "sample"])).unwrap(),
            Command::ProfileSample
        );
        assert_eq!(
            parse_command(&args(&["profile", "init"])).unwrap(),
            Command::ProfileInit {
                path: PathBuf::from(profiles::DEFAULT_PROFILE_PATH)
            }
        );
        assert_eq!(
            parse_command(&args(&["profile", "init", "profiles.json"])).unwrap(),
            Command::ProfileInit {
                path: PathBuf::from("profiles.json")
            }
        );
        assert_eq!(
            parse_command(&args(&["profile", "validate"])).unwrap(),
            Command::ProfileValidate {
                path: PathBuf::from(profiles::DEFAULT_PROFILE_PATH),
                format: OutputFormat::Human
            }
        );
        assert_eq!(
            parse_command(&args(&["profile", "validate", "--json"])).unwrap(),
            Command::ProfileValidate {
                path: PathBuf::from(profiles::DEFAULT_PROFILE_PATH),
                format: OutputFormat::Json
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "profile",
                "validate",
                "examples/codegen/gemstone-rs.codegen-profiles.json"
            ]))
            .unwrap(),
            Command::ProfileValidate {
                path: PathBuf::from("examples/codegen/gemstone-rs.codegen-profiles.json"),
                format: OutputFormat::Human
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "profile",
                "validate",
                "examples/codegen/gemstone-rs.codegen-profiles.json",
                "--json"
            ]))
            .unwrap(),
            Command::ProfileValidate {
                path: PathBuf::from("examples/codegen/gemstone-rs.codegen-profiles.json"),
                format: OutputFormat::Json
            }
        );
        assert_eq!(
            parse_command(&args(&["profile", "list"])).unwrap(),
            Command::ProfileList {
                path: PathBuf::from(profiles::DEFAULT_PROFILE_PATH),
                format: OutputFormat::Human
            }
        );
        assert_eq!(
            parse_command(&args(&["profile", "list", "--json", "profiles.json"])).unwrap(),
            Command::ProfileList {
                path: PathBuf::from("profiles.json"),
                format: OutputFormat::Json
            }
        );
        assert_eq!(
            parse_command(&args(&["profile", "show", "default"])).unwrap(),
            Command::ProfileShow {
                path: PathBuf::from(profiles::DEFAULT_PROFILE_PATH),
                name: "default".to_string(),
                format: OutputFormat::Human
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "profile",
                "show",
                "default",
                "examples/codegen/gemstone-rs.codegen-profiles.json",
                "--json"
            ]))
            .unwrap(),
            Command::ProfileShow {
                path: PathBuf::from("examples/codegen/gemstone-rs.codegen-profiles.json"),
                name: "default".to_string(),
                format: OutputFormat::Json
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "profile",
                "resolve",
                "default",
                "examples/codegen/gemstone-rs.codegen-profiles.json",
                "--json"
            ]))
            .unwrap(),
            Command::ProfileResolve {
                path: PathBuf::from("examples/codegen/gemstone-rs.codegen-profiles.json"),
                name: "default".to_string(),
                format: OutputFormat::Json
            }
        );
        assert_eq!(
            parse_command(&args(&["profile", "check", "--json", "profiles.json"])).unwrap(),
            Command::ProfileCheck {
                path: PathBuf::from("profiles.json"),
                format: OutputFormat::Json
            }
        );
    }

    #[test]
    fn profile_check_json_includes_summary_counts() {
        let entries = vec![
            profiles::ProfileCheckEntry {
                name: "fresh".to_string(),
                config: Some(PathBuf::from("fresh.codegen")),
                output: Some(PathBuf::from("fresh.rs")),
                exists: true,
                up_to_date: true,
                error: None,
            },
            profiles::ProfileCheckEntry {
                name: "stale".to_string(),
                config: Some(PathBuf::from("stale.codegen")),
                output: Some(PathBuf::from("stale.rs")),
                exists: true,
                up_to_date: false,
                error: None,
            },
            profiles::ProfileCheckEntry {
                name: "broken".to_string(),
                config: None,
                output: None,
                exists: false,
                up_to_date: false,
                error: Some("missing config".to_string()),
            },
        ];

        let json = profiles::ProfileCheckReport {
            path: PathBuf::from("profiles.json"),
            entries,
        }
        .to_json();

        assert!(json.contains(r#""success":true"#));
        assert!(json.contains(r#""ok":false"#));
        assert!(json.contains(r#""profileCount":3"#));
        assert!(json.contains(r#""okCount":1"#));
        assert!(json.contains(r#""staleCount":1"#));
        assert!(json.contains(r#""errorCount":1"#));
        assert!(json.contains(r#""name":"broken""#));
        assert!(json.contains(r#""error":"missing config""#));
    }

    #[test]
    fn profile_check_counts_separate_stale_and_errors() {
        let entries = vec![
            profiles::ProfileCheckEntry {
                name: "fresh".to_string(),
                config: None,
                output: None,
                exists: true,
                up_to_date: true,
                error: None,
            },
            profiles::ProfileCheckEntry {
                name: "stale".to_string(),
                config: None,
                output: None,
                exists: true,
                up_to_date: false,
                error: None,
            },
            profiles::ProfileCheckEntry {
                name: "broken".to_string(),
                config: None,
                output: None,
                exists: false,
                up_to_date: false,
                error: Some("missing config".to_string()),
            },
        ];

        let report = profiles::ProfileCheckReport {
            path: PathBuf::from("profiles.json"),
            entries,
        };
        assert_eq!(
            report.counts(),
            profiles::ProfileCheckCounts {
                profile_count: 3,
                ok_count: 1,
                stale_count: 1,
                error_count: 1,
            }
        );
    }
}
