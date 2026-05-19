use gemstone_rs::{
    browser::{Browser, ALL_PROTOCOLS},
    codegen::{self, DEFAULT_CONFIG_PATH},
    gci_library_path, gci_library_resolution, profiles, py_native, BridgeKeyType, BridgeValue,
    Config, Error as GemStoneError, Oop, Session, Value, DEFAULT_BRIDGE_ROOT,
    DEFAULT_BRIDGE_VALUE_DEPTH,
};
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};

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
        Command::Hello { format } => {
            print_hello(format);
            Ok(())
        }
        Command::PyNativeCapabilities { format } => {
            print_py_native_capabilities(format);
            Ok(())
        }
        Command::PyNativeCheck { path, format } => run_py_native_check(&path, format),
        Command::PyNativeSamples { format } => {
            print_py_native_samples(format);
            Ok(())
        }
        Command::PyNativeSamplesCheck { path, format } => {
            run_py_native_samples_check(&path, format)
        }
        Command::PyNativeSmokeCheck { path, format } => run_py_native_smoke_check(&path, format),
        Command::PyNativeSmoke { dry_run, format } => run_py_native_smoke(dry_run, format),
        Command::PyNativeMigration { format } => {
            print_py_native_migration(format);
            Ok(())
        }
        Command::PyNativeCompatibility { format } => {
            print_py_native_compatibility(format);
            Ok(())
        }
        Command::PyNativeCompatibilityCheck { path, format } => {
            run_py_native_compatibility_check(&path, format)
        }
        Command::PyNativeConformance { format } => {
            print_py_native_conformance(format);
            Ok(())
        }
        Command::PyNativeConformanceCheck { path, format } => {
            run_py_native_conformance_check(&path, format)
        }
        Command::PyNativeHandoff { format } => {
            print_py_native_handoff(format);
            Ok(())
        }
        Command::PyNativeHandoffCheck { path, format } => {
            run_py_native_handoff_check(&path, format)
        }
        Command::PyNativePublishReceipt { format } => {
            print_py_native_publish_receipt(format);
            Ok(())
        }
        Command::PyNativePublishReceiptCheck { path, format } => {
            run_py_native_publish_receipt_check(&path, format)
        }
        Command::PyNativeCheckAll { root, format } => {
            run_py_native_check_all(root.as_deref(), format)
        }
        Command::CompareGemstonePy { view, format } => {
            print_gemstone_py_comparison(view, format);
            Ok(())
        }
        Command::CompareGemstoneJs { view, format } => {
            print_gemstone_js_comparison(view, format);
            Ok(())
        }
        Command::CompareAll { view, format } => {
            print_all_comparisons(view, format);
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
        Command::ExamplesMap { format } => {
            print_feature_map(format);
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
        Command::ExamplesRun {
            name,
            dry_run,
            extra_args,
        } => {
            let example = find_example(&name).ok_or_else(|| {
                CliError::Example(format!(
                    "example {name} not found; run `gemstone-rs examples list`"
                ))
            })?;
            run_example(example, dry_run, &extra_args)
        }
        Command::ExamplesScaffold { name, path, force } => {
            let template = find_scaffold_template(&name).ok_or_else(|| {
                CliError::Example(format!(
                    "scaffold template {name} not found; available templates: {}",
                    scaffold_template_names()
                ))
            })?;
            let target = path.unwrap_or_else(|| PathBuf::from(template.package_name));
            scaffold_example_project(template, &target, force)?;
            println!("wrote {}", target.display());
            println!("next:");
            println!("  cd {}", target.display());
            println!("  cargo run");
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
        Command::BridgeValue {
            root,
            key,
            key_type,
            depth,
        } => {
            let mut session = login()?;
            let value = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.get_bridge_value_with_depth(&key, key_type, depth)?
            };
            print_bridge_value(&value, 0);
            Ok(())
        }
        Command::BridgeShape {
            root,
            key,
            key_type,
            depth,
        } => {
            let mut session = login()?;
            let value = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.get_bridge_value_with_depth(&key, key_type, depth)?
            };
            print_bridge_shape_report(&value);
            Ok(())
        }
        Command::BridgeMappingPreview {
            root,
            key,
            key_type,
            depth,
            mapped_name,
        } => {
            let mut session = login()?;
            let value = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.get_bridge_value_with_depth(&key, key_type, depth)?
            };
            println!(
                "{}",
                codegen::mapping_config_from_bridge_value(&mapped_name, &value)
            );
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
        | GemStoneError::WorkerStopped
        | GemStoneError::WorkerPanicked
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

fn print_bridge_value(value: &BridgeValue, indent: usize) {
    let prefix = " ".repeat(indent);
    match value {
        BridgeValue::Nil => println!("{prefix}nil"),
        BridgeValue::Bool(value) => println!("{prefix}Bool({value})"),
        BridgeValue::SmallInt(value) => println!("{prefix}SmallInt({value})"),
        BridgeValue::String(value) => println!("{prefix}String({value:?})"),
        BridgeValue::Symbol(value) => println!("{prefix}Symbol({value:?})"),
        BridgeValue::Oop(oop) => println!("{prefix}Oop({})", oop.raw()),
        BridgeValue::Dictionary(entries) => {
            println!("{prefix}Dictionary");
            for (key, value) in entries {
                println!("{prefix}  {key}:");
                print_bridge_value(value, indent + 4);
            }
        }
        BridgeValue::KeyedDictionary(entries) => {
            println!("{prefix}KeyedDictionary");
            for (key, value) in entries {
                println!("{prefix}  {} ({}):", key.name, key.key_type.config_name());
                print_bridge_value(value, indent + 4);
            }
        }
        BridgeValue::Array(values) => {
            println!("{prefix}Array");
            for (index, value) in values.iter().enumerate() {
                println!("{prefix}  [{}]:", index + 1);
                print_bridge_value(value, indent + 4);
            }
        }
    }
}

fn print_bridge_shape_report(value: &BridgeValue) {
    let report = value.shape_report();
    println!("BridgeValue shape");
    println!("  total_nodes: {}", report.total_nodes);
    println!("  scalar_nodes: {}", report.scalar_nodes);
    println!("  dictionary_nodes: {}", report.dictionary_nodes);
    println!("  array_nodes: {}", report.array_nodes);
    println!("  opaque_oops: {}", report.opaque_oops);
    println!("  unique_oops: {}", report.unique_oops);
    println!("  repeated_oop_refs: {}", report.repeated_oop_refs);
    println!("  nil_nodes: {}", report.nil_nodes);
    println!("  max_depth: {}", report.max_depth);
    println!("relationships:");
    for node in &report.nodes {
        let key_type = node
            .key_type
            .map(|key_type| key_type.config_name())
            .unwrap_or("-");
        let note = node
            .note
            .as_ref()
            .map(|note| format!("\tnote={note}"))
            .unwrap_or_default();
        let oop = node
            .oop
            .map(|oop| format!("\toop={}", oop.raw()))
            .unwrap_or_default();
        let identity = node
            .identity_id
            .map(|identity_id| format!("\tidentity_id={identity_id}"))
            .unwrap_or_default();
        let repeated = if node.repeated_identity {
            "\trepeated_identity=true"
        } else {
            ""
        };
        println!(
            "  {}\tkind={}\tkey_type={}\tdepth={}\tchildren={}{}{}{}{}",
            node.path,
            node.kind,
            key_type,
            node.depth,
            node.child_count,
            oop,
            identity,
            repeated,
            note
        );
    }
    if !report.identity_groups.is_empty() {
        println!("repeated identities:");
        for group in &report.identity_groups {
            println!(
                "  identity_id={}\toop={}\tpaths={}",
                group.identity_id,
                group.oop.raw(),
                group.paths.join(", ")
            );
        }
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FeatureInfo {
    stream: &'static str,
    title: &'static str,
    crates: &'static str,
    examples: &'static str,
    docs: &'static str,
    gemstone_py_reference: &'static str,
    status: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ComparisonInfo {
    topic: &'static str,
    gemstone_py: &'static str,
    gemstone_rs: &'static str,
    recommendation: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct JsComparisonInfo {
    topic: &'static str,
    gemstone_py: &'static str,
    gemstone_js: &'static str,
    recommendation: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GapInfo {
    priority: &'static str,
    area: &'static str,
    gemstone_py_strength: &'static str,
    gemstone_rs_gap: &'static str,
    next_action: &'static str,
    verify_with: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BatchInfo {
    number: u8,
    focus: &'static str,
    hours_min: u8,
    hours_max: u8,
    outcome: &'static str,
    verify_with: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScorecardInfo {
    title: &'static str,
    comparison: &'static str,
    answer: &'static str,
    gemstone_py_use_when: &'static [&'static str],
    project_use_when: &'static [&'static str],
    gemstone_py_strengths: &'static [&'static str],
    project_strengths: &'static [&'static str],
    batches: &'static [BatchInfo],
    gaps: &'static [GapInfo],
    project_label: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParityInfo {
    area: &'static str,
    gemstone_py_score: u8,
    project_score: u8,
    leader: &'static str,
    status: &'static str,
    next_action: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScaffoldTemplate {
    name: &'static str,
    package_name: &'static str,
    title: &'static str,
    description: &'static str,
    main_rs: &'static str,
    extra_dependencies: &'static str,
    extra_files: &'static [ScaffoldFile],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScaffoldFile {
    path: &'static str,
    source: &'static str,
}

const GEMSTONE_RS_VERSION_TOKEN: &str = "{gemstone_rs_version}";

const EXAMPLES: &[ExampleInfo] = &[
    ExampleInfo {
        name: "hello",
        title: "Hello",
        command: "gemstone-rs hello",
        category: "cli",
        requires_live: false,
        description: "No-GemStone CLI sanity check, matching gemstone-examples hello.",
    },
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
        name: "session_worker",
        title: "Session worker",
        command: "cargo run -p gemstone-rs --example session_worker",
        category: "web",
        requires_live: true,
        description: "Dedicated-thread SessionWorker for web services and async runtimes.",
    },
    ExampleInfo {
        name: "session_worker_pool",
        title: "Session worker pool",
        command: "cargo run -p gemstone-rs --example session_worker_pool",
        category: "web",
        requires_live: true,
        description: "Bounded round-robin pool of dedicated GemStone session workers.",
    },
    ExampleInfo {
        name: "async_worker",
        title: "Async worker facade",
        command: "cargo run -p gemstone-rs --example async_worker",
        category: "web",
        requires_live: true,
        description: "Awaitable SessionWorkerPool calls for async runtimes without moving Session across threads.",
    },
    ExampleInfo {
        name: "python_native_adapter",
        title: "Python native adapter",
        command: "cargo run -p gemstone-rs --example python_native_adapter",
        category: "native",
        requires_live: true,
        description: "Exercise the dependency-free py_native contract used by the gemstone-py-native PyO3 bridge.",
    },
    ExampleInfo {
        name: "py_native_capabilities",
        title: "py-native capabilities",
        command: "gemstone-rs py-native capabilities",
        category: "native",
        requires_live: false,
        description: "Print the Rust adapter contract that gemstone-py-native should expose.",
    },
    ExampleInfo {
        name: "py_native_contract_fixture",
        title: "py-native contract fixture",
        command: "gemstone-rs py-native check examples/py-native/gemstone-rs.py-native.json",
        category: "native",
        requires_live: false,
        description: "Validate the checked-in py-native capabilities fixture against the Rust core.",
    },
    ExampleInfo {
        name: "py_native_samples_fixture",
        title: "py-native samples fixture",
        command: "gemstone-rs py-native check-samples examples/py-native/gemstone-rs.py-native-samples.json",
        category: "native",
        requires_live: false,
        description: "Validate checked-in py-native value and error samples for wrapper CI.",
    },
    ExampleInfo {
        name: "py_native_smoke_fixture",
        title: "py-native smoke fixture",
        command: "gemstone-rs py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json",
        category: "native",
        requires_live: false,
        description: "Validate the checked-in dry-run py-native smoke fixture against the Rust core.",
    },
    ExampleInfo {
        name: "py_native_migration_plan",
        title: "py-native migration plan",
        command: "gemstone-rs py-native migration",
        category: "native",
        requires_live: false,
        description: "Print the shared-core migration checklist for wiring gemstone-py-native to gemstone-rs.",
    },
    ExampleInfo {
        name: "py_native_compatibility_fixture",
        title: "py-native compatibility fixture",
        command: "gemstone-rs py-native check-compat examples/py-native/gemstone-rs.py-native-compat.json",
        category: "native",
        requires_live: false,
        description: "Validate the checked-in Python compatibility shim contract for gemstone-py-native.",
    },
    ExampleInfo {
        name: "py_native_conformance_fixture",
        title: "py-native conformance fixture",
        command: "gemstone-rs py-native check-conformance examples/py-native/gemstone-rs.py-native-conformance.json",
        category: "native",
        requires_live: false,
        description: "Validate the checked-in wrapper conformance contract for gemstone-py-native integration.",
    },
    ExampleInfo {
        name: "py_native_handoff_bundle",
        title: "py-native handoff bundle",
        command: "gemstone-rs py-native check-handoff examples/py-native/gemstone-rs.py-native-handoff.json",
        category: "native",
        requires_live: false,
        description: "Validate the checked-in downstream gemstone-py-native handoff manifest.",
    },
    ExampleInfo {
        name: "py_native_publish_receipt",
        title: "py-native publish receipt",
        command: "gemstone-rs py-native check-publish-receipt examples/py-native/gemstone-rs.py-native-publish-receipt.json",
        category: "native",
        requires_live: false,
        description: "Validate the checked-in TestPyPI/PyPI publish receipt for the Rust-backed gemstone-py-native wheels.",
    },
    ExampleInfo {
        name: "py_native_shared_core_gate",
        title: "py-native shared-core gate",
        command: "gemstone-rs py-native check-all",
        category: "native",
        requires_live: false,
        description: "Validate every checked-in py-native fixture used by downstream gemstone-py-native integration.",
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
        name: "bridge_value_inspection",
        title: "BridgeValue inspection",
        command: "cargo run -p gemstone-rs --example bridge_value_inspection",
        category: "mapping",
        requires_live: true,
        description: "Read nested BridgeRoot dictionaries and arrays back as dynamic BridgeValue trees with shape reports.",
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
    ExampleInfo {
        name: "http_service",
        title: "HTTP service",
        command: "cargo run -p gemstone-rs --example http_service",
        category: "web",
        requires_live: true,
        description: "Standard-library HTTP service with /, /health/local, and /health/gemstone routes.",
    },
    ExampleInfo {
        name: "axum_service",
        title: "Axum service",
        command: "cargo run --manifest-path examples/axum-service/Cargo.toml",
        category: "web",
        requires_live: true,
        description: "Checked Axum service with /, /health/local, and /health/gemstone routes.",
    },
    ExampleInfo {
        name: "actix_service",
        title: "Actix service",
        command: "cargo run --manifest-path examples/actix-service/Cargo.toml",
        category: "web",
        requires_live: true,
        description: "Checked Actix Web service with /, /health/local, and /health/gemstone routes.",
    },
];

const FEATURE_MAP: &[FeatureInfo] = &[
    FeatureInfo {
        stream: "1",
        title: "Runtime and GCI loading",
        crates: "gemstone-gci, gemstone-rs::Config",
        examples: "hello, hello_gemstone, quickstart",
        docs: "docs/setup-guide.md, docs/performance-safety.md",
        gemstone_py_reference: "gemstone_py.native, native backend checks",
        status: "Rust-native core; Python still has broader packaged backend docs",
    },
    FeatureInfo {
        stream: "2",
        title: "Safe sessions and transactions",
        crates: "gemstone-rs::Session, SessionWorker, SessionWorkerPool, TransactionGuard",
        examples: "quickstart, transactions, session_worker, session_worker_pool, live_smoke_cookbook",
        docs: "docs/user-manual.md, docs/cookbook.md",
        gemstone_py_reference: "GemStoneSession, SessionFacade, transaction policies",
        status: "Core parity for sync eval/perform/commit/abort plus dedicated-thread workers and a bounded worker pool",
    },
    FeatureInfo {
        stream: "3",
        title: "Browser and inspection",
        crates: "gemstone-rs::browser, gemstone-rs-cli browse, gemstone-rs-explorer",
        examples: "browser, tooling/cli-browser-walkthrough.md",
        docs: "docs/user-manual.md, docs/explorer.md",
        gemstone_py_reference: "gemstone_py.inspection, python-gemstone-database-explorer",
        status: "API parity growing; Python explorer is still more mature",
    },
    FeatureInfo {
        stream: "4",
        title: "OOP and value handling",
        crates: "Oop, Value, export-set helpers",
        examples: "oop_values",
        docs: "docs/user-manual.md, docs/performance-safety.md",
        gemstone_py_reference: "managed OOP handles and typed access examples",
        status: "Explicit Rust ownership model; Python is easier for casual scripting",
    },
    FeatureInfo {
        stream: "5",
        title: "BridgeRoot object mapping",
        crates: "gemstone-rs::bridge, gemstone-rs-macros",
        examples: "bridge_root_mapping, derive_mapping, bridge_value_inspection, generated_mapping_app",
        docs: "docs/object-mapping.md, docs/cookbook.md",
        gemstone_py_reference: "SmalltalkBridge, PersistentRoot, facade examples",
        status: "Rust has typed mapping, derive, nested dynamic BridgeValue read-back, repeated-OOP identity groups, and path-aware diagnostics; a fully transparent object model remains future work",
    },
    FeatureInfo {
        stream: "6",
        title: "Typed codegen",
        crates: "gemstone-rs::codegen, gemstone-rs-cli codegen/profile",
        examples: "codegen_preview, codegen_workflow, codegen_discover, profile_codegen_workflow, generated_wrapper_app",
        docs: "docs/codegen.md, docs/profile-schema.md",
        gemstone_py_reference: "gemstone_py.codegen, typed_access/codegen_demo",
        status: "Preview/diff/check/generate, installed profile workflow parity, generated metadata tests, and live discovery of source-header args plus protocol/source docs; deeper live type discovery remains planned",
    },
    FeatureInfo {
        stream: "7",
        title: "Explorer workflow",
        crates: "gemstone-rs-explorer",
        examples: "tooling/explorer.md",
        docs: "docs/explorer.md, docs/screenshots.md",
        gemstone_py_reference: "python-gemstone-database-explorer",
        status: "Useful local UI/API with profile status, codegen diff/explain, editable generated output, nested BridgeValue rendering, comparison workflows, and committed visual assets; Python explorer remains the richer product reference",
    },
    FeatureInfo {
        stream: "8",
        title: "VS Code workbench",
        crates: "vscode-gemstone-rs-workbench",
        examples: "tooling/vscode-workbench.md",
        docs: "docs/vscode-workbench.md",
        gemstone_py_reference: "gemstone-py Workbench",
        status: "Command workflow and embedded webview now render setup checks, profile status, codegen summaries/diffs/editable output, BridgeRoot keys/nested values, comparison status, and Marketplace/GitHub visuals",
    },
    FeatureInfo {
        stream: "9",
        title: "Rust web services",
        crates: "gemstone-rs::web, gemstone-rs-axum, gemstone-rs-actix, SessionWorkerPool, checked Axum/Actix services, and installed scaffolds",
        examples: "session_worker, session_worker_pool, async_worker, http_service, examples/axum-service, examples/actix-service, examples scaffold session_worker_pool/axum_service/actix_service",
        docs: "docs/examples-guide.md, docs/cookbook.md",
        gemstone_py_reference: "FastAPI, Litestar, Django examples",
        status: "Shared JSON health helpers, std HTTP, graceful health-pool startup, route/request/lifecycle/duration headers, production-style service/cache/security middleware headers, packaged Axum/Actix adapters, checked services, installed scaffolds, local/live route smoke, and a dependency-free async worker facade exist; broader framework coverage can grow incrementally",
    },
    FeatureInfo {
        stream: "10",
        title: "Release and verification",
        crates: "scripts, Makefile, GitHub Actions",
        examples: "release verification commands",
        docs: "docs/release-checklist.md",
        gemstone_py_reference: "PyPI/TestPyPI/native wheel/VSIX release tooling",
        status: "Crates/VSIX verification path exists; Python release lane is more complete",
    },
    FeatureInfo {
        stream: "11",
        title: "Shared native core",
        crates: "gemstone-gci, gemstone-rs::py_native, gemstone-py-native wrapper",
        examples: "python_native_adapter, py_native_pyo3_adapter scaffold, downstream gemstone-py-native bridge",
        docs: "docs/shared-core-integration.md",
        gemstone_py_reference: "gemstone-py-native",
        status: "Rust-side PyO3 adapter contract, compatibility shim map, conformance fixture, handoff manifest, publish receipt, shared-core gate, starter scaffold, and downstream gemstone-py-native RustCoreSession bridge are wired; local live-stone smoke and published-wheel verification passed",
    },
];

const GEMSTONE_PY_COMPARISON: &[ComparisonInfo] = &[
    ComparisonInfo {
        topic: "Best fit",
        gemstone_py: "Python apps, scripts, notebooks, FastAPI/Litestar/Django examples, and Python-first teams",
        gemstone_rs: "Rust services, CLIs, workers, native tooling, and no-Python GemStone access",
        recommendation: "Choose the language/runtime your application already lives in",
    },
    ComparisonInfo {
        topic: "Install path",
        gemstone_py: "python -m pip install gemstone-py; optional native acceleration with gemstone-py[fast]",
        gemstone_rs: "cargo add gemstone-rs; cargo install gemstone-rs-cli gemstone-rs-explorer",
        recommendation: "Use pip for Python deliverables and Cargo for Rust deliverables",
    },
    ComparisonInfo {
        topic: "Examples",
        gemstone_py: "gemstone-examples list, plan3-map, hello, quickstart, fastapi, litestar",
        gemstone_rs: "gemstone-rs hello; examples list/map/show/run/scaffold; Cargo examples from source checkout",
        recommendation: "gemstone-py is ahead for breadth; gemstone-rs now has comparable discovery and standalone project scaffolds",
    },
    ComparisonInfo {
        topic: "Web frameworks",
        gemstone_py: "FastAPI, Litestar, and Django examples are first-class",
        gemstone_rs: "Shared gemstone_rs::web health helpers, standard-library HTTP, SessionWorkerPool, packaged Axum/Actix adapters, checked examples, request-trace headers, lifecycle-duration headers, and production-style service/cache/security middleware headers exist",
        recommendation: "Use gemstone-py today for the broadest framework coverage; use gemstone-rs when Rust services need direct GemStone health routes and bounded session-worker boundaries",
    },
    ComparisonInfo {
        topic: "Codegen and mapping",
        gemstone_py: "Python codegen and typed access demos",
        gemstone_rs: "Rust wrapper generation, typed argument conversion, typed return helpers, BridgeRoot mapping, derive macro, preview/diff/check/generate",
        recommendation: "Use gemstone-rs when compile-time Rust wrapper checks matter",
    },
    ComparisonInfo {
        topic: "Explorer and VS Code",
        gemstone_py: "More mature database explorer and workbench product flow",
        gemstone_rs: "Local explorer, command workbench, embedded webview with structured setup/profile/codegen/diff/BridgeRoot panels, repeated-OOP identity groups, and CLI-backed codegen workflow",
        recommendation: "Python explorer remains the product reference; Rust explorer is the backend proving ground",
    },
    ComparisonInfo {
        topic: "Native bridge direction",
        gemstone_py: "Python API now has an additive gemstone-py-native RustCoreSession bridge over the Rust core while preserving existing Python return behavior",
        gemstone_rs: "Owns the shared GCI core plus dependency-free py_native contract, compatibility, conformance, handoff, and publish-receipt reports",
        recommendation: "Keep native wheel/sdist checks, live smoke, and post-publish install verification green",
    },
];

const GEMSTONE_PY_USE_WHEN: &[&str] = &[
    "You are building Python applications, notebooks, scripts, or web services.",
    "You want the broadest current examples across FastAPI, Litestar, Django, async, docs, and packaging.",
    "You need the mature Python database explorer and PyPI/TestPyPI release path today.",
];

const GEMSTONE_RS_USE_WHEN: &[&str] = &[
    "You are building Rust services, CLIs, workers, or local tooling that should talk to GemStone without Python.",
    "You want compile-time checked generated wrappers, typed return helpers, and explicit OOP/value handling.",
    "You want the Rust GCI core that now backs the additive gemstone-py-native shared-core bridge.",
];

const GEMSTONE_PY_STRENGTHS: &[&str] = &[
    "Mature Python package and optional native acceleration install path.",
    "Broader web framework examples and async workflow coverage.",
    "Richer database explorer and public docs/release surfaces.",
];

const GEMSTONE_RS_STRENGTHS: &[&str] = &[
    "Direct Rust API over GemStone/S with no Python process required.",
    "Typed codegen, BridgeRoot object mapping, derive support, and compile-time wrapper checks.",
    "CLI, explorer, and VS Code workflows that exercise the Rust core directly.",
];

const GEMSTONE_RS_PARITY: &[ParityInfo] = &[
    ParityInfo {
        area: "Core sessions and GCI",
        gemstone_py_score: 4,
        project_score: 4,
        leader: "tie",
        status: "Both projects cover login, eval, perform, transactions, and live smoke tests; Rust is stricter about ownership and thread boundaries.",
        next_action: "Keep live smoke tests aligned and expand multi-version GemStone coverage.",
    },
    ParityInfo {
        area: "Web frameworks",
        gemstone_py_score: 5,
        project_score: 4,
        leader: "gemstone-py",
        status: "gemstone-py has broader FastAPI, Litestar, and Django examples; gemstone-rs has shared health helpers, Axum/Actix adapters, request-trace headers, lifecycle-duration headers, production-style middleware headers, and local/live route smoke tests.",
        next_action: "Add broader framework adapter examples and richer real-application middleware patterns.",
    },
    ParityInfo {
        area: "Async and lifetime behavior",
        gemstone_py_score: 4,
        project_score: 4,
        leader: "tie",
        status: "gemstone-py has broader async web examples and FastAPI lifetime coverage; gemstone-rs has dependency-free awaitable SessionWorker/SessionWorkerPool calls while keeping Session on a dedicated thread.",
        next_action: "Add deeper cancellation, shutdown, and lifetime tests around async worker futures.",
    },
    ParityInfo {
        area: "Codegen and object mapping",
        gemstone_py_score: 4,
        project_score: 4,
        leader: "tie",
        status: "gemstone-rs has compile-time wrapper checks, generated metadata tests, typed return helpers, BridgeRoot mapping, derive support, nested dynamic BridgeValue read-back, repeated-OOP identity groups, and path-aware diagnostics; gemstone-py remains the broader reference.",
        next_action: "Finish deeper live type discovery, typed helper polish, and transparent object-model experiments.",
    },
    ParityInfo {
        area: "Explorer and VS Code",
        gemstone_py_score: 5,
        project_score: 4,
        leader: "gemstone-py",
        status: "The Python explorer is still the richer product reference; gemstone-rs now has a useful explorer, VS Code commands, an embedded webview, editable generated output, BridgeRoot key pickers, nested BridgeValue rendering, shape reports, repeated-OOP identity groups, mapping previews, and committed Marketplace/GitHub visuals.",
        next_action: "Keep Marketplace/GitHub screenshots current and use explorer/API smoke tests after UI changes; no active parity batch remains.",
    },
    ParityInfo {
        area: "Release and install lane",
        gemstone_py_score: 5,
        project_score: 5,
        leader: "tie",
        status: "gemstone-rs has crates, VSIX, checksums, PDFs, release wrappers, post-release verification, and a shared live smoke lane.",
        next_action: "Use the release and post-release workflows regularly so published artifacts and live GemStone smoke stay exercised.",
    },
    ParityInfo {
        area: "Shared native core",
        gemstone_py_score: 3,
        project_score: 5,
        leader: "gemstone-rs",
        status: "gemstone-rs owns the clean Rust GCI/session core, exposes the py_native contract, and gemstone-py-native now has an additive RustCoreSession bridge plus verified TestPyPI/PyPI wheels.",
        next_action: "Keep the shared native-core release lane exercised with native-wheels TestPyPI/PyPI install verification on each publish.",
    },
];

const GEMSTONE_PY_GAPS: &[GapInfo] = &[
    GapInfo {
        priority: "P2",
        area: "Explorer product polish",
        gemstone_py_strength: "python-gemstone-database-explorer is the richer class browser and product reference.",
        gemstone_rs_gap: "gemstone-rs-explorer and the VS Code webview now cover structured setup checks, profile status, live browsing, source previews, codegen summaries/diffs, editable generated output, BridgeRoot key pickers, nested BridgeValue rendering, shape reports, repeated-OOP identity groups, mapping previews, open-file actions, browser fallback prompts, comparison status, and committed Marketplace/GitHub visuals. The remaining gap is product refinement: fresher visuals, workflow feedback, and ongoing editor polish.",
        next_action: "Keep Marketplace/GitHub screenshots current and run explorer/API smoke tests after UI changes; no active parity batch remains.",
        verify_with: "python3 scripts/explorer_endpoint_smoke.py; vscode-gemstone-rs-workbench smoke test",
    },
    GapInfo {
        priority: "P1",
        area: "Installed example experience",
        gemstone_py_strength: "gemstone-examples launches installed examples without needing a source checkout.",
        gemstone_rs_gap: "gemstone-rs now scaffolds quickstart, browser, BridgeRoot mapping, derive mapping, generated wrappers, codegen discovery, profile codegen, standard HTTP, worker-pool, Axum, and Actix projects, but explorer-integrated workflows still need installed templates.",
        next_action: "Expand examples scaffold templates to explorer-integrated projects and richer generated wrapper profile variants.",
        verify_with: "gemstone-rs examples scaffold profile_codegen_workflow /tmp/gemstone-rs-profile-codegen --force",
    },
    GapInfo {
        priority: "P2",
        area: "Web framework adapters",
        gemstone_py_strength: "FastAPI, Litestar, and Django examples are first-class and documented.",
        gemstone_rs_gap: "gemstone-rs now has shared JSON health helpers, standard-library HTTP, graceful health-pool startup, route/request/lifecycle/duration headers, production-style service/cache/security middleware headers, packaged Axum/Actix adapters, checked services, async health handlers, local/live route smoke coverage, and installed Axum/Actix scaffolds. It can still grow broader framework coverage beyond Axum/Actix.",
        next_action: "Add more framework adapter examples only when a real Rust service needs them.",
        verify_with: "cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes",
    },
    GapInfo {
        priority: "P2",
        area: "Async lifetime depth",
        gemstone_py_strength: "gemstone-py has async examples, FastAPI integration, and lifetime/GC tests around async behavior.",
        gemstone_rs_gap: "gemstone-rs now has awaitable SessionWorkerPool calls and async Axum/Actix health handlers, but it still needs deeper cancellation, shutdown, and lifetime tests.",
        next_action: "Add targeted cancellation/shutdown tests for pending worker futures.",
        verify_with: "GS_RUN_LIVE_RUST=1 cargo test -p gemstone-rs live_",
    },
    GapInfo {
        priority: "P2",
        area: "Release lane depth",
        gemstone_py_strength: "gemstone-py has PyPI/TestPyPI, native wheel, VSIX, and post-publish verification lanes.",
        gemstone_rs_gap: "gemstone-rs has the release lane wired, but it should be exercised routinely after each publish.",
        next_action: "Run the Release and Post-Release Verify workflows after publishing, with live smoke enabled when a test stone is available.",
        verify_with: "scripts/publish_verify.sh <version>; scripts/live_smoke.sh",
    },
];

const GEMSTONE_RS_BATCHES: &[BatchInfo] = &[];

const GEMSTONE_JS_COMPARISON: &[JsComparisonInfo] = &[
    JsComparisonInfo {
        topic: "Best fit",
        gemstone_py: "Python applications, scripts, notebooks, data tooling, FastAPI/Litestar/Django services, and Python-first teams",
        gemstone_js: "TypeScript/JavaScript services, CLIs, Node-based tooling, web adapters, and teams already using npm workflows",
        recommendation: "Choose gemstone-py for Python products; choose gemstone-js for async TypeScript applications that need direct GemStone access",
    },
    JsComparisonInfo {
        topic: "Install path",
        gemstone_py: "PyPI package with optional native acceleration through gemstone-py[fast]",
        gemstone_js: "npm package shape with optional @gemstone-js/native; local package is 0.1.0-alpha.0 and requires Node >=24",
        recommendation: "Use gemstone-py for the more stable published path today; use gemstone-js when npm distribution is the product boundary",
    },
    JsComparisonInfo {
        topic: "API style",
        gemstone_py: "Sync and async Python APIs, explicit sessions, managed OOP handles, and Pythonic converters",
        gemstone_js: "Async-first Session API, AsyncDisposable support, typed OOP helpers, managed handles, request scopes, pools, and converter registry",
        recommendation: "gemstone-js is naturally stronger for async JavaScript services; gemstone-py is easier for Python scripting",
    },
    JsComparisonInfo {
        topic: "Web frameworks",
        gemstone_py: "FastAPI, Litestar, and Django examples are first-class",
        gemstone_js: "Express, Fastify, Fetch API, and Hono adapters delegate request teardown through a shared scope layer",
        recommendation: "Both are credible for web services; pick the framework ecosystem your application already uses",
    },
    JsComparisonInfo {
        topic: "Persistence helpers",
        gemstone_py: "Persistent roots, collection helpers, migrations, object log, reduced-conflict helpers, and typed access examples",
        gemstone_js: "PersistentRoot, GsDict, OrderedCollection, GStore, migrations, ObjectLog, reduced-conflict wrappers, and query helpers",
        recommendation: "gemstone-js has caught up surprisingly well in surface area, but gemstone-py has more live-project maturity",
    },
    JsComparisonInfo {
        topic: "Codegen",
        gemstone_py: "Python codegen and typed-access demos",
        gemstone_js: "Manifest/decorator-driven TypeScript codegen with schemas, arity checks, typed signatures, and generated examples",
        recommendation: "Use gemstone-js when TypeScript types and decorator scanning matter; gemstone-py remains the broader reference implementation",
    },
    JsComparisonInfo {
        topic: "Tooling",
        gemstone_py: "Database explorer, VS Code workbench, examples CLI, native wheel publishing, PyPI/TestPyPI verification",
        gemstone_js: "Doctor, examples CLI, inspect CLI, bootstrap, migrations CLI, benchmarks, checksums, and API-contract checks",
        recommendation: "gemstone-py has the stronger visual tooling; gemstone-js has strong package/CI-oriented developer tooling",
    },
    JsComparisonInfo {
        topic: "Native/runtime layer",
        gemstone_py: "Python native path is already integrated with release tooling",
        gemstone_js: "Node native package, Deno/Bun FFI starters, mock runtime, and library discovery via GS_LIB_PATH/GS_LIB/GEMSTONE",
        recommendation: "gemstone-js needs more live native-platform proof before it matches gemstone-py operational confidence",
    },
];

const GEMSTONE_JS_USE_WHEN: &[&str] = &[
    "You are building Node, TypeScript, or JavaScript services that should keep GemStone access in the JS runtime.",
    "You want async-first APIs, npm packaging, and TypeScript-generated signatures.",
    "You are willing to work with a newer alpha surface while native package confidence matures.",
];

const GEMSTONE_JS_STRENGTHS: &[&str] = &[
    "Async-first API shape that fits Node services naturally.",
    "Manifest/decorator codegen with TypeScript type signatures and schema checks.",
    "npm-oriented doctor, examples, migrations, inspect, benchmarks, and package smoke tooling.",
];

const GEMSTONE_JS_PARITY: &[ParityInfo] = &[
    ParityInfo {
        area: "Core sessions and runtime",
        gemstone_py_score: 4,
        project_score: 3,
        leader: "gemstone-py",
        status: "gemstone-js has an async-first session surface and runtime discovery, but gemstone-py has the more proven package and live-project use.",
        next_action: "Publish and verify clean-install native/runtime packages across supported Node platforms.",
    },
    ParityInfo {
        area: "Web frameworks",
        gemstone_py_score: 5,
        project_score: 3,
        leader: "gemstone-py",
        status: "gemstone-py has first-class FastAPI, Litestar, and Django examples; gemstone-js has Express, Fastify, Fetch API, and Hono adapter direction.",
        next_action: "Add end-to-end installed web examples and live route smoke tests.",
    },
    ParityInfo {
        area: "Async service fit",
        gemstone_py_score: 4,
        project_score: 4,
        leader: "tie",
        status: "gemstone-js fits Node async services naturally; gemstone-py has broader async/lifetime examples today.",
        next_action: "Expand request-scope teardown, pool, transaction, and framework adapter live tests.",
    },
    ParityInfo {
        area: "Persistence helpers",
        gemstone_py_score: 5,
        project_score: 4,
        leader: "gemstone-py",
        status: "gemstone-js has persistent roots, migrations, ObjectLog, query helpers, and reduced-conflict wrappers, but less production mileage.",
        next_action: "Add more clean-install persistence examples and live migration smoke tests.",
    },
    ParityInfo {
        area: "Codegen",
        gemstone_py_score: 4,
        project_score: 4,
        leader: "tie",
        status: "gemstone-js has manifest/decorator codegen with TypeScript signatures; gemstone-py remains the broader reference.",
        next_action: "Connect codegen to visual tooling and live discovery flows.",
    },
    ParityInfo {
        area: "Visual tooling",
        gemstone_py_score: 5,
        project_score: 1,
        leader: "gemstone-py",
        status: "gemstone-py has the mature explorer and VS Code workbench; gemstone-js has CLI tooling but no visual explorer yet.",
        next_action: "Build or reuse explorer/workbench concepts for TypeScript inspection and codegen.",
    },
    ParityInfo {
        area: "Release and native confidence",
        gemstone_py_score: 5,
        project_score: 2,
        leader: "gemstone-py",
        status: "gemstone-js is alpha and needs more live native-platform proof before matching gemstone-py operational confidence.",
        next_action: "Exercise npm package, native package, checksums, live smoke, and post-publish verification in CI.",
    },
];

const GEMSTONE_JS_BATCHES: &[BatchInfo] = &[
    BatchInfo {
        number: 1,
        focus: "Native publish confidence",
        hours_min: 6,
        hours_max: 10,
        outcome: "Publish and verify gemstone-js plus @gemstone-js/native from a clean install across supported Node platforms.",
        verify_with: "cd /Users/tariq/src/gemstone-js && npm run verify && GS_RUN_LIVE=1 npm run test:live",
    },
    BatchInfo {
        number: 2,
        focus: "Visual tooling",
        hours_min: 10,
        hours_max: 18,
        outcome: "Add explorer/workbench flows for inspect, browse, TypeScript codegen, and persistent roots.",
        verify_with: "npm run inspect; npm run examples; browser smoke for the future explorer",
    },
    BatchInfo {
        number: 3,
        focus: "Installed examples",
        hours_min: 5,
        hours_max: 8,
        outcome: "Add clean-install examples for quickstart, web adapters, migrations, codegen, and persistence helpers.",
        verify_with: "npm run examples:check; gemstone-js-examples --json",
    },
    BatchInfo {
        number: 4,
        focus: "Live CI",
        hours_min: 8,
        hours_max: 14,
        outcome: "Cover Node, Deno, Bun, framework adapters, pools, transactions, migrations, and query helpers against a real stone.",
        verify_with: "GS_RUN_LIVE=1 npm run test:live",
    },
    BatchInfo {
        number: 5,
        focus: "Documentation and release polish",
        hours_min: 5,
        hours_max: 8,
        outcome: "Add a Medium-style article, PDF docs, release checklist, checksums, and npm post-publish verification.",
        verify_with: "npm run pack:check; docs link check when available",
    },
    BatchInfo {
        number: 6,
        focus: "Cross-project alignment",
        hours_min: 8,
        hours_max: 14,
        outcome: "Align gemstone-js concepts with gemstone-py and gemstone-rs explorers, codegen, persistent roots, and live smoke workflows.",
        verify_with: "gemstone-js comparison checklist plus clean-install smoke scripts",
    },
];

const GEMSTONE_JS_GAPS: &[GapInfo] = &[
    GapInfo {
        priority: "P1",
        area: "Published native confidence",
        gemstone_py_strength: "gemstone-py has a working package/release lane and optional native acceleration path.",
        gemstone_rs_gap: "gemstone-js is alpha and depends on optional @gemstone-js/native plus Deno/Bun FFI starter paths that need broader live proof.",
        next_action: "Publish and verify gemstone-js plus @gemstone-js/native across supported Node platforms, then run npm verify and live smoke tests from a clean install.",
        verify_with: "cd /Users/tariq/src/gemstone-js && npm run verify && GS_RUN_LIVE=1 npm run test:live",
    },
    GapInfo {
        priority: "P1",
        area: "Visual tooling",
        gemstone_py_strength: "gemstone-py has python-gemstone-database-explorer and a more mature VS Code workbench flow.",
        gemstone_rs_gap: "gemstone-js has inspect/doctor/examples CLIs but no equivalent visual database explorer or VS Code workbench yet.",
        next_action: "Build a gemstone-js explorer/workbench layer or reuse the Rust/Python explorer concepts for TypeScript codegen and inspection workflows.",
        verify_with: "npm run inspect; npm run examples; browser smoke for the future explorer",
    },
    GapInfo {
        priority: "P1",
        area: "Installed examples",
        gemstone_py_strength: "gemstone-py has a mature examples guide and installed example runner.",
        gemstone_rs_gap: "gemstone-js has gemstone-js-examples and a catalog, but needs more end-to-end installed-package examples with live GemStone credentials.",
        next_action: "Add clean-install example smoke tests for quickstart, codegen, migrations, web adapters, and persistence helpers.",
        verify_with: "npm run examples:check; gemstone-js-examples --json",
    },
    GapInfo {
        priority: "P2",
        area: "Live coverage",
        gemstone_py_strength: "gemstone-py has broader live GemStone testing across sync, async, framework, lifetime, and native behavior.",
        gemstone_rs_gap: "gemstone-js has a live test entry point, but more runtime/platform combinations and framework-adapter live flows are needed.",
        next_action: "Add scheduled/manual live CI for login, eval, performWith, pools, request scopes, migrations, query helpers, and all web adapters.",
        verify_with: "GS_RUN_LIVE=1 npm run test:live",
    },
    GapInfo {
        priority: "P2",
        area: "Documentation polish",
        gemstone_py_strength: "gemstone-py has a broader article/docs/PDF/release documentation set.",
        gemstone_rs_gap: "gemstone-js docs are extensive but need a polished article, release checklist, and comparison docs equivalent to gemstone-py.",
        next_action: "Add a Medium-style article, release checklist, PDF docs, and a clear gemstone-js vs gemstone-py guide.",
        verify_with: "npm run pack:check; docs link check when available",
    },
];

const NO_EXTRA_SCAFFOLD_FILES: &[ScaffoldFile] = &[];

const SCAFFOLD_TEMPLATES: &[ScaffoldTemplate] = &[
    ScaffoldTemplate {
        name: "quickstart",
        package_name: "gemstone-rs-quickstart",
        title: "gemstone-rs Quickstart",
        description: "Login, evaluate 3 + 4, and round-trip a UserGlobals string.",
        main_rs: include_str!("../templates/quickstart.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "browser",
        package_name: "gemstone-rs-browser",
        title: "gemstone-rs Browser",
        description: "Browse dictionaries, protocols, methods, and Object>>printString source.",
        main_rs: include_str!("../templates/browser.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "bridge_root_mapping",
        package_name: "gemstone-rs-bridge-root-mapping",
        title: "gemstone-rs BridgeRoot Mapping",
        description: "Store and read a typed Rust payload under GemStoneRsBridgeRoot.",
        main_rs: include_str!("../templates/bridge_root_mapping.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "derive_mapping",
        package_name: "gemstone-rs-derive-mapping",
        title: "gemstone-rs Derive Mapping",
        description: "Use #[derive(BridgeMapped)] with nested structs, vectors, maps, optionals, and symbol keys.",
        main_rs: include_str!("../templates/derive_mapping.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "codegen_preview",
        package_name: "gemstone-rs-codegen-preview",
        title: "gemstone-rs Codegen Preview",
        description:
            "Preview generated Rust wrappers without writing files or connecting to GemStone.",
        main_rs: include_str!("../templates/codegen_preview.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "codegen_workflow",
        package_name: "gemstone-rs-codegen-workflow",
        title: "gemstone-rs Codegen Workflow",
        description: "Run config, preview, diff, check, and generate in one offline example.",
        main_rs: include_str!("../templates/codegen_workflow.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "codegen_discover",
        package_name: "gemstone-rs-codegen-discover",
        title: "gemstone-rs Codegen Discovery",
        description: "Discover a starter wrapper config from live GemStone classes.",
        main_rs: include_str!("../templates/codegen_discover.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "codegen_discover_mapping",
        package_name: "gemstone-rs-codegen-discover-mapping",
        title: "gemstone-rs Mapping Discovery",
        description: "Discover a starter BridgeRoot mapping config from a live class.",
        main_rs: include_str!("../templates/codegen_discover_mapping.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "profile_codegen_workflow",
        package_name: "gemstone-rs-profile-codegen-workflow",
        title: "gemstone-rs Profile Codegen Workflow",
        description: "Run profile-driven codegen explain, generate, and profile check from scaffolded config files.",
        main_rs: include_str!("../templates/profile_codegen_workflow.rs"),
        extra_dependencies: "",
        extra_files: &[
            ScaffoldFile {
                path: "gemstone-rs.codegen",
                source: include_str!("../templates/profile_codegen_workflow.codegen"),
            },
            ScaffoldFile {
                path: "gemstone-rs.codegen-profiles.json",
                source: include_str!("../templates/profile_codegen_workflow.codegen-profiles.json"),
            },
        ],
    },
    ScaffoldTemplate {
        name: "generated_wrapper_app",
        package_name: "gemstone-rs-generated-wrapper-app",
        title: "gemstone-rs Generated Wrapper App",
        description: "Call Object>>printString through a generated-style Rust wrapper.",
        main_rs: include_str!("../templates/generated_wrapper_app.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "generated_mapping_app",
        package_name: "gemstone-rs-generated-mapping-app",
        title: "gemstone-rs Generated Mapping App",
        description: "Use generated-style BridgeMapped structs stored under BridgeRoot.",
        main_rs: include_str!("../templates/generated_mapping_app.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "http_service",
        package_name: "gemstone-rs-http-service",
        title: "gemstone-rs HTTP Service",
        description: "Standard-library HTTP service with /, /health/local, and /health/gemstone.",
        main_rs: include_str!("../templates/http_service.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "session_worker_pool",
        package_name: "gemstone-rs-session-worker-pool",
        title: "gemstone-rs Session Worker Pool",
        description: "Bounded pool of dedicated GemStone session workers for web services.",
        main_rs: include_str!("../templates/session_worker_pool.rs"),
        extra_dependencies: "",
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "py_native_pyo3_adapter",
        package_name: "gemstone-py-native-starter",
        title: "gemstone-py-native PyO3 Starter",
        description: "Thin PyO3 adapter scaffold over gemstone_rs::py_native for gemstone-py-native style packages.",
        main_rs: include_str!("../templates/py_native_pyo3_adapter.rs"),
        extra_dependencies: r#"pyo3 = "0.28"

[features]
extension-module = ["pyo3/extension-module"]

[lib]
name = "gemstone_py_native"
crate-type = ["cdylib", "rlib"]
"#,
        extra_files: &[
            ScaffoldFile {
                path: "src/lib.rs",
                source: include_str!("../templates/py_native_pyo3_adapter_lib.rs"),
            },
            ScaffoldFile {
                path: "pyproject.toml",
                source: include_str!("../templates/py_native_pyo3_adapter_pyproject.toml"),
            },
            ScaffoldFile {
                path: "PYTHON.md",
                source: include_str!("../templates/py_native_pyo3_adapter_python.md"),
            },
            ScaffoldFile {
                path: "python/gemstone_py_native_compat.py",
                source: include_str!("../templates/py_native_pyo3_adapter_compat.py"),
            },
            ScaffoldFile {
                path: "tests/test_smoke.py",
                source: include_str!("../templates/py_native_pyo3_adapter_smoke_test.py"),
            },
        ],
    },
    ScaffoldTemplate {
        name: "axum_service",
        package_name: "gemstone-rs-axum-service",
        title: "gemstone-rs Axum Service",
        description: "Standalone Axum service with /, /health/local, and /health/gemstone.",
        main_rs: include_str!("../templates/axum_service.rs"),
        extra_dependencies: r#"axum = "0.8"
gemstone-rs-axum = "{gemstone_rs_version}"
tokio = { version = "1", features = ["macros", "net", "rt-multi-thread"] }
"#,
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
    ScaffoldTemplate {
        name: "actix_service",
        package_name: "gemstone-rs-actix-service",
        title: "gemstone-rs Actix Service",
        description: "Standalone Actix Web service with /, /health/local, and /health/gemstone.",
        main_rs: include_str!("../templates/actix_service.rs"),
        extra_dependencies: r#"actix-web = "4"
gemstone-rs-actix = "{gemstone_rs_version}"
"#,
        extra_files: NO_EXTRA_SCAFFOLD_FILES,
    },
];

fn print_hello(format: OutputFormat) {
    let version = env!("CARGO_PKG_VERSION");
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let family = env::consts::FAMILY;
    let executable = env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    match format {
        OutputFormat::Human => {
            println!("Hello from:");
            println!("  gemstone-rs CLI version: {version}");
            println!("  Rust target OS:          {os}");
            println!("  Rust target arch:        {arch}");
            println!("  Rust target family:      {family}");
            println!("  Executable:              {executable}");
        }
        OutputFormat::Json => {
            println!(
                r#"{{"name":"gemstone-rs","version":"{}","os":"{}","arch":"{}","family":"{}","executable":"{}"}}"#,
                escape_json(version),
                escape_json(os),
                escape_json(arch),
                escape_json(family),
                escape_json(&executable)
            );
        }
    }
}

fn print_py_native_capabilities(format: OutputFormat) {
    let capabilities = py_native::capabilities();
    match format {
        OutputFormat::Human => {
            println!("{}", py_native::PY_NATIVE_CONTRACT_NAME);
            println!("  contract_version: {}", capabilities.contract_version);
            println!("  threading: {}", capabilities.threading);
            println!("  operations: {}", capabilities.operations.join(", "));
            println!(
                "  value_kinds: {}",
                py_native::PY_NATIVE_VALUE_KINDS.join(", ")
            );
            println!(
                "  error_kinds: {}",
                py_native::PY_NATIVE_ERROR_KINDS.join(", ")
            );
            println!("  oop_constants:");
            println!("    nil: {}", py_native::nil_oop());
            println!("    true: {}", py_native::bool_oop(true));
            println!("    false: {}", py_native::bool_oop(false));
            println!("    smallint_7: {}", py_native::smallint_oop(7));
            println!("    char_A: {}", py_native::char_oop('A'));
        }
        OutputFormat::Json => println!("{}", capabilities.to_json()),
    }
}

fn default_py_native_fixture_path() -> PathBuf {
    PathBuf::from("examples/py-native/gemstone-rs.py-native.json")
}

fn default_py_native_smoke_fixture_path() -> PathBuf {
    PathBuf::from("examples/py-native/gemstone-rs.py-native-smoke.json")
}

fn default_py_native_samples_fixture_path() -> PathBuf {
    PathBuf::from("examples/py-native/gemstone-rs.py-native-samples.json")
}

fn default_py_native_compatibility_fixture_path() -> PathBuf {
    PathBuf::from("examples/py-native/gemstone-rs.py-native-compat.json")
}

fn default_py_native_conformance_fixture_path() -> PathBuf {
    PathBuf::from("examples/py-native/gemstone-rs.py-native-conformance.json")
}

fn default_py_native_handoff_fixture_path() -> PathBuf {
    PathBuf::from("examples/py-native/gemstone-rs.py-native-handoff.json")
}

fn default_py_native_publish_receipt_fixture_path() -> PathBuf {
    PathBuf::from("examples/py-native/gemstone-rs.py-native-publish-receipt.json")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PyNativeCheckAllStep {
    name: &'static str,
    path: PathBuf,
    command: &'static str,
    ok: bool,
    error: Option<String>,
}

impl PyNativeCheckAllStep {
    fn to_json(&self) -> String {
        let error = self
            .error
            .as_ref()
            .map(|value| format!(r#""{}""#, escape_json(value)))
            .unwrap_or_else(|| "null".to_string());
        format!(
            r#"{{"name":"{}","path":"{}","command":"{}","ok":{},"error":{}}}"#,
            escape_json(self.name),
            escape_json(&self.path.display().to_string()),
            escape_json(self.command),
            if self.ok { "true" } else { "false" },
            error
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PyNativeCheckAllReport {
    contract_version: u16,
    root: Option<PathBuf>,
    steps: Vec<PyNativeCheckAllStep>,
}

impl PyNativeCheckAllReport {
    fn ok(&self) -> bool {
        self.steps.iter().all(|step| step.ok)
    }

    fn ok_count(&self) -> usize {
        self.steps.iter().filter(|step| step.ok).count()
    }

    fn error_count(&self) -> usize {
        self.steps.len() - self.ok_count()
    }

    fn to_json(&self) -> String {
        let root = self
            .root
            .as_ref()
            .map(|path| format!(r#""{}""#, escape_json(&path.display().to_string())))
            .unwrap_or_else(|| "null".to_string());
        let steps = self
            .steps
            .iter()
            .map(PyNativeCheckAllStep::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"ok":{},"contractVersion":{},"root":{},"stepCount":{},"okCount":{},"errorCount":{},"steps":[{}]}}"#,
            if self.ok() { "true" } else { "false" },
            self.contract_version,
            root,
            self.steps.len(),
            self.ok_count(),
            self.error_count(),
            steps
        )
    }
}

fn run_py_native_check_all(root: Option<&Path>, format: OutputFormat) -> Result<(), CliError> {
    let report = build_py_native_check_all_report(root);
    match format {
        OutputFormat::Json => println!("{}", report.to_json()),
        OutputFormat::Human => {
            println!("py-native shared-core gate");
            println!(
                "  root: {}",
                report
                    .root
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| ".".to_string())
            );
            println!(
                "  summary: {} ok, {} errors, {} total",
                report.ok_count(),
                report.error_count(),
                report.steps.len()
            );
            for step in &report.steps {
                println!(
                    "  {}\t{}\t{}",
                    if step.ok { "ok" } else { "error" },
                    step.name,
                    step.path.display()
                );
                if let Some(error) = &step.error {
                    println!("    {error}");
                }
            }
        }
    }

    if report.ok() {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "py-native shared-core gate failed: {} errors",
            report.error_count()
        )))
    }
}

fn build_py_native_check_all_report(root: Option<&Path>) -> PyNativeCheckAllReport {
    let mut steps = Vec::new();
    push_py_native_check_all_step(
        &mut steps,
        root,
        "capabilities",
        default_py_native_fixture_path(),
        py_native::capabilities().to_json(),
        "gemstone-rs py-native check examples/py-native/gemstone-rs.py-native.json",
    );
    push_py_native_check_all_step(
        &mut steps,
        root,
        "samples",
        default_py_native_samples_fixture_path(),
        py_native::samples_report().to_json(),
        "gemstone-rs py-native check-samples examples/py-native/gemstone-rs.py-native-samples.json",
    );
    push_py_native_check_all_step(
        &mut steps,
        root,
        "smoke",
        default_py_native_smoke_fixture_path(),
        py_native::smoke_dry_run_report().to_json(),
        "gemstone-rs py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json",
    );
    push_py_native_check_all_step(
        &mut steps,
        root,
        "compatibility",
        default_py_native_compatibility_fixture_path(),
        py_native::compatibility_report().to_json(),
        "gemstone-rs py-native check-compat examples/py-native/gemstone-rs.py-native-compat.json",
    );
    push_py_native_check_all_step(
        &mut steps,
        root,
        "conformance",
        default_py_native_conformance_fixture_path(),
        py_native::conformance_report().to_json(),
        "gemstone-rs py-native check-conformance examples/py-native/gemstone-rs.py-native-conformance.json",
    );
    push_py_native_check_all_step(
        &mut steps,
        root,
        "handoff",
        default_py_native_handoff_fixture_path(),
        py_native::handoff_report().to_json(),
        "gemstone-rs py-native check-handoff examples/py-native/gemstone-rs.py-native-handoff.json",
    );
    push_py_native_check_all_step(
        &mut steps,
        root,
        "publish-receipt",
        default_py_native_publish_receipt_fixture_path(),
        py_native::publish_receipt().to_json(),
        "gemstone-rs py-native check-publish-receipt examples/py-native/gemstone-rs.py-native-publish-receipt.json",
    );
    PyNativeCheckAllReport {
        contract_version: py_native::capabilities().contract_version,
        root: root.map(Path::to_path_buf),
        steps,
    }
}

fn push_py_native_check_all_step(
    steps: &mut Vec<PyNativeCheckAllStep>,
    root: Option<&Path>,
    name: &'static str,
    relative_path: PathBuf,
    expected: String,
    command: &'static str,
) {
    let path = root
        .map(|root| root.join(&relative_path))
        .unwrap_or(relative_path);
    let error = match fs::read_to_string(&path) {
        Ok(actual) if actual.trim_end() == expected => None,
        Ok(_) => Some(format!("{} does not match `{}`", path.display(), command)),
        Err(error) => Some(format!("{}: {}", path.display(), error)),
    };
    steps.push(PyNativeCheckAllStep {
        name,
        path,
        command,
        ok: error.is_none(),
        error,
    });
}

fn run_py_native_check(path: &Path, format: OutputFormat) -> Result<(), CliError> {
    let expected = py_native::capabilities().to_json();
    let actual = fs::read_to_string(path)?;
    let actual = actual.trim_end();
    let matches = actual == expected;

    match format {
        OutputFormat::Human => {
            if matches {
                println!("py-native contract ok: {}", path.display());
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"path":"{}","ok":{},"contractVersion":{}}}"#,
                escape_json(&path.display().to_string()),
                if matches { "true" } else { "false" },
                py_native::capabilities().contract_version
            );
        }
    }

    if matches {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "{} does not match `gemstone-rs py-native capabilities --json`; regenerate or review the contract change",
            path.display()
        )))
    }
}

fn print_py_native_samples(format: OutputFormat) {
    let report = py_native::samples_report();
    match format {
        OutputFormat::Human => {
            println!("py-native adapter samples");
            println!("  contract_version: {}", report.contract_version);
            println!("  values:");
            for sample in &report.values {
                println!("    {}: {}", sample.name, sample.value.to_json());
            }
            println!("  errors:");
            for sample in &report.errors {
                println!("    {}: {}", sample.name, sample.error.to_json());
            }
        }
        OutputFormat::Json => println!("{}", report.to_json()),
    }
}

fn run_py_native_samples_check(path: &Path, format: OutputFormat) -> Result<(), CliError> {
    let report = py_native::samples_report();
    let expected = report.to_json();
    let actual = fs::read_to_string(path)?;
    let actual = actual.trim_end();
    let matches = actual == expected;

    match format {
        OutputFormat::Human => {
            if matches {
                println!("py-native samples contract ok: {}", path.display());
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"path":"{}","ok":{},"contractVersion":{},"valueCount":{},"errorCount":{}}}"#,
                escape_json(&path.display().to_string()),
                if matches { "true" } else { "false" },
                report.contract_version,
                report.values.len(),
                report.errors.len()
            );
        }
    }

    if matches {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "{} does not match `gemstone-rs py-native samples --json`; regenerate or review the samples contract change",
            path.display()
        )))
    }
}

fn print_py_native_compatibility(format: OutputFormat) {
    let report = py_native::compatibility_report();
    match format {
        OutputFormat::Json => println!("{}", report.to_json()),
        OutputFormat::Human => {
            println!("py-native compatibility API");
            println!("  contract_version: {}", report.contract_version);
            println!("  module: {}", report.module);
            println!("  session_class: {}", report.session_class);
            println!("  handle_class: {}", report.handle_class);
            println!("  return_policy: {}", report.return_policy);
            println!("  methods:");
            for method in &report.methods {
                println!(
                    "    {} -> {}: {} -> {}",
                    method.python_method,
                    method.native_method,
                    method.native_return,
                    method.python_return
                );
            }
        }
    }
}

fn run_py_native_compatibility_check(path: &Path, format: OutputFormat) -> Result<(), CliError> {
    let report = py_native::compatibility_report();
    let expected = report.to_json();
    let actual = fs::read_to_string(path)?;
    let actual = actual.trim_end();
    let matches = actual == expected;

    match format {
        OutputFormat::Human => {
            if matches {
                println!("py-native compatibility contract ok: {}", path.display());
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"path":"{}","ok":{},"contractVersion":{},"methodCount":{}}}"#,
                escape_json(&path.display().to_string()),
                if matches { "true" } else { "false" },
                report.contract_version,
                report.methods.len()
            );
        }
    }

    if matches {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "{} does not match `gemstone-rs py-native compatibility --json`; regenerate or review the compatibility contract change",
            path.display()
        )))
    }
}

fn print_py_native_conformance(format: OutputFormat) {
    let report = py_native::conformance_report();
    match format {
        OutputFormat::Json => println!("{}", report.to_json()),
        OutputFormat::Human => {
            println!("py-native wrapper conformance");
            println!("  target_package: {}", report.target_package);
            println!("  contract_version: {}", report.contract_version);
            println!("  status: {}", report.status);
            println!("  module_functions: {}", report.module_functions.join(", "));
            println!(
                "  native_session_methods: {}",
                report.native_session_methods.join(", ")
            );
            println!(
                "  compatibility_methods: {}",
                report.compatibility_methods.join(", ")
            );
            println!("  fixtures:");
            for fixture in &report.fixtures {
                println!("    {}: {}", fixture.path, fixture.purpose);
                println!("      {}", fixture.command);
            }
            println!("  scaffold_files:");
            for file in &report.scaffold_files {
                println!("    {}: {}", file.path, file.purpose);
            }
        }
    }
}

fn run_py_native_conformance_check(path: &Path, format: OutputFormat) -> Result<(), CliError> {
    let report = py_native::conformance_report();
    let expected = report.to_json();
    let actual = fs::read_to_string(path)?;
    let actual = actual.trim_end();
    let matches = actual == expected;

    match format {
        OutputFormat::Human => {
            if matches {
                println!("py-native conformance contract ok: {}", path.display());
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"path":"{}","ok":{},"contractVersion":{},"moduleFunctionCount":{},"sessionMethodCount":{},"compatibilityMethodCount":{},"fixtureCount":{},"scaffoldFileCount":{}}}"#,
                escape_json(&path.display().to_string()),
                if matches { "true" } else { "false" },
                report.contract_version,
                report.module_functions.len(),
                report.native_session_methods.len(),
                report.compatibility_methods.len(),
                report.fixtures.len(),
                report.scaffold_files.len()
            );
        }
    }

    if matches {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "{} does not match `gemstone-rs py-native conformance --json`; regenerate or review the conformance contract change",
            path.display()
        )))
    }
}

fn print_py_native_handoff(format: OutputFormat) {
    let report = py_native::handoff_report();
    match format {
        OutputFormat::Json => println!("{}", report.to_json()),
        OutputFormat::Human => {
            println!("py-native gemstone-py handoff");
            println!("  target_package: {}", report.target_package);
            println!("  adapter_module: {}", report.adapter_module);
            println!("  scaffold: {}", report.scaffold);
            println!("  contract_version: {}", report.contract_version);
            println!("  status: {}", report.status);
            println!("  artifacts:");
            for artifact in &report.artifacts {
                println!("    {}: {}", artifact.name, artifact.purpose);
                println!("      path: {}", optional_field(artifact.path));
                println!("      schema: {}", artifact.schema);
                println!("      command: {}", artifact.command);
                println!("      check: {}", optional_field(artifact.check_command));
            }
            println!("  acceptance:");
            for criterion in &report.acceptance {
                println!(
                    "    {} [{}]",
                    criterion.id,
                    if criterion.required {
                        "required"
                    } else {
                        "optional"
                    }
                );
                println!("      verify: {}", criterion.verify);
            }
        }
    }
}

fn run_py_native_handoff_check(path: &Path, format: OutputFormat) -> Result<(), CliError> {
    let report = py_native::handoff_report();
    let expected = report.to_json();
    let actual = fs::read_to_string(path)?;
    let actual = actual.trim_end();
    let matches = actual == expected;

    match format {
        OutputFormat::Human => {
            if matches {
                println!("py-native handoff bundle ok: {}", path.display());
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"path":"{}","ok":{},"contractVersion":{},"artifactCount":{},"acceptanceCount":{}}}"#,
                escape_json(&path.display().to_string()),
                if matches { "true" } else { "false" },
                report.contract_version,
                report.artifacts.len(),
                report.acceptance.len()
            );
        }
    }

    if matches {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "{} does not match `gemstone-rs py-native handoff --json`; regenerate or review the handoff bundle change",
            path.display()
        )))
    }
}

fn print_py_native_publish_receipt(format: OutputFormat) {
    let receipt = py_native::publish_receipt();
    match format {
        OutputFormat::Json => println!("{}", receipt.to_json()),
        OutputFormat::Human => {
            println!("py-native publish receipt");
            println!("  target_package: {}", receipt.target_package);
            println!("  rust_core: {}", receipt.rust_core);
            println!("  release_tag: {}", receipt.release_tag);
            println!("  status: {}", receipt.status);
            println!("  targets:");
            for target in &receipt.targets {
                println!(
                    "    {} {} {} [{}]",
                    target.index, target.package, target.version, target.conclusion
                );
                println!("      workflow: {} #{}", target.workflow, target.run_id);
                println!("      run: {}", target.run_url);
                println!("      verified_at: {}", target.verified_at);
                println!("      install: {}", target.install_command);
                println!("      checks: {}", target.verification);
            }
        }
    }
}

fn run_py_native_publish_receipt_check(path: &Path, format: OutputFormat) -> Result<(), CliError> {
    let receipt = py_native::publish_receipt();
    let expected = receipt.to_json();
    let actual = fs::read_to_string(path)?;
    let actual = actual.trim_end();
    let matches = actual == expected;

    match format {
        OutputFormat::Human => {
            if matches {
                println!("py-native publish receipt ok: {}", path.display());
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"path":"{}","ok":{},"contractVersion":{},"targetCount":{},"releaseTag":"{}"}}"#,
                escape_json(&path.display().to_string()),
                if matches { "true" } else { "false" },
                receipt.contract_version,
                receipt.targets.len(),
                escape_json(receipt.release_tag)
            );
        }
    }

    if matches {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "{} does not match `gemstone-rs py-native publish-receipt --json`; regenerate or review the publish receipt change",
            path.display()
        )))
    }
}

fn run_py_native_smoke_check(path: &Path, format: OutputFormat) -> Result<(), CliError> {
    let report = py_native::smoke_dry_run_report();
    let expected = report.to_json();
    let actual = fs::read_to_string(path)?;
    let actual = actual.trim_end();
    let matches = actual == expected;

    match format {
        OutputFormat::Human => {
            if matches {
                println!("py-native smoke contract ok: {}", path.display());
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"path":"{}","ok":{},"dryRun":{},"contractVersion":{}}}"#,
                escape_json(&path.display().to_string()),
                if matches { "true" } else { "false" },
                if report.dry_run { "true" } else { "false" },
                report.contract_version
            );
        }
    }

    if matches {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(format!(
            "{} does not match `gemstone-rs py-native smoke --dry-run --json`; regenerate or review the smoke contract change",
            path.display()
        )))
    }
}

fn run_py_native_smoke(dry_run: bool, format: OutputFormat) -> Result<(), CliError> {
    let report = if dry_run {
        py_native::smoke_dry_run_report()
    } else {
        py_native::smoke_live_report()
    };

    match format {
        OutputFormat::Human => print_py_native_smoke_human(&report),
        OutputFormat::Json => print_py_native_smoke_json(&report),
    }

    if report.ok() {
        Ok(())
    } else {
        Err(CliError::CodegenCheck(
            "py-native adapter smoke failed; see step report above".to_string(),
        ))
    }
}

fn print_py_native_smoke_human(report: &py_native::PyNativeSmokeReport) {
    println!("py-native adapter smoke");
    println!(
        "  mode: {}",
        if report.dry_run {
            "dry-run"
        } else {
            "live GemStone"
        }
    );
    for step in &report.steps {
        println!(
            "  {} {}: {}",
            if step.ok { "ok" } else { "error" },
            step.name,
            step.detail
        );
    }
}

fn print_py_native_smoke_json(report: &py_native::PyNativeSmokeReport) {
    println!("{}", report.to_json());
}

fn print_py_native_migration(format: OutputFormat) {
    let report = py_native::migration_report();
    match format {
        OutputFormat::Json => println!("{}", report.to_json()),
        OutputFormat::Human => {
            println!("py-native gemstone-py migration");
            println!("  target_package: {}", report.target_package);
            println!("  contract_version: {}", report.contract_version);
            println!("  status: {}", report.status);
            println!(
                "  progress: {} done, {} pending",
                report.done_count(),
                report.pending_count()
            );
            println!("  steps:");
            for step in &report.steps {
                println!("    {} {}: {}", step.status, step.id, step.title);
                println!("      {}", step.detail);
                println!("      verify: {}", step.verify);
            }
        }
    }
}

fn print_gemstone_py_comparison(view: CompareView, format: OutputFormat) {
    match view {
        CompareView::Summary => print_gemstone_py_comparison_summary(format),
        CompareView::Status => print_status(
            "gemstone-rs status vs gemstone-py",
            gemstone_py_scorecard_info(),
            GEMSTONE_RS_PARITY,
            format,
        ),
        CompareView::Scorecard => print_scorecard(gemstone_py_scorecard_info(), format),
        CompareView::Parity => print_parity(
            "gemstone-rs parity vs gemstone-py",
            "gemstone-py",
            "gemstone-rs",
            GEMSTONE_RS_PARITY,
            format,
        ),
        CompareView::Gaps => print_gemstone_py_gap_report(format),
        CompareView::Next => print_next_action(
            "gemstone-rs next action vs gemstone-py",
            "gemstone-py",
            "docs/gemstone-py-vs-gemstone-rs.md",
            GEMSTONE_RS_BATCHES,
            GEMSTONE_PY_GAPS,
            "gemstone-rs",
            format,
        ),
        CompareView::Totals => print_batch_totals(
            "gemstone-rs remaining work vs gemstone-py",
            "gemstone-py",
            GEMSTONE_RS_BATCHES,
            format,
        ),
        CompareView::Batches => print_batch_plan(
            "gemstone-rs remaining batches vs gemstone-py",
            "gemstone-py",
            "batches",
            "docs/gemstone-py-vs-gemstone-rs.md",
            GEMSTONE_RS_BATCHES,
            format,
        ),
    }
}

fn print_gemstone_js_comparison(view: CompareView, format: OutputFormat) {
    match view {
        CompareView::Summary => print_gemstone_js_comparison_summary(format),
        CompareView::Status => print_status(
            "gemstone-js status vs gemstone-py",
            gemstone_js_scorecard_info(),
            GEMSTONE_JS_PARITY,
            format,
        ),
        CompareView::Scorecard => print_scorecard(gemstone_js_scorecard_info(), format),
        CompareView::Parity => print_parity(
            "gemstone-js parity vs gemstone-py",
            "gemstone-js",
            "gemstone-js",
            GEMSTONE_JS_PARITY,
            format,
        ),
        CompareView::Gaps => print_gemstone_js_gap_report(format),
        CompareView::Next => print_next_action(
            "gemstone-js next action vs gemstone-py",
            "gemstone-js",
            "docs/gemstone-js-vs-gemstone-py.md",
            GEMSTONE_JS_BATCHES,
            GEMSTONE_JS_GAPS,
            "gemstone-js",
            format,
        ),
        CompareView::Totals => print_batch_totals(
            "gemstone-js remaining work vs gemstone-py",
            "gemstone-js",
            GEMSTONE_JS_BATCHES,
            format,
        ),
        CompareView::Batches => print_batch_plan(
            "gemstone-js catch-up batches vs gemstone-py",
            "gemstone-js",
            "batches",
            "docs/gemstone-js-vs-gemstone-py.md",
            GEMSTONE_JS_BATCHES,
            format,
        ),
    }
}

fn print_all_comparisons(view: CompareView, format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("active gemstone-rs comparison reports");
            if matches!(view, CompareView::Batches | CompareView::Totals) {
                let totals = all_batch_totals();
                println!(
                    "  Active total: {} batches, roughly {}-{} hours",
                    totals.total_batches, totals.hours_min, totals.hours_max
                );
            }
            println!();
            match view {
                CompareView::Totals => {
                    print_batch_totals(
                        "gemstone-rs remaining work vs gemstone-py",
                        "gemstone-py",
                        GEMSTONE_RS_BATCHES,
                        OutputFormat::Human,
                    );
                }
                CompareView::Status => {
                    let totals = all_batch_totals();
                    println!(
                        "Active remaining work: {} batches, roughly {}-{} hours",
                        totals.total_batches, totals.hours_min, totals.hours_max
                    );
                    println!();
                    print_status(
                        "gemstone-rs status vs gemstone-py",
                        gemstone_py_scorecard_info(),
                        GEMSTONE_RS_PARITY,
                        OutputFormat::Human,
                    );
                }
                CompareView::Scorecard => {
                    print_scorecard(gemstone_py_scorecard_info(), OutputFormat::Human);
                }
                CompareView::Parity => {
                    print_parity(
                        "gemstone-rs parity vs gemstone-py",
                        "gemstone-py",
                        "gemstone-rs",
                        GEMSTONE_RS_PARITY,
                        OutputFormat::Human,
                    );
                }
                _ => print_gemstone_py_comparison(view, OutputFormat::Human),
            }
        }
        OutputFormat::Json => print_all_comparisons_json(view),
    }
}

fn print_all_comparisons_json(view: CompareView) {
    match view {
        CompareView::Summary => {
            println!(
                r#"{{"comparison":"all","view":"summary","activeOnly":true,"comparisons":[{}]}}"#,
                comparison_summary_json_entry(
                    "gemstone-py",
                    GEMSTONE_PY_COMPARISON
                        .iter()
                        .map(comparison_json)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            );
        }
        CompareView::Status => {
            let totals = all_batch_totals();
            println!(
                r#"{{"comparison":"all","view":"status","activeOnly":true,"totalBatches":{},"hoursMin":{},"hoursMax":{},"comparisons":[{}]}}"#,
                totals.total_batches,
                totals.hours_min,
                totals.hours_max,
                status_json_entry(gemstone_py_scorecard_info(), GEMSTONE_RS_PARITY)
            );
        }
        CompareView::Scorecard => {
            let totals = all_batch_totals();
            println!(
                r#"{{"comparison":"all","view":"scorecard","activeOnly":true,"totalBatches":{},"hoursMin":{},"hoursMax":{},"comparisons":[{}]}}"#,
                totals.total_batches,
                totals.hours_min,
                totals.hours_max,
                scorecard_json_entry(gemstone_py_scorecard_info())
            );
        }
        CompareView::Parity => {
            println!(
                r#"{{"comparison":"all","view":"parity","activeOnly":true,"comparisons":[{}]}}"#,
                parity_json_entry("gemstone-py", "gemstone-rs", GEMSTONE_RS_PARITY)
            );
        }
        CompareView::Gaps => {
            println!(
                r#"{{"comparison":"all","view":"gaps","activeOnly":true,"comparisons":[{}]}}"#,
                gap_report_json_entry(
                    "gemstone-py",
                    GEMSTONE_PY_GAPS
                        .iter()
                        .map(gap_json)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            );
        }
        CompareView::Next => {
            println!(
                r#"{{"comparison":"all","view":"next","activeOnly":true,"comparisons":[{}]}}"#,
                next_action_json_entry(
                    "gemstone-py",
                    GEMSTONE_RS_BATCHES,
                    GEMSTONE_PY_GAPS,
                    "gemstone-rs"
                )
            );
        }
        CompareView::Totals => {
            let totals = all_batch_totals();
            println!(
                r#"{{"comparison":"all","view":"totals","activeOnly":true,"totalBatches":{},"hoursMin":{},"hoursMax":{},"comparisons":[{}]}}"#,
                totals.total_batches,
                totals.hours_min,
                totals.hours_max,
                batch_totals_json_entry("gemstone-py", GEMSTONE_RS_BATCHES)
            );
        }
        CompareView::Batches => {
            let totals = all_batch_totals();
            println!(
                r#"{{"comparison":"all","view":"batches","activeOnly":true,"totalBatches":{},"hoursMin":{},"hoursMax":{},"comparisons":[{}]}}"#,
                totals.total_batches,
                totals.hours_min,
                totals.hours_max,
                batch_plan_json_entry("gemstone-py", GEMSTONE_RS_BATCHES)
            );
        }
    }
}

fn print_gemstone_py_comparison_summary(format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("gemstone-rs vs gemstone-py");
            println!("  Full guide: docs/gemstone-py-vs-gemstone-rs.md");
            println!("  Gap report: gemstone-rs compare gemstone-py --gaps");
            println!();
            for row in GEMSTONE_PY_COMPARISON {
                println!("{}", row.topic);
                println!("  gemstone-py: {}", row.gemstone_py);
                println!("  gemstone-rs: {}", row.gemstone_rs);
                println!("  Recommendation: {}", row.recommendation);
                println!();
            }
        }
        OutputFormat::Json => {
            let rows = GEMSTONE_PY_COMPARISON
                .iter()
                .map(comparison_json)
                .collect::<Vec<_>>()
                .join(",");
            println!(
                r#"{{"comparison":"gemstone-py","view":"summary","rows":[{}]}}"#,
                rows
            );
        }
    }
}

fn gemstone_py_scorecard_info() -> ScorecardInfo {
    ScorecardInfo {
        title: "gemstone-rs scorecard vs gemstone-py",
        comparison: "gemstone-py",
        answer: "gemstone-py remains more mature for Python apps, web examples, explorer polish, and release lanes; gemstone-rs is the better fit for Rust-native services, CLIs, typed wrappers, and the shared native core.",
        gemstone_py_use_when: GEMSTONE_PY_USE_WHEN,
        project_use_when: GEMSTONE_RS_USE_WHEN,
        gemstone_py_strengths: GEMSTONE_PY_STRENGTHS,
        project_strengths: GEMSTONE_RS_STRENGTHS,
        batches: GEMSTONE_RS_BATCHES,
        gaps: GEMSTONE_PY_GAPS,
        project_label: "gemstone-rs",
    }
}

fn print_gemstone_py_gap_report(format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("gemstone-rs gaps vs gemstone-py");
            println!("  Full guide: docs/gemstone-py-vs-gemstone-rs.md");
            println!();
            for gap in GEMSTONE_PY_GAPS {
                println!("{} {}", gap.priority, gap.area);
                println!("  gemstone-py strength: {}", gap.gemstone_py_strength);
                println!("  gemstone-rs gap: {}", gap.gemstone_rs_gap);
                println!("  Next action: {}", gap.next_action);
                println!("  Verify with: {}", gap.verify_with);
                println!();
            }
        }
        OutputFormat::Json => {
            let gaps = GEMSTONE_PY_GAPS
                .iter()
                .map(gap_json)
                .collect::<Vec<_>>()
                .join(",");
            println!(
                r#"{{"comparison":"gemstone-py","view":"gaps","gaps":[{}]}}"#,
                gaps
            );
        }
    }
}

fn print_gemstone_js_comparison_summary(format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("gemstone-js vs gemstone-py");
            println!("  Full guide: docs/gemstone-js-vs-gemstone-py.md");
            println!("  Gap report: gemstone-rs compare gemstone-js --gaps");
            println!();
            for row in GEMSTONE_JS_COMPARISON {
                println!("{}", row.topic);
                println!("  gemstone-py: {}", row.gemstone_py);
                println!("  gemstone-js: {}", row.gemstone_js);
                println!("  Recommendation: {}", row.recommendation);
                println!();
            }
        }
        OutputFormat::Json => {
            let rows = GEMSTONE_JS_COMPARISON
                .iter()
                .map(js_comparison_json)
                .collect::<Vec<_>>()
                .join(",");
            println!(
                r#"{{"comparison":"gemstone-js","view":"summary","rows":[{}]}}"#,
                rows
            );
        }
    }
}

fn gemstone_js_scorecard_info() -> ScorecardInfo {
    ScorecardInfo {
        title: "gemstone-js scorecard vs gemstone-py",
        comparison: "gemstone-js",
        answer: "gemstone-py remains more mature for Python products and visual tooling; gemstone-js is the TypeScript/Node path when async JS services and npm packaging are the product boundary.",
        gemstone_py_use_when: GEMSTONE_PY_USE_WHEN,
        project_use_when: GEMSTONE_JS_USE_WHEN,
        gemstone_py_strengths: GEMSTONE_PY_STRENGTHS,
        project_strengths: GEMSTONE_JS_STRENGTHS,
        batches: GEMSTONE_JS_BATCHES,
        gaps: GEMSTONE_JS_GAPS,
        project_label: "gemstone-js",
    }
}

fn print_gemstone_js_gap_report(format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("gemstone-js gaps vs gemstone-py");
            println!("  Full guide: docs/gemstone-js-vs-gemstone-py.md");
            println!();
            for gap in GEMSTONE_JS_GAPS {
                println!("{} {}", gap.priority, gap.area);
                println!("  gemstone-py strength: {}", gap.gemstone_py_strength);
                println!("  gemstone-js gap: {}", gap.gemstone_rs_gap);
                println!("  Next action: {}", gap.next_action);
                println!("  Verify with: {}", gap.verify_with);
                println!();
            }
        }
        OutputFormat::Json => {
            let gaps = GEMSTONE_JS_GAPS
                .iter()
                .map(js_gap_json)
                .collect::<Vec<_>>()
                .join(",");
            println!(
                r#"{{"comparison":"gemstone-js","view":"gaps","gaps":[{}]}}"#,
                gaps
            );
        }
    }
}

fn print_scorecard(info: ScorecardInfo, format: OutputFormat) {
    let (hours_min, hours_max) = total_batch_hours(info.batches);
    let next_batch = info.batches.first();
    let top_gap = info
        .gaps
        .first()
        .expect("comparison gap reports must contain at least one gap");

    match format {
        OutputFormat::Human => {
            println!("{}", info.title);
            println!("  Answer: {}", info.answer);
            println!(
                "  Remaining work: {} batches, roughly {}-{} hours",
                info.batches.len(),
                hours_min,
                hours_max
            );
            println!();
            print_scorecard_list("Use gemstone-py when", info.gemstone_py_use_when);
            print_scorecard_list(
                &format!("Use {} when", info.project_label),
                info.project_use_when,
            );
            print_scorecard_list(
                "gemstone-py is stronger today at",
                info.gemstone_py_strengths,
            );
            print_scorecard_list(
                &format!("{} is stronger today at", info.project_label),
                info.project_strengths,
            );
            if let Some(next_batch) = next_batch {
                println!(
                    "Next batch: {}. {} ({}-{} hours)",
                    next_batch.number, next_batch.focus, next_batch.hours_min, next_batch.hours_max
                );
                println!("  Outcome: {}", next_batch.outcome);
                println!("  Verify with: {}", next_batch.verify_with);
            } else {
                println!("Next batch: none. Active gemstone-rs parity batches are closed.");
            }
            println!();
            println!("Top gap: {} {}", top_gap.priority, top_gap.area);
            println!("  Next action: {}", top_gap.next_action);
            println!("  Verify with: {}", top_gap.verify_with);
        }
        OutputFormat::Json => println!(
            r#"{{"comparison":"{}","view":"scorecard",{}}}"#,
            escape_json(info.comparison),
            scorecard_json_body(info)
        ),
    }
}

fn print_scorecard_list(title: &str, values: &[&'static str]) {
    println!("{title}:");
    for value in values {
        println!("  - {value}");
    }
    println!();
}

fn print_status(
    title: &str,
    info: ScorecardInfo,
    parity_rows: &[ParityInfo],
    format: OutputFormat,
) {
    let (hours_min, hours_max) = total_batch_hours(info.batches);
    let parity = parity_totals(parity_rows);
    let next_batch = info.batches.first();
    let top_gap = info
        .gaps
        .first()
        .expect("comparison gap reports must contain at least one gap");

    match format {
        OutputFormat::Human => {
            println!("{title}");
            println!("  Answer: {}", info.answer);
            println!(
                "  Parity: gemstone-py {}/{}; {} {}/{}; gap {}",
                parity.gemstone_py_score,
                parity.max_score,
                info.project_label,
                parity.project_score,
                parity.max_score,
                parity.score_gap
            );
            println!(
                "  Remaining work: {} batches, roughly {}-{} hours",
                info.batches.len(),
                hours_min,
                hours_max
            );
            if let Some(next_batch) = next_batch {
                println!(
                    "  Next batch: {}. {} ({}-{} hours)",
                    next_batch.number, next_batch.focus, next_batch.hours_min, next_batch.hours_max
                );
            } else {
                println!("  Next batch: none; active parity batches are closed");
            }
            println!("  Top gap: {} {}", top_gap.priority, top_gap.area);
            println!("  Next action: {}", top_gap.next_action);
            println!();
            println!("Commands:");
            println!("  gemstone-rs compare {} --scorecard", info.comparison);
            println!("  gemstone-rs compare {} --parity", info.comparison);
            println!("  gemstone-rs compare {} --batches", info.comparison);
        }
        OutputFormat::Json => println!(
            r#"{{"comparison":"{}","view":"status",{}}}"#,
            escape_json(info.comparison),
            status_json_body(info, parity_rows)
        ),
    }
}

fn print_parity(
    title: &str,
    comparison: &str,
    project_label: &str,
    rows: &[ParityInfo],
    format: OutputFormat,
) {
    let totals = parity_totals(rows);
    match format {
        OutputFormat::Human => {
            println!("{title}");
            println!(
                "  Scores: gemstone-py {}/{}; {} {}/{}; gap {}",
                totals.gemstone_py_score,
                totals.max_score,
                project_label,
                totals.project_score,
                totals.max_score,
                totals.score_gap
            );
            println!();
            for row in rows {
                println!("{}", row.area);
                println!("  gemstone-py: {}/5", row.gemstone_py_score);
                println!("  {project_label}: {}/5", row.project_score);
                println!("  Leader: {}", row.leader);
                println!("  Status: {}", row.status);
                println!("  Next action: {}", row.next_action);
                println!();
            }
        }
        OutputFormat::Json => println!(
            r#"{{"comparison":"{}","view":"parity",{}}}"#,
            escape_json(comparison),
            parity_json_body(project_label, rows)
        ),
    }
}

fn comparison_json(row: &ComparisonInfo) -> String {
    format!(
        r#"{{"topic":"{}","gemstonePy":"{}","gemstoneRs":"{}","recommendation":"{}"}}"#,
        escape_json(row.topic),
        escape_json(row.gemstone_py),
        escape_json(row.gemstone_rs),
        escape_json(row.recommendation)
    )
}

fn js_comparison_json(row: &JsComparisonInfo) -> String {
    format!(
        r#"{{"topic":"{}","gemstonePy":"{}","gemstoneJs":"{}","recommendation":"{}"}}"#,
        escape_json(row.topic),
        escape_json(row.gemstone_py),
        escape_json(row.gemstone_js),
        escape_json(row.recommendation)
    )
}

fn gap_json(gap: &GapInfo) -> String {
    format!(
        r#"{{"priority":"{}","area":"{}","gemstonePyStrength":"{}","gemstoneRsGap":"{}","nextAction":"{}","verifyWith":"{}"}}"#,
        escape_json(gap.priority),
        escape_json(gap.area),
        escape_json(gap.gemstone_py_strength),
        escape_json(gap.gemstone_rs_gap),
        escape_json(gap.next_action),
        escape_json(gap.verify_with)
    )
}

fn js_gap_json(gap: &GapInfo) -> String {
    format!(
        r#"{{"priority":"{}","area":"{}","gemstonePyStrength":"{}","gemstoneJsGap":"{}","nextAction":"{}","verifyWith":"{}"}}"#,
        escape_json(gap.priority),
        escape_json(gap.area),
        escape_json(gap.gemstone_py_strength),
        escape_json(gap.gemstone_rs_gap),
        escape_json(gap.next_action),
        escape_json(gap.verify_with)
    )
}

fn comparison_summary_json_entry(comparison: &str, rows: String) -> String {
    format!(
        r#"{{"comparison":"{}","rows":[{}]}}"#,
        escape_json(comparison),
        rows
    )
}

fn gap_report_json_entry(comparison: &str, gaps: String) -> String {
    format!(
        r#"{{"comparison":"{}","gaps":[{}]}}"#,
        escape_json(comparison),
        gaps
    )
}

fn scorecard_json_entry(info: ScorecardInfo) -> String {
    format!(
        r#"{{"comparison":"{}",{}}}"#,
        escape_json(info.comparison),
        scorecard_json_body(info)
    )
}

fn scorecard_json_body(info: ScorecardInfo) -> String {
    format!(
        r#""answer":"{}","gemstonePyUseWhen":[{}],"projectUseWhen":[{}],"gemstonePyStrengths":[{}],"projectStrengths":[{}],"remaining":{},"nextBatch":{},"topGap":{}"#,
        escape_json(info.answer),
        json_hint_array(info.gemstone_py_use_when),
        json_hint_array(info.project_use_when),
        json_hint_array(info.gemstone_py_strengths),
        json_hint_array(info.project_strengths),
        remaining_json(info.batches),
        optional_batch_json(info.batches.first()),
        generic_gap_json(
            info.gaps
                .first()
                .expect("comparison gap reports must contain at least one gap"),
            info.project_label
        )
    )
}

fn status_json_entry(info: ScorecardInfo, parity_rows: &[ParityInfo]) -> String {
    format!(
        r#"{{"comparison":"{}",{}}}"#,
        escape_json(info.comparison),
        status_json_body(info, parity_rows)
    )
}

fn status_json_body(info: ScorecardInfo, parity_rows: &[ParityInfo]) -> String {
    format!(
        r#""answer":"{}","remaining":{},"parity":{},"nextBatch":{},"topGap":{},"commands":{}"#,
        escape_json(info.answer),
        remaining_json(info.batches),
        parity_totals_json(parity_totals(parity_rows)),
        optional_batch_json(info.batches.first()),
        generic_gap_json(
            info.gaps
                .first()
                .expect("comparison gap reports must contain at least one gap"),
            info.project_label
        ),
        status_commands_json(info.comparison)
    )
}

fn status_commands_json(comparison: &str) -> String {
    format!(
        r#"{{"scorecard":"gemstone-rs compare {} --scorecard","parity":"gemstone-rs compare {} --parity","batches":"gemstone-rs compare {} --batches","totals":"gemstone-rs compare {} --totals"}}"#,
        escape_json(comparison),
        escape_json(comparison),
        escape_json(comparison),
        escape_json(comparison)
    )
}

fn parity_json_entry(comparison: &str, project_label: &str, rows: &[ParityInfo]) -> String {
    format!(
        r#"{{"comparison":"{}",{}}}"#,
        escape_json(comparison),
        parity_json_body(project_label, rows)
    )
}

fn parity_json_body(project_label: &str, rows: &[ParityInfo]) -> String {
    let totals = parity_totals(rows);
    let rows_json = rows
        .iter()
        .map(parity_row_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#""project":"{}","overall":{},"rows":[{}]"#,
        escape_json(project_label),
        parity_totals_json(totals),
        rows_json
    )
}

fn print_next_action(
    title: &str,
    comparison: &str,
    guide: &str,
    batches: &[BatchInfo],
    gaps: &[GapInfo],
    project_label: &str,
    format: OutputFormat,
) {
    let Some(gap) = gaps.first() else {
        return;
    };
    match format {
        OutputFormat::Human => {
            println!("{title}");
            println!("  Full guide: {guide}");
            println!();
            if let Some(batch) = batches.first() {
                println!(
                    "Next batch: {}. {} ({}-{} hours)",
                    batch.number, batch.focus, batch.hours_min, batch.hours_max
                );
                println!("  Outcome: {}", batch.outcome);
                println!("  Verify with: {}", batch.verify_with);
            } else {
                println!("Next batch: none. Active gemstone-rs parity batches are closed.");
            }
            println!();
            println!("Top gap: {} {}", gap.priority, gap.area);
            println!("  gemstone-py strength: {}", gap.gemstone_py_strength);
            println!("  {project_label} gap: {}", gap.gemstone_rs_gap);
            println!("  Next action: {}", gap.next_action);
            println!("  Verify with: {}", gap.verify_with);
        }
        OutputFormat::Json => {
            println!(
                r#"{{"comparison":"{}","view":"next","batch":{},"gap":{}}}"#,
                escape_json(comparison),
                optional_batch_json(batches.first()),
                generic_gap_json(gap, project_label)
            );
        }
    }
}

fn next_action_json_entry(
    comparison: &str,
    batches: &[BatchInfo],
    gaps: &[GapInfo],
    project_label: &str,
) -> String {
    let gap = gaps
        .first()
        .expect("comparison gap reports must contain at least one gap");
    format!(
        r#"{{"comparison":"{}","batch":{},"gap":{}}}"#,
        escape_json(comparison),
        optional_batch_json(batches.first()),
        generic_gap_json(gap, project_label)
    )
}

fn print_batch_plan(
    title: &str,
    comparison: &str,
    view: &str,
    guide: &str,
    batches: &[BatchInfo],
    format: OutputFormat,
) {
    let (min_hours, max_hours) = total_batch_hours(batches);
    match format {
        OutputFormat::Human => {
            println!("{title}");
            println!("  Full guide: {guide}");
            println!(
                "  Total: {} batches, roughly {}-{} hours",
                batches.len(),
                min_hours,
                max_hours
            );
            println!();
            for batch in batches {
                println!(
                    "Batch {}: {} ({}-{} hours)",
                    batch.number, batch.focus, batch.hours_min, batch.hours_max
                );
                println!("  Outcome: {}", batch.outcome);
                println!("  Verify with: {}", batch.verify_with);
                println!();
            }
        }
        OutputFormat::Json => {
            let rows = batches.iter().map(batch_json).collect::<Vec<_>>().join(",");
            println!(
                r#"{{"comparison":"{}","view":"{}","totalBatches":{},"hoursMin":{},"hoursMax":{},"batches":[{}]}}"#,
                escape_json(comparison),
                escape_json(view),
                batches.len(),
                min_hours,
                max_hours,
                rows
            );
        }
    }
}

fn batch_plan_json_entry(comparison: &str, batches: &[BatchInfo]) -> String {
    let (min_hours, max_hours) = total_batch_hours(batches);
    let rows = batches.iter().map(batch_json).collect::<Vec<_>>().join(",");
    format!(
        r#"{{"comparison":"{}","totalBatches":{},"hoursMin":{},"hoursMax":{},"batches":[{}]}}"#,
        escape_json(comparison),
        batches.len(),
        min_hours,
        max_hours,
        rows
    )
}

fn print_batch_totals(title: &str, comparison: &str, batches: &[BatchInfo], format: OutputFormat) {
    let (min_hours, max_hours) = total_batch_hours(batches);
    match format {
        OutputFormat::Human => {
            println!("{title}");
            println!(
                "  Total: {} batches, roughly {}-{} hours",
                batches.len(),
                min_hours,
                max_hours
            );
        }
        OutputFormat::Json => {
            println!(
                r#"{{"comparison":"{}","view":"totals","totalBatches":{},"hoursMin":{},"hoursMax":{}}}"#,
                escape_json(comparison),
                batches.len(),
                min_hours,
                max_hours
            );
        }
    }
}

fn batch_totals_json_entry(comparison: &str, batches: &[BatchInfo]) -> String {
    let (min_hours, max_hours) = total_batch_hours(batches);
    format!(
        r#"{{"comparison":"{}","totalBatches":{},"hoursMin":{},"hoursMax":{}}}"#,
        escape_json(comparison),
        batches.len(),
        min_hours,
        max_hours
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParityTotals {
    gemstone_py_score: u16,
    project_score: u16,
    max_score: u16,
    score_gap: i16,
}

fn parity_totals(rows: &[ParityInfo]) -> ParityTotals {
    let gemstone_py_score = rows
        .iter()
        .map(|row| u16::from(row.gemstone_py_score))
        .sum();
    let project_score = rows.iter().map(|row| u16::from(row.project_score)).sum();
    let max_score = rows.len() as u16 * 5;
    ParityTotals {
        gemstone_py_score,
        project_score,
        max_score,
        score_gap: gemstone_py_score as i16 - project_score as i16,
    }
}

fn parity_totals_json(totals: ParityTotals) -> String {
    format!(
        r#"{{"gemstonePyScore":{},"projectScore":{},"maxScore":{},"scoreGap":{}}}"#,
        totals.gemstone_py_score, totals.project_score, totals.max_score, totals.score_gap
    )
}

fn parity_row_json(row: &ParityInfo) -> String {
    format!(
        r#"{{"area":"{}","gemstonePyScore":{},"projectScore":{},"leader":"{}","status":"{}","nextAction":"{}"}}"#,
        escape_json(row.area),
        row.gemstone_py_score,
        row.project_score,
        escape_json(row.leader),
        escape_json(row.status),
        escape_json(row.next_action)
    )
}

fn remaining_json(batches: &[BatchInfo]) -> String {
    let (min_hours, max_hours) = total_batch_hours(batches);
    format!(
        r#"{{"totalBatches":{},"hoursMin":{},"hoursMax":{}}}"#,
        batches.len(),
        min_hours,
        max_hours
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BatchTotals {
    total_batches: usize,
    hours_min: u16,
    hours_max: u16,
}

fn all_batch_totals() -> BatchTotals {
    let (rs_min, rs_max) = total_batch_hours(GEMSTONE_RS_BATCHES);
    BatchTotals {
        total_batches: GEMSTONE_RS_BATCHES.len(),
        hours_min: rs_min,
        hours_max: rs_max,
    }
}

fn total_batch_hours(batches: &[BatchInfo]) -> (u16, u16) {
    batches.iter().fold((0, 0), |(min, max), batch| {
        (
            min + u16::from(batch.hours_min),
            max + u16::from(batch.hours_max),
        )
    })
}

fn batch_json(batch: &BatchInfo) -> String {
    format!(
        r#"{{"number":{},"focus":"{}","hoursMin":{},"hoursMax":{},"outcome":"{}","verifyWith":"{}"}}"#,
        batch.number,
        escape_json(batch.focus),
        batch.hours_min,
        batch.hours_max,
        escape_json(batch.outcome),
        escape_json(batch.verify_with)
    )
}

fn optional_batch_json(batch: Option<&BatchInfo>) -> String {
    batch.map(batch_json).unwrap_or_else(|| "null".to_string())
}

fn generic_gap_json(gap: &GapInfo, project_label: &str) -> String {
    format!(
        r#"{{"priority":"{}","area":"{}","gemstonePyStrength":"{}","project":"{}","projectGap":"{}","nextAction":"{}","verifyWith":"{}"}}"#,
        escape_json(gap.priority),
        escape_json(gap.area),
        escape_json(gap.gemstone_py_strength),
        escape_json(project_label),
        escape_json(gap.gemstone_rs_gap),
        escape_json(gap.next_action),
        escape_json(gap.verify_with)
    )
}

fn find_example(name: &str) -> Option<&'static ExampleInfo> {
    EXAMPLES.iter().find(|example| {
        example.name.eq_ignore_ascii_case(name) || example.title.eq_ignore_ascii_case(name)
    })
}

fn print_examples_list(format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("gemstone-rs examples");
            println!(
                "Use `gemstone-rs examples show <name>` for details, `examples run <name>` from a source checkout, or `examples map` for feature parity."
            );
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

fn print_feature_map(format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("gemstone-rs feature map");
            println!("  Compare with gemstone-py: docs/gemstone-py-vs-gemstone-rs.md");
            println!("  See also: docs/feature-map.md");
            println!();
            for feature in FEATURE_MAP {
                println!("Stream {}: {}", feature.stream, feature.title);
                println!("  Crates:   {}", feature.crates);
                println!("  Examples: {}", feature.examples);
                println!("  Docs:     {}", feature.docs);
                println!("  gemstone-py reference: {}", feature.gemstone_py_reference);
                println!("  Status:   {}", feature.status);
                println!();
            }
        }
        OutputFormat::Json => {
            let features = FEATURE_MAP
                .iter()
                .map(feature_json)
                .collect::<Vec<_>>()
                .join(",");
            println!(r#"{{"features":[{}]}}"#, features);
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

fn feature_json(feature: &FeatureInfo) -> String {
    format!(
        r#"{{"stream":"{}","title":"{}","crates":"{}","examples":"{}","docs":"{}","gemstonePyReference":"{}","status":"{}"}}"#,
        escape_json(feature.stream),
        escape_json(feature.title),
        escape_json(feature.crates),
        escape_json(feature.examples),
        escape_json(feature.docs),
        escape_json(feature.gemstone_py_reference),
        escape_json(feature.status)
    )
}

fn example_run_command(example: &ExampleInfo, extra_args: &[String]) -> String {
    let mut command = example.command.to_string();
    if !extra_args.is_empty() {
        if example_cli_args(example.name).is_none() {
            command.push_str(" --");
        }
        for arg in extra_args {
            command.push(' ');
            command.push_str(&display_command_arg(arg));
        }
    }
    command
}

fn display_command_arg(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':' | '='))
    {
        value.to_string()
    } else {
        shell_quote(value)
    }
}

fn run_example(
    example: &ExampleInfo,
    dry_run: bool,
    extra_args: &[String],
) -> Result<(), CliError> {
    let command = example_run_command(example, extra_args);
    if dry_run {
        println!("{command}");
        return Ok(());
    }

    println!("running: {command}");
    let mut process;
    if let Some(cli_args) = example_cli_args(example.name) {
        process = ProcessCommand::new(env::current_exe()?);
        process.args(cli_args);
        process.args(extra_args);
    } else if let Some(manifest_path) = example_manifest_path(example.name) {
        process = ProcessCommand::new("cargo");
        process.args(["run", "--manifest-path", manifest_path]);
        if !extra_args.is_empty() {
            process.arg("--");
            process.args(extra_args);
        }
    } else {
        process = ProcessCommand::new("cargo");
        process.args(["run", "-p", "gemstone-rs", "--example", example.name]);
        if !extra_args.is_empty() {
            process.arg("--");
            process.args(extra_args);
        }
    }

    let status = process.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(CliError::Example(format!(
            "example {} exited with {}",
            example.name, status
        )))
    }
}

fn example_cli_args(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "hello" => Some(&["hello"]),
        "py_native_capabilities" => Some(&["py-native", "capabilities"]),
        "py_native_contract_fixture" => Some(&[
            "py-native",
            "check",
            "examples/py-native/gemstone-rs.py-native.json",
        ]),
        "py_native_samples_fixture" => Some(&[
            "py-native",
            "check-samples",
            "examples/py-native/gemstone-rs.py-native-samples.json",
        ]),
        "py_native_smoke_fixture" => Some(&[
            "py-native",
            "check-smoke",
            "examples/py-native/gemstone-rs.py-native-smoke.json",
        ]),
        "py_native_migration_plan" => Some(&["py-native", "migration"]),
        "py_native_compatibility_fixture" => Some(&[
            "py-native",
            "check-compat",
            "examples/py-native/gemstone-rs.py-native-compat.json",
        ]),
        "py_native_conformance_fixture" => Some(&[
            "py-native",
            "check-conformance",
            "examples/py-native/gemstone-rs.py-native-conformance.json",
        ]),
        "py_native_handoff_bundle" => Some(&[
            "py-native",
            "check-handoff",
            "examples/py-native/gemstone-rs.py-native-handoff.json",
        ]),
        "py_native_publish_receipt" => Some(&[
            "py-native",
            "check-publish-receipt",
            "examples/py-native/gemstone-rs.py-native-publish-receipt.json",
        ]),
        "py_native_shared_core_gate" => Some(&["py-native", "check-all"]),
        _ => None,
    }
}

fn example_manifest_path(name: &str) -> Option<&'static str> {
    match name {
        "axum_service" => Some("examples/axum-service/Cargo.toml"),
        "actix_service" => Some("examples/actix-service/Cargo.toml"),
        _ => None,
    }
}

fn find_scaffold_template(name: &str) -> Option<&'static ScaffoldTemplate> {
    let name = match name {
        "actix" | "actix-web" | "actix_web" => "actix_service",
        "axum" | "framework" | "framework_adapter" | "framework-adapter" => "axum_service",
        "bridge" | "mapping" => "bridge_root_mapping",
        "codegen" => "codegen_workflow",
        "discover" => "codegen_discover",
        "discover-mapping" | "discover_mapping" | "mapping-discover" | "mapping_discover" => {
            "codegen_discover_mapping"
        }
        "derive" | "derive-mapping" => "derive_mapping",
        "generated-mapping" | "generated_mapping" => "generated_mapping_app",
        "generated-wrapper" | "generated_wrapper" | "wrapper" => "generated_wrapper_app",
        "http" | "http-service" | "http_service" => "http_service",
        "py-native" | "py_native" | "pynative" | "python-native" | "python_native" | "pyo3" => {
            "py_native_pyo3_adapter"
        }
        "pool" | "session-worker-pool" | "session_worker" | "session-worker" => {
            "session_worker_pool"
        }
        "profile" | "profiles" | "profile-codegen" | "profile_codegen" => {
            "profile_codegen_workflow"
        }
        other => other,
    };
    SCAFFOLD_TEMPLATES
        .iter()
        .find(|template| template.name.eq_ignore_ascii_case(name))
}

fn scaffold_template_names() -> String {
    SCAFFOLD_TEMPLATES
        .iter()
        .map(|template| template.name)
        .collect::<Vec<_>>()
        .join(", ")
}

fn scaffold_example_project(
    template: &ScaffoldTemplate,
    target: &Path,
    force: bool,
) -> Result<(), CliError> {
    let mut files = vec![
        target.join("Cargo.toml"),
        target.join("src").join("main.rs"),
        target.join("README.md"),
        target.join(".gitignore"),
    ];
    files.extend(
        template
            .extra_files
            .iter()
            .map(|file| target.join(file.path)),
    );
    if !force {
        if let Some(existing) = files.iter().find(|path| path.exists()) {
            return Err(CliError::Example(format!(
                "{} already exists; pass --force to overwrite scaffolded files",
                existing.display()
            )));
        }
    }

    fs::create_dir_all(target.join("src"))?;
    write_scaffold_file(&files[0], &scaffold_cargo_toml(template), force)?;
    write_scaffold_file(&files[1], template.main_rs, force)?;
    write_scaffold_file(&files[2], &scaffold_readme(template), force)?;
    write_scaffold_file(&files[3], "target/\n.env\n.env.gemstone-rs\n", force)?;
    for file in template.extra_files {
        write_scaffold_file(&target.join(file.path), file.source, force)?;
    }
    Ok(())
}

fn write_scaffold_file(path: &Path, source: &str, force: bool) -> Result<(), CliError> {
    if path.exists() && !force {
        return Err(CliError::Example(format!(
            "{} already exists; pass --force to overwrite scaffolded files",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, source)?;
    Ok(())
}

fn scaffold_cargo_toml(template: &ScaffoldTemplate) -> String {
    let mut cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
gemstone-rs = "{}"
"#,
        template.package_name,
        env!("CARGO_PKG_VERSION")
    );
    if !template.extra_dependencies.is_empty() {
        let extra_dependencies = template
            .extra_dependencies
            .replace(GEMSTONE_RS_VERSION_TOKEN, env!("CARGO_PKG_VERSION"));
        cargo_toml.push_str(&extra_dependencies);
    }
    cargo_toml
}

fn scaffold_readme(template: &ScaffoldTemplate) -> String {
    format!(
        r#"# {}

{}

## Setup

Set your GemStone/S environment first:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=your-password
```

Then run:

```bash
cargo run
```

This project was generated by:

```bash
gemstone-rs examples scaffold {}
```
"#,
        template.title, template.description, template.name
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

fn optional_field(value: &str) -> &str {
    if value.is_empty() {
        "-"
    } else {
        value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Help,
    Hello {
        format: OutputFormat,
    },
    PyNativeCapabilities {
        format: OutputFormat,
    },
    PyNativeCheck {
        path: PathBuf,
        format: OutputFormat,
    },
    PyNativeSamples {
        format: OutputFormat,
    },
    PyNativeSamplesCheck {
        path: PathBuf,
        format: OutputFormat,
    },
    PyNativeSmokeCheck {
        path: PathBuf,
        format: OutputFormat,
    },
    PyNativeSmoke {
        dry_run: bool,
        format: OutputFormat,
    },
    PyNativeMigration {
        format: OutputFormat,
    },
    PyNativeCompatibility {
        format: OutputFormat,
    },
    PyNativeCompatibilityCheck {
        path: PathBuf,
        format: OutputFormat,
    },
    PyNativeConformance {
        format: OutputFormat,
    },
    PyNativeConformanceCheck {
        path: PathBuf,
        format: OutputFormat,
    },
    PyNativeHandoff {
        format: OutputFormat,
    },
    PyNativeHandoffCheck {
        path: PathBuf,
        format: OutputFormat,
    },
    PyNativePublishReceipt {
        format: OutputFormat,
    },
    PyNativePublishReceiptCheck {
        path: PathBuf,
        format: OutputFormat,
    },
    PyNativeCheckAll {
        root: Option<PathBuf>,
        format: OutputFormat,
    },
    CompareGemstonePy {
        view: CompareView,
        format: OutputFormat,
    },
    CompareGemstoneJs {
        view: CompareView,
        format: OutputFormat,
    },
    CompareAll {
        view: CompareView,
        format: OutputFormat,
    },
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
    ExamplesMap {
        format: OutputFormat,
    },
    ExamplesShow {
        name: String,
        format: OutputFormat,
    },
    ExamplesRun {
        name: String,
        dry_run: bool,
        extra_args: Vec<String>,
    },
    ExamplesScaffold {
        name: String,
        path: Option<PathBuf>,
        force: bool,
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
    BridgeValue {
        root: String,
        key: String,
        key_type: BridgeKeyType,
        depth: usize,
    },
    BridgeShape {
        root: String,
        key: String,
        key_type: BridgeKeyType,
        depth: usize,
    },
    BridgeMappingPreview {
        root: String,
        key: String,
        key_type: BridgeKeyType,
        depth: usize,
        mapped_name: String,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompareView {
    Summary,
    Status,
    Scorecard,
    Parity,
    Gaps,
    Next,
    Totals,
    Batches,
}

fn parse_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::Help);
    };

    match command {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "hello" => parse_hello_command(&args[1..]),
        "py-native" | "py_native" | "pynative" => parse_py_native_command(&args[1..]),
        "compare" => parse_compare_command(&args[1..]),
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

fn parse_hello_command(args: &[String]) -> Result<Command, CliError> {
    parse_format_only_command(args, "hello [--json]", |format| Command::Hello { format })
}

fn parse_py_native_command(args: &[String]) -> Result<Command, CliError> {
    match args.first().map(String::as_str) {
        None | Some("capabilities" | "contract") | Some("--json") => {
            let offset = if matches!(
                args.first().map(String::as_str),
                Some("capabilities" | "contract")
            ) {
                1
            } else {
                0
            };
            parse_format_only_command(
                &args[offset..],
                "py-native capabilities [--json]",
                |format| Command::PyNativeCapabilities { format },
            )
        }
        Some("check" | "validate") => parse_py_native_check_command(&args[1..]),
        Some("samples" | "sample-values" | "values") => parse_format_only_command(
            &args[1..],
            "py-native samples [--json]",
            |format| Command::PyNativeSamples { format },
        ),
        Some("check-samples" | "samples-check" | "validate-samples") => {
            parse_py_native_samples_check_command(&args[1..])
        }
        Some("check-smoke" | "smoke-check" | "validate-smoke") => {
            parse_py_native_smoke_check_command(&args[1..])
        }
        Some("smoke") => parse_py_native_smoke_command(&args[1..]),
        Some("migration" | "plan" | "roadmap" | "checklist") => parse_format_only_command(
            &args[1..],
            "py-native migration [--json]",
            |format| Command::PyNativeMigration { format },
        ),
        Some("compat" | "compatibility" | "api") => parse_format_only_command(
            &args[1..],
            "py-native compatibility [--json]",
            |format| Command::PyNativeCompatibility { format },
        ),
        Some("check-compat" | "compat-check" | "validate-compat") => {
            parse_py_native_compatibility_check_command(&args[1..])
        }
        Some("conformance" | "backend" | "backend-contract") => parse_format_only_command(
            &args[1..],
            "py-native conformance [--json]",
            |format| Command::PyNativeConformance { format },
        ),
        Some("check-conformance" | "conformance-check" | "validate-conformance") => {
            parse_py_native_conformance_check_command(&args[1..])
        }
        Some("handoff" | "handoff-bundle" | "bundle") => parse_format_only_command(
            &args[1..],
            "py-native handoff [--json]",
            |format| Command::PyNativeHandoff { format },
        ),
        Some("check-handoff" | "handoff-check" | "validate-handoff") => {
            parse_py_native_handoff_check_command(&args[1..])
        }
        Some("publish-receipt" | "publish-status" | "publish" | "receipt") => {
            parse_format_only_command(&args[1..], "py-native publish-receipt [--json]", |format| {
                Command::PyNativePublishReceipt { format }
            })
        }
        Some("check-publish-receipt" | "publish-receipt-check" | "validate-publish-receipt") => {
            parse_py_native_publish_receipt_check_command(&args[1..])
        }
        Some("check-all" | "validate-all" | "gate" | "acceptance") => {
            parse_py_native_check_all_command(&args[1..])
        }
        Some("-h" | "--help") => Err(CliError::usage(
            "expected: py-native capabilities [--json] | py-native check [path] [--json] | py-native samples [--json] | py-native check-samples [path] [--json] | py-native check-smoke [path] [--json] | py-native smoke [--dry-run] [--json] | py-native migration [--json] | py-native compatibility [--json] | py-native check-compat [path] [--json] | py-native conformance [--json] | py-native check-conformance [path] [--json] | py-native handoff [--json] | py-native check-handoff [path] [--json] | py-native publish-receipt [--json] | py-native check-publish-receipt [path] [--json] | py-native check-all [--root path] [--json]",
        )),
        Some(command) => Err(CliError::usage(format!(
            "unknown py-native command: {command}; expected capabilities|check|samples|check-samples|check-smoke|smoke|migration|compatibility|check-compat|conformance|check-conformance|handoff|check-handoff|publish-receipt|check-publish-receipt|check-all"
        ))),
    }
}

fn parse_py_native_check_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => {
                return Err(CliError::usage("expected: py-native check [path] [--json]"))
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native check option: {option}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected py-native check argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativeCheck {
        path: path.unwrap_or_else(default_py_native_fixture_path),
        format,
    })
}

fn parse_py_native_samples_check_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: py-native check-samples [path] [--json]",
                ))
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native check-samples option: {option}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected py-native check-samples argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativeSamplesCheck {
        path: path.unwrap_or_else(default_py_native_samples_fixture_path),
        format,
    })
}

fn parse_py_native_compatibility_check_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut path = None;
    for value in args {
        match value.as_str() {
            "--json" => format = OutputFormat::Json,
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native check-compat option: {option}"
                )))
            }
            _ if path.is_none() => path = Some(PathBuf::from(value)),
            _ => {
                return Err(CliError::usage(format!(
                    "unexpected py-native check-compat argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativeCompatibilityCheck {
        path: path.unwrap_or_else(default_py_native_compatibility_fixture_path),
        format,
    })
}

fn parse_py_native_conformance_check_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut path = None;
    for value in args {
        match value.as_str() {
            "--json" => format = OutputFormat::Json,
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native check-conformance option: {option}"
                )))
            }
            _ if path.is_none() => path = Some(PathBuf::from(value)),
            _ => {
                return Err(CliError::usage(format!(
                    "unexpected py-native check-conformance argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativeConformanceCheck {
        path: path.unwrap_or_else(default_py_native_conformance_fixture_path),
        format,
    })
}

fn parse_py_native_handoff_check_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut path = None;
    for value in args {
        match value.as_str() {
            "--json" => format = OutputFormat::Json,
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native check-handoff option: {option}"
                )))
            }
            _ if path.is_none() => path = Some(PathBuf::from(value)),
            _ => {
                return Err(CliError::usage(format!(
                    "unexpected py-native check-handoff argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativeHandoffCheck {
        path: path.unwrap_or_else(default_py_native_handoff_fixture_path),
        format,
    })
}

fn parse_py_native_publish_receipt_check_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut path = None;
    for value in args {
        match value.as_str() {
            "--json" => format = OutputFormat::Json,
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native check-publish-receipt option: {option}"
                )))
            }
            _ if path.is_none() => path = Some(PathBuf::from(value)),
            _ => {
                return Err(CliError::usage(format!(
                    "unexpected py-native check-publish-receipt argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativePublishReceiptCheck {
        path: path.unwrap_or_else(default_py_native_publish_receipt_fixture_path),
        format,
    })
}

fn parse_py_native_check_all_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut root = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                format = OutputFormat::Json;
                index += 1;
            }
            "--root" => {
                let value = args.get(index + 1).ok_or_else(|| {
                    CliError::usage("expected path after py-native check-all --root")
                })?;
                root = Some(PathBuf::from(value));
                index += 2;
            }
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: py-native check-all [--root path] [--json]",
                ))
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native check-all option: {option}"
                )))
            }
            value if root.is_none() => {
                root = Some(PathBuf::from(value));
                index += 1;
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected py-native check-all argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativeCheckAll { root, format })
}

fn parse_py_native_smoke_check_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: py-native check-smoke [path] [--json]",
                ))
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native check-smoke option: {option}"
                )));
            }
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected py-native check-smoke argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativeSmokeCheck {
        path: path.unwrap_or_else(default_py_native_smoke_fixture_path),
        format,
    })
}

fn parse_py_native_smoke_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut dry_run = false;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "--dry-run" => dry_run = true,
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: py-native smoke [--dry-run] [--json]",
                ))
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown py-native smoke option: {option}"
                )));
            }
            value => {
                return Err(CliError::usage(format!(
                    "unexpected py-native smoke argument: {value}"
                )))
            }
        }
    }
    Ok(Command::PyNativeSmoke { dry_run, format })
}

fn parse_compare_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    let mut view = CompareView::Summary;
    let mut target: Option<&str> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "--status" | "--brief" | "--answer" => view = CompareView::Status,
            "--scorecard" | "--matrix" | "--decision" => view = CompareView::Scorecard,
            "--parity" | "--maturity" | "--readiness" => view = CompareView::Parity,
            "--gaps" | "--gap-report" | "--roadmap" => view = CompareView::Gaps,
            "--next" | "--next-action" => view = CompareView::Next,
            "--totals" | "--total" => view = CompareView::Totals,
            "--batches" | "--batch-plan" | "--work" => view = CompareView::Batches,
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: compare gemstone-py|gemstone-js|all [--status|--scorecard|--parity|--gaps|--next|--totals|--batches] [--json]",
                ));
            }
            "gemstone-py" | "gemstone_py" | "py" if target.is_none() => {
                target = Some("gemstone-py");
            }
            "gemstone-js" | "gemstone_js" | "js" if target.is_none() => {
                target = Some("gemstone-js");
            }
            "all" | "everything" if target.is_none() => {
                target = Some("all");
            }
            "status" | "brief" | "answer" => view = CompareView::Status,
            "scorecard" | "matrix" | "decision" => view = CompareView::Scorecard,
            "parity" | "maturity" | "readiness" => view = CompareView::Parity,
            "gaps" | "gap-report" | "roadmap" => view = CompareView::Gaps,
            "next" | "next-action" => view = CompareView::Next,
            "totals" | "total" => view = CompareView::Totals,
            "batches" | "batch-plan" | "work" => view = CompareView::Batches,
            value if value.starts_with('-') => {
                return Err(CliError::usage(format!("unknown compare option: {value}")));
            }
            value => {
                return Err(CliError::usage(format!(
                    "unknown comparison target: {value}; expected gemstone-py, gemstone-js, or all"
                )))
            }
        }
    }

    match target.unwrap_or("gemstone-py") {
        "gemstone-js" => Ok(Command::CompareGemstoneJs { view, format }),
        "all" => Ok(Command::CompareAll { view, format }),
        _ => Ok(Command::CompareGemstonePy { view, format }),
    }
}

fn parse_format_only_command(
    args: &[String],
    usage: &'static str,
    build: impl Fn(OutputFormat) -> Command,
) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => return Err(CliError::usage(format!("expected: {usage}"))),
            other => {
                return Err(CliError::usage(format!(
                    "unexpected argument for {usage}: {other}"
                )))
            }
        }
    }
    Ok(build(format))
}

fn parse_examples_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::ExamplesList {
            format: OutputFormat::Human,
        });
    };
    match command {
        "list" => parse_examples_list_command(&args[1..]),
        "map" | "feature-map" | "plan3-map" => parse_examples_map_command(&args[1..]),
        "hello" => parse_hello_command(&args[1..]),
        "show" => parse_examples_show_command(&args[1..]),
        "run" => parse_examples_run_command(&args[1..]),
        "scaffold" | "copy" => parse_examples_scaffold_command(&args[1..]),
        "--json" => Ok(Command::ExamplesList {
            format: OutputFormat::Json,
        }),
        "-h" | "--help" => Err(CliError::usage(
            "expected: examples [list [--json] | map [--json] | hello [--json] | show <name> [--json] | run <name> [--dry-run] [-- <args>...] | scaffold <name> [path] [--force]]",
        )),
        name => {
            if args.len() > 2 {
                return Err(CliError::usage(
                    "expected: examples [list [--json] | map [--json] | hello [--json] | show <name> [--json] | run <name> [--dry-run] [-- <args>...] | scaffold <name> [path] [--force]]",
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

fn parse_examples_map_command(args: &[String]) -> Result<Command, CliError> {
    let mut format = OutputFormat::Human;
    for arg in args {
        match arg.as_str() {
            "--json" => format = OutputFormat::Json,
            "-h" | "--help" => {
                return Err(CliError::usage("expected: examples map [--json]"));
            }
            other => {
                return Err(CliError::usage(format!(
                    "unexpected examples map argument: {other}"
                )))
            }
        }
    }
    Ok(Command::ExamplesMap { format })
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

fn parse_examples_run_command(args: &[String]) -> Result<Command, CliError> {
    let mut dry_run = false;
    let mut name = None;
    let mut extra_args = Vec::new();
    let mut passthrough = false;

    for arg in args {
        if passthrough {
            extra_args.push(arg.clone());
            continue;
        }
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--" => passthrough = true,
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: examples run <name> [--dry-run] [-- <args>...]",
                ));
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown examples run option: {option}"
                )));
            }
            value if name.is_none() => name = Some(value.to_string()),
            other => {
                return Err(CliError::usage(format!(
                    "unexpected examples run argument: {other}; pass example arguments after --"
                )))
            }
        }
    }

    Ok(Command::ExamplesRun {
        name: name.ok_or_else(|| CliError::usage("missing example name"))?,
        dry_run,
        extra_args,
    })
}

fn parse_examples_scaffold_command(args: &[String]) -> Result<Command, CliError> {
    let mut name = None;
    let mut path = None;
    let mut force = false;
    for arg in args {
        match arg.as_str() {
            "--force" => force = true,
            "-h" | "--help" => {
                return Err(CliError::usage(
                    "expected: examples scaffold <name> [path] [--force]",
                ));
            }
            option if option.starts_with('-') => {
                return Err(CliError::usage(format!(
                    "unknown examples scaffold option: {option}"
                )));
            }
            value if name.is_none() => name = Some(value.to_string()),
            value if path.is_none() => path = Some(PathBuf::from(value)),
            value => {
                return Err(CliError::usage(format!(
                    "unexpected examples scaffold argument: {value}"
                )));
            }
        }
    }

    Ok(Command::ExamplesScaffold {
        name: name.ok_or_else(|| CliError::usage("missing scaffold example name"))?,
        path,
        force,
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
            "expected: bridge root|keys|get|value|shape|mapping-preview|inspect|put|put-string|put-symbol|put-smallint|put-bool|remove|sample-config",
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
        "value" => {
            let key = args
                .get(1)
                .cloned()
                .ok_or_else(|| CliError::usage("missing key for bridge value"))?;
            let options = parse_bridge_options(&args[2..])?;
            Ok(Command::BridgeValue {
                root: options.root,
                key,
                key_type: options.key_type,
                depth: options.depth.unwrap_or(DEFAULT_BRIDGE_VALUE_DEPTH),
            })
        }
        "shape" | "relationships" => {
            let key = args
                .get(1)
                .cloned()
                .ok_or_else(|| CliError::usage("missing key for bridge shape"))?;
            let options = parse_bridge_options(&args[2..])?;
            Ok(Command::BridgeShape {
                root: options.root,
                key,
                key_type: options.key_type,
                depth: options.depth.unwrap_or(DEFAULT_BRIDGE_VALUE_DEPTH),
            })
        }
        "mapping-preview" | "preview-mapping" => {
            let key = args
                .get(1)
                .cloned()
                .ok_or_else(|| CliError::usage("missing key for bridge mapping-preview"))?;
            let options = parse_bridge_options(&args[2..])?;
            Ok(Command::BridgeMappingPreview {
                root: options.root,
                key,
                key_type: options.key_type,
                depth: options.depth.unwrap_or(DEFAULT_BRIDGE_VALUE_DEPTH),
                mapped_name: options
                    .mapped_name
                    .unwrap_or_else(|| "BookingDraft".to_string()),
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
            "expected: bridge root|keys|get|value|shape|mapping-preview|inspect|put|put-string|put-symbol|put-smallint|put-bool|remove|sample-config",
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
    depth: Option<usize>,
    mapped_name: Option<String>,
}

fn parse_bridge_options(args: &[String]) -> Result<BridgeOptions, CliError> {
    let mut root = DEFAULT_BRIDGE_ROOT.to_string();
    let mut key_type = BridgeKeyType::String;
    let mut value_type = None;
    let mut depth = None;
    let mut mapped_name = None;
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
            "--depth" => {
                index += 1;
                depth =
                    Some(parse_bridge_depth(args.get(index).ok_or_else(|| {
                        CliError::usage("missing value for --depth")
                    })?)?);
            }
            option if option.starts_with("--depth=") => {
                let (_, value) = option.split_once('=').unwrap_or_default();
                if value.is_empty() {
                    return Err(CliError::usage("missing value for --depth="));
                }
                depth = Some(parse_bridge_depth(value)?);
            }
            "--mapped" | "--mapped-name" => {
                index += 1;
                mapped_name = Some(
                    args.get(index)
                        .cloned()
                        .ok_or_else(|| CliError::usage("missing value for --mapped"))?,
                );
            }
            option if option.starts_with("--mapped=") || option.starts_with("--mapped-name=") => {
                let (name, value) = option.split_once('=').unwrap_or_default();
                if value.is_empty() {
                    return Err(CliError::usage(format!("missing value for {name}=")));
                }
                mapped_name = Some(value.to_string());
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
        depth,
        mapped_name,
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

fn parse_bridge_depth(value: &str) -> Result<usize, CliError> {
    value
        .trim()
        .parse::<usize>()
        .map(|depth| depth.min(16))
        .map_err(|_| CliError::usage(format!("expected bridge depth integer, got {value}")))
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
  gemstone-rs hello [--json]
  gemstone-rs py-native capabilities [--json]
  gemstone-rs py-native check [path] [--json]
  gemstone-rs py-native samples [--json]
  gemstone-rs py-native check-samples [path] [--json]
  gemstone-rs py-native check-smoke [path] [--json]
  gemstone-rs py-native smoke [--dry-run] [--json]
  gemstone-rs py-native migration [--json]
  gemstone-rs py-native compatibility [--json]
  gemstone-rs py-native check-compat [path] [--json]
  gemstone-rs py-native conformance [--json]
  gemstone-rs py-native check-conformance [path] [--json]
  gemstone-rs py-native handoff [--json]
  gemstone-rs py-native check-handoff [path] [--json]
  gemstone-rs py-native publish-receipt [--json]
  gemstone-rs py-native check-publish-receipt [path] [--json]
  gemstone-rs py-native check-all [--root path] [--json]
  gemstone-rs compare gemstone-py|gemstone-js|all [--status|--scorecard|--parity|--gaps|--next|--totals|--batches] [--json]
  gemstone-rs doctor [--env-file <path>] [--live] [--strict] [--json]
  gemstone-rs env sample
  gemstone-rs env write [path] [--force]
  gemstone-rs examples list [--json]
  gemstone-rs examples map [--json]
  gemstone-rs examples hello [--json]
  gemstone-rs examples show <name> [--json]
  gemstone-rs examples run <name> [--dry-run] [-- <args>...]
  gemstone-rs examples scaffold <name> [path] [--force]
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
  gemstone-rs bridge value <key> [--depth <n>] [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge shape <key> [--depth <n>] [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge mapping-preview <key> [--mapped <name>] [--depth <n>] [--symbol|--string|--key-type String|Symbol] [--root <name>]
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
    Example(String),
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
            Self::Example(message) => write!(f, "{message}"),
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
            Self::CodegenCheck(_) | Self::Example(_) | Self::Doctor(_) | Self::EnvFile(_) => None,
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
    fn parses_hello_and_comparison_commands() {
        assert_eq!(
            parse_command(&args(&["hello"])).unwrap(),
            Command::Hello {
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["hello", "--json"])).unwrap(),
            Command::Hello {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "capabilities"])).unwrap(),
            Command::PyNativeCapabilities {
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "--json"])).unwrap(),
            Command::PyNativeCapabilities {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py_native", "contract", "--json"])).unwrap(),
            Command::PyNativeCapabilities {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "check"])).unwrap(),
            Command::PyNativeCheck {
                path: default_py_native_fixture_path(),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "py-native",
                "validate",
                "examples/py-native/gemstone-rs.py-native.json",
                "--json"
            ]))
            .unwrap(),
            Command::PyNativeCheck {
                path: PathBuf::from("examples/py-native/gemstone-rs.py-native.json"),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "samples", "--json"])).unwrap(),
            Command::PyNativeSamples {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "check-samples"])).unwrap(),
            Command::PyNativeSamplesCheck {
                path: default_py_native_samples_fixture_path(),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "py-native",
                "validate-samples",
                "examples/py-native/gemstone-rs.py-native-samples.json",
                "--json"
            ]))
            .unwrap(),
            Command::PyNativeSamplesCheck {
                path: PathBuf::from("examples/py-native/gemstone-rs.py-native-samples.json"),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "check-smoke"])).unwrap(),
            Command::PyNativeSmokeCheck {
                path: default_py_native_smoke_fixture_path(),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "py-native",
                "smoke-check",
                "examples/py-native/gemstone-rs.py-native-smoke.json",
                "--json"
            ]))
            .unwrap(),
            Command::PyNativeSmokeCheck {
                path: PathBuf::from("examples/py-native/gemstone-rs.py-native-smoke.json"),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "smoke", "--dry-run", "--json"])).unwrap(),
            Command::PyNativeSmoke {
                dry_run: true,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "migration", "--json"])).unwrap(),
            Command::PyNativeMigration {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "checklist"])).unwrap(),
            Command::PyNativeMigration {
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "compatibility", "--json"])).unwrap(),
            Command::PyNativeCompatibility {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "check-compat"])).unwrap(),
            Command::PyNativeCompatibilityCheck {
                path: default_py_native_compatibility_fixture_path(),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "py-native",
                "validate-compat",
                "examples/py-native/gemstone-rs.py-native-compat.json",
                "--json"
            ]))
            .unwrap(),
            Command::PyNativeCompatibilityCheck {
                path: PathBuf::from("examples/py-native/gemstone-rs.py-native-compat.json"),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "conformance", "--json"])).unwrap(),
            Command::PyNativeConformance {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "check-conformance"])).unwrap(),
            Command::PyNativeConformanceCheck {
                path: default_py_native_conformance_fixture_path(),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "py-native",
                "validate-conformance",
                "examples/py-native/gemstone-rs.py-native-conformance.json",
                "--json"
            ]))
            .unwrap(),
            Command::PyNativeConformanceCheck {
                path: PathBuf::from("examples/py-native/gemstone-rs.py-native-conformance.json"),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "handoff", "--json"])).unwrap(),
            Command::PyNativeHandoff {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "check-handoff"])).unwrap(),
            Command::PyNativeHandoffCheck {
                path: default_py_native_handoff_fixture_path(),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "py-native",
                "validate-handoff",
                "examples/py-native/gemstone-rs.py-native-handoff.json",
                "--json"
            ]))
            .unwrap(),
            Command::PyNativeHandoffCheck {
                path: PathBuf::from("examples/py-native/gemstone-rs.py-native-handoff.json"),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "publish-receipt", "--json"])).unwrap(),
            Command::PyNativePublishReceipt {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "check-publish-receipt"])).unwrap(),
            Command::PyNativePublishReceiptCheck {
                path: default_py_native_publish_receipt_fixture_path(),
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "py-native",
                "validate-publish-receipt",
                "examples/py-native/gemstone-rs.py-native-publish-receipt.json",
                "--json"
            ]))
            .unwrap(),
            Command::PyNativePublishReceiptCheck {
                path: PathBuf::from(
                    "examples/py-native/gemstone-rs.py-native-publish-receipt.json"
                ),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["py-native", "check-all", "--json"])).unwrap(),
            Command::PyNativeCheckAll {
                root: None,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "py-native",
                "gate",
                "--root",
                "/tmp/gemstone-rs-check",
                "--json"
            ]))
            .unwrap(),
            Command::PyNativeCheckAll {
                root: Some(PathBuf::from("/tmp/gemstone-rs-check")),
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["examples", "hello", "--json"])).unwrap(),
            Command::Hello {
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Summary,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "--json"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Summary,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py", "--gaps", "--json"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Gaps,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py", "--scorecard", "--json"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Scorecard,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py", "--status", "--json"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Status,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py", "--parity", "--json"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Parity,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py", "gaps"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Gaps,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py", "--batches"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Batches,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py", "--next"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Next,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-py", "--totals"])).unwrap(),
            Command::CompareGemstonePy {
                view: CompareView::Totals,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "gemstone-js"])).unwrap(),
            Command::CompareGemstoneJs {
                view: CompareView::Summary,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "js", "--gaps", "--json"])).unwrap(),
            Command::CompareGemstoneJs {
                view: CompareView::Gaps,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "js", "next", "--json"])).unwrap(),
            Command::CompareGemstoneJs {
                view: CompareView::Next,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "js", "batches", "--json"])).unwrap(),
            Command::CompareGemstoneJs {
                view: CompareView::Batches,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "all", "--next", "--json"])).unwrap(),
            Command::CompareAll {
                view: CompareView::Next,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "all", "scorecard"])).unwrap(),
            Command::CompareAll {
                view: CompareView::Scorecard,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "all", "brief"])).unwrap(),
            Command::CompareAll {
                view: CompareView::Status,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "all", "maturity", "--json"])).unwrap(),
            Command::CompareAll {
                view: CompareView::Parity,
                format: OutputFormat::Json,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "everything", "batches"])).unwrap(),
            Command::CompareAll {
                view: CompareView::Batches,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["compare", "all", "totals", "--json"])).unwrap(),
            Command::CompareAll {
                view: CompareView::Totals,
                format: OutputFormat::Json,
            }
        );
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
            parse_command(&args(&["examples", "map"])).unwrap(),
            Command::ExamplesMap {
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["examples", "plan3-map", "--json"])).unwrap(),
            Command::ExamplesMap {
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
        assert_eq!(
            parse_command(&args(&["examples", "run", "quickstart", "--dry-run"])).unwrap(),
            Command::ExamplesRun {
                name: "quickstart".to_string(),
                dry_run: true,
                extra_args: Vec::new(),
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "examples",
                "run",
                "codegen_workflow",
                "--dry-run",
                "--",
                "--demo",
                "value with spaces"
            ]))
            .unwrap(),
            Command::ExamplesRun {
                name: "codegen_workflow".to_string(),
                dry_run: true,
                extra_args: vec!["--demo".to_string(), "value with spaces".to_string()],
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "examples",
                "scaffold",
                "quickstart",
                "/tmp/gemstone-rs-quickstart",
                "--force"
            ]))
            .unwrap(),
            Command::ExamplesScaffold {
                name: "quickstart".to_string(),
                path: Some(PathBuf::from("/tmp/gemstone-rs-quickstart")),
                force: true,
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
        assert_eq!(
            example_run_command(workflow, &[]),
            "cargo run -p gemstone-rs --example codegen_workflow"
        );
        assert_eq!(
            example_run_command(workflow, &["--flag".to_string(), "two words".to_string()]),
            "cargo run -p gemstone-rs --example codegen_workflow -- --flag 'two words'"
        );

        let async_worker = find_example("async_worker").unwrap();
        assert!(async_worker.requires_live);
        assert!(async_worker.description.contains("Awaitable"));

        let py_native = find_example("python_native_adapter").unwrap();
        assert!(py_native.requires_live);
        assert!(py_native.description.contains("py_native contract"));

        let py_native_fixture = find_example("py-native smoke fixture").unwrap();
        assert!(!py_native_fixture.requires_live);
        assert_eq!(
            example_cli_args(py_native_fixture.name),
            Some(
                &[
                    "py-native",
                    "check-smoke",
                    "examples/py-native/gemstone-rs.py-native-smoke.json",
                ][..]
            )
        );
        assert_eq!(
            example_run_command(py_native_fixture, &[]),
            "gemstone-rs py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json"
        );
        assert_eq!(
            example_run_command(py_native_fixture, &["--json".to_string()]),
            "gemstone-rs py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json --json"
        );
        let py_native_samples = find_example("py-native samples fixture").unwrap();
        assert!(!py_native_samples.requires_live);
        assert_eq!(
            example_cli_args(py_native_samples.name),
            Some(
                &[
                    "py-native",
                    "check-samples",
                    "examples/py-native/gemstone-rs.py-native-samples.json",
                ][..]
            )
        );
        assert_eq!(
            example_run_command(py_native_samples, &[]),
            "gemstone-rs py-native check-samples examples/py-native/gemstone-rs.py-native-samples.json"
        );
        let py_native_compat = find_example("py-native compatibility fixture").unwrap();
        assert!(!py_native_compat.requires_live);
        assert_eq!(
            example_cli_args(py_native_compat.name),
            Some(
                &[
                    "py-native",
                    "check-compat",
                    "examples/py-native/gemstone-rs.py-native-compat.json",
                ][..]
            )
        );
        assert_eq!(
            example_run_command(py_native_compat, &[]),
            "gemstone-rs py-native check-compat examples/py-native/gemstone-rs.py-native-compat.json"
        );
        let py_native_conformance = find_example("py-native conformance fixture").unwrap();
        assert!(!py_native_conformance.requires_live);
        assert_eq!(
            example_cli_args(py_native_conformance.name),
            Some(
                &[
                    "py-native",
                    "check-conformance",
                    "examples/py-native/gemstone-rs.py-native-conformance.json",
                ][..]
            )
        );
        assert_eq!(
            example_run_command(py_native_conformance, &[]),
            "gemstone-rs py-native check-conformance examples/py-native/gemstone-rs.py-native-conformance.json"
        );
        let py_native_handoff = find_example("py-native handoff bundle").unwrap();
        assert!(!py_native_handoff.requires_live);
        assert_eq!(
            example_cli_args(py_native_handoff.name),
            Some(
                &[
                    "py-native",
                    "check-handoff",
                    "examples/py-native/gemstone-rs.py-native-handoff.json",
                ][..]
            )
        );
        assert_eq!(
            example_run_command(py_native_handoff, &[]),
            "gemstone-rs py-native check-handoff examples/py-native/gemstone-rs.py-native-handoff.json"
        );
        let py_native_gate = find_example("py-native shared-core gate").unwrap();
        assert!(!py_native_gate.requires_live);
        assert_eq!(
            example_cli_args(py_native_gate.name),
            Some(&["py-native", "check-all"][..])
        );
        assert_eq!(
            example_run_command(py_native_gate, &[]),
            "gemstone-rs py-native check-all"
        );
    }

    #[test]
    fn feature_map_compares_rust_streams_with_gemstone_py() {
        assert!(FEATURE_MAP
            .iter()
            .any(|feature| feature.title == "Typed codegen"
                && feature
                    .gemstone_py_reference
                    .contains("gemstone_py.codegen")));
        assert!(FEATURE_MAP
            .iter()
            .any(|feature| feature.title == "Rust web services"
                && feature.status.contains("packaged Axum/Actix adapters")));
        assert!(FEATURE_MAP
            .iter()
            .any(|feature| feature.title == "Safe sessions and transactions"
                && feature.crates.contains("SessionWorker")));
        assert!(FEATURE_MAP
            .iter()
            .any(|feature| feature.title == "Shared native core"
                && feature.crates.contains("py_native")
                && feature.examples.contains("python_native_adapter")));
        assert!(feature_json(&FEATURE_MAP[0]).contains(r#""gemstonePyReference":"#));
    }

    #[test]
    fn framework_examples_use_checked_manifest_runner() {
        let axum = find_example("axum_service").unwrap();
        let actix = find_example("actix_service").unwrap();
        assert_eq!(
            example_run_command(axum, &["--routes".to_string()]),
            "cargo run --manifest-path examples/axum-service/Cargo.toml -- --routes"
        );
        assert_eq!(
            example_run_command(actix, &["--routes".to_string()]),
            "cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes"
        );
        assert_eq!(
            example_manifest_path(axum.name),
            Some("examples/axum-service/Cargo.toml")
        );
        assert_eq!(
            example_manifest_path(actix.name),
            Some("examples/actix-service/Cargo.toml")
        );
    }

    #[test]
    fn comparison_rows_include_clear_gemstone_py_gaps() {
        assert!(GEMSTONE_PY_COMPARISON
            .iter()
            .any(|row| row.topic == "Web frameworks"
                && row.gemstone_py.contains("FastAPI")
                && row.gemstone_rs.contains("packaged Axum/Actix adapters")));
        assert!(comparison_json(&GEMSTONE_PY_COMPARISON[0]).contains(r#""gemstonePy":"#));
    }

    #[test]
    fn comparison_rows_include_clear_gemstone_js_gaps() {
        assert!(GEMSTONE_JS_COMPARISON.iter().any(|row| {
            row.topic == "Web frameworks"
                && row.gemstone_py.contains("FastAPI")
                && row.gemstone_js.contains("Express")
        }));
        assert!(GEMSTONE_JS_GAPS.iter().any(|gap| {
            gap.priority == "P1"
                && gap.area == "Visual tooling"
                && gap.gemstone_py_strength.contains("database-explorer")
                && gap.next_action.contains("explorer")
        }));
        assert!(js_comparison_json(&GEMSTONE_JS_COMPARISON[0]).contains(r#""gemstoneJs":"#));
        assert!(js_gap_json(&GEMSTONE_JS_GAPS[0]).contains(r#""gemstoneJsGap":"#));
    }

    #[test]
    fn comparison_batch_plans_are_actionable() {
        assert_eq!(GEMSTONE_RS_BATCHES.len(), 0);
        assert_eq!(GEMSTONE_JS_BATCHES.len(), 6);
        assert_eq!(total_batch_hours(GEMSTONE_RS_BATCHES), (0, 0));
        assert_eq!(total_batch_hours(GEMSTONE_JS_BATCHES), (42, 72));
        assert_eq!(
            all_batch_totals(),
            BatchTotals {
                total_batches: 0,
                hours_min: 0,
                hours_max: 0,
            }
        );
        assert!(GEMSTONE_JS_BATCHES
            .iter()
            .any(|batch| batch.focus.contains("Visual tooling")));
        assert!(batch_json(&GEMSTONE_JS_BATCHES[0]).contains(r#""hoursMax":10"#));
        assert!(generic_gap_json(&GEMSTONE_JS_GAPS[0], "gemstone-js")
            .contains(r#""project":"gemstone-js""#));
        assert!(!generic_gap_json(&GEMSTONE_PY_GAPS[0], "gemstone-rs").contains(r#""view":"#));
        assert!(next_action_json_entry(
            "gemstone-js",
            GEMSTONE_JS_BATCHES,
            GEMSTONE_JS_GAPS,
            "gemstone-js"
        )
        .contains(r#""comparison":"gemstone-js""#));
        assert!(batch_plan_json_entry("gemstone-py", GEMSTONE_RS_BATCHES)
            .contains(r#""totalBatches":0"#));
        assert!(batch_totals_json_entry("gemstone-js", GEMSTONE_JS_BATCHES)
            .contains(r#""hoursMax":72"#));
        assert!(scorecard_json_entry(gemstone_py_scorecard_info())
            .contains(r#""remaining":{"totalBatches":0,"hoursMin":0,"hoursMax":0}"#));
        assert!(scorecard_json_entry(gemstone_py_scorecard_info()).contains(r#""nextBatch":null"#));
        assert!(
            !status_json_entry(gemstone_py_scorecard_info(), GEMSTONE_RS_PARITY)
                .contains(r#""view":"#)
        );
        assert!(
            status_json_body(gemstone_py_scorecard_info(), GEMSTONE_RS_PARITY)
                .contains(r#""scoreGap":0"#)
        );
        assert_eq!(
            parity_totals(GEMSTONE_RS_PARITY),
            ParityTotals {
                gemstone_py_score: 30,
                project_score: 30,
                max_score: 35,
                score_gap: 0,
            }
        );
        assert!(
            !parity_json_entry("gemstone-py", "gemstone-rs", GEMSTONE_RS_PARITY)
                .contains(r#""view":"#)
        );
        assert!(parity_json_body("gemstone-rs", GEMSTONE_RS_PARITY)
            .contains(r#""project":"gemstone-rs""#));
    }

    #[test]
    fn gap_report_prioritizes_actionable_gemstone_py_gaps() {
        assert!(!GEMSTONE_PY_GAPS
            .iter()
            .any(|gap| gap.area == "Shared native core live/publish verification"));
        assert!(GEMSTONE_PY_GAPS.iter().any(|gap| {
            gap.priority == "P2"
                && gap.area == "Web framework adapters"
                && gap.gemstone_rs_gap.contains("async health handlers")
        }));
        assert!(GEMSTONE_PY_GAPS
            .iter()
            .any(|gap| gap.area == "Explorer product polish"
                && gap.gemstone_rs_gap.contains("structured setup checks")
                && gap.next_action.contains("no active parity batch")));
        assert!(gap_json(&GEMSTONE_PY_GAPS[0]).contains(r#""nextAction":"#));
    }

    #[test]
    fn scaffold_writes_runnable_project_shape_and_refuses_overwrite() {
        for name in [
            "quickstart",
            "browser",
            "bridge_root_mapping",
            "derive_mapping",
            "codegen_preview",
            "codegen_workflow",
            "codegen_discover",
            "codegen_discover_mapping",
            "profile_codegen_workflow",
            "generated_wrapper_app",
            "generated_mapping_app",
            "http_service",
            "session_worker_pool",
            "py_native_pyo3_adapter",
            "axum_service",
            "actix_service",
        ] {
            assert!(
                find_scaffold_template(name).is_some(),
                "missing scaffold template {name}"
            );
        }
        assert!(scaffold_template_names().contains("codegen_workflow"));
        assert_eq!(
            find_scaffold_template("codegen").unwrap().name,
            "codegen_workflow"
        );
        assert_eq!(
            find_scaffold_template("bridge").unwrap().name,
            "bridge_root_mapping"
        );
        assert_eq!(
            find_scaffold_template("wrapper").unwrap().name,
            "generated_wrapper_app"
        );
        assert_eq!(
            find_scaffold_template("framework").unwrap().name,
            "axum_service"
        );
        assert_eq!(
            find_scaffold_template("pool").unwrap().name,
            "session_worker_pool"
        );
        assert_eq!(
            find_scaffold_template("actix").unwrap().name,
            "actix_service"
        );
        assert_eq!(
            find_scaffold_template("discover").unwrap().name,
            "codegen_discover"
        );
        assert_eq!(
            find_scaffold_template("profiles").unwrap().name,
            "profile_codegen_workflow"
        );
        assert_eq!(
            find_scaffold_template("py-native").unwrap().name,
            "py_native_pyo3_adapter"
        );

        let target =
            std::env::temp_dir().join(format!("gemstone-rs-scaffold-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&target);
        let template = find_scaffold_template("quickstart").unwrap();

        scaffold_example_project(template, &target, false).unwrap();
        let cargo_toml = fs::read_to_string(target.join("Cargo.toml")).unwrap();
        let main_rs = fs::read_to_string(target.join("src").join("main.rs")).unwrap();
        let readme = fs::read_to_string(target.join("README.md")).unwrap();

        assert!(cargo_toml.contains(r#"name = "gemstone-rs-quickstart""#));
        assert!(cargo_toml.contains(r#"gemstone-rs = ""#));
        assert!(main_rs.contains("Session::login(Config::from_env()?)"));
        assert!(readme.contains("gemstone-rs examples scaffold quickstart"));

        let axum = find_scaffold_template("axum").unwrap();
        let axum_toml = scaffold_cargo_toml(axum);
        assert!(axum_toml.contains(r#"name = "gemstone-rs-axum-service""#));
        assert!(axum_toml.contains(r#"axum = "0.8""#));
        assert!(axum_toml.contains(&format!(
            r#"gemstone-rs-axum = "{}""#,
            env!("CARGO_PKG_VERSION")
        )));
        assert!(axum_toml.contains(r#"tokio = { version = "1""#));

        let actix = find_scaffold_template("actix").unwrap();
        let actix_toml = scaffold_cargo_toml(actix);
        assert!(actix_toml.contains(r#"name = "gemstone-rs-actix-service""#));
        assert!(actix_toml.contains(r#"actix-web = "4""#));
        assert!(actix_toml.contains(&format!(
            r#"gemstone-rs-actix = "{}""#,
            env!("CARGO_PKG_VERSION")
        )));

        let py_native = find_scaffold_template("py_native_pyo3_adapter").unwrap();
        let py_native_toml = scaffold_cargo_toml(py_native);
        assert!(py_native_toml.contains(r#"name = "gemstone-py-native-starter""#));
        assert!(py_native_toml.contains(r#"pyo3 = "0.28""#));
        assert!(py_native_toml.contains(r#"extension-module = ["pyo3/extension-module"]"#));
        assert!(py_native_toml.contains(r#"name = "gemstone_py_native""#));
        assert!(py_native.main_rs.contains("migration_json"));
        assert!(py_native.main_rs.contains("compatibility_json"));
        assert!(py_native.main_rs.contains("conformance_json"));
        assert!(py_native.main_rs.contains("handoff_json"));
        assert!(py_native.extra_files.iter().any(|file| {
            file.path == "src/lib.rs"
                && file.source.contains("fn migration_json()")
                && file.source.contains("fn compatibility_json()")
                && file.source.contains("fn conformance_json()")
                && file.source.contains("fn handoff_json()")
                && file.source.contains("migration_report().to_json()")
                && file.source.contains("compatibility_report().to_json()")
                && file.source.contains("conformance_report().to_json()")
                && file.source.contains("handoff_report().to_json()")
                && file.source.contains("fn eval_json(")
                && file.source.contains("fn value_to_oop_symbol(")
                && file.source.contains("fn perform_raw_oop(")
                && file.source.contains("fn perform_json(")
                && file.source.contains("fn new_symbol(")
                && file.source.contains("fn global_put_string(")
                && file.source.contains("fn commit(")
        }));
        assert!(py_native.extra_files.iter().any(|file| {
            file.path == "tests/test_smoke.py"
                && file
                    .source
                    .contains("test_migration_json_tracks_python_wrapper_work")
                && file
                    .source
                    .contains("test_native_session_exposes_core_adapter_methods")
                && file
                    .source
                    .contains("test_compatibility_report_documents_return_policy")
                && file
                    .source
                    .contains("test_conformance_json_tracks_backend_surface")
                && file
                    .source
                    .contains("test_handoff_json_tracks_downstream_acceptance")
        }));
        assert!(py_native.extra_files.iter().any(|file| {
            file.path == "python/gemstone_py_native_compat.py"
                && file.source.contains("class NativeCompatibilitySession")
                && file.source.contains("class OopHandle")
                && file.source.contains("def eval_value")
                && file.source.contains("def perform_value")
                && file.source.contains("def value_to_oop_symbol")
                && file.source.contains("def compatibility_report")
        }));

        let err = scaffold_example_project(template, &target, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("already exists"));
        scaffold_example_project(template, &target, true).unwrap();

        let profile_target = target.with_file_name(format!(
            "gemstone-rs-profile-scaffold-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&profile_target);
        let profile_template = find_scaffold_template("profile_codegen_workflow").unwrap();
        scaffold_example_project(profile_template, &profile_target, false).unwrap();
        assert!(profile_target.join("gemstone-rs.codegen").exists());
        assert!(profile_target
            .join("gemstone-rs.codegen-profiles.json")
            .exists());
        let profile_main = fs::read_to_string(profile_target.join("src").join("main.rs")).unwrap();
        assert!(profile_main.contains("profiles::load_file"));
        let _ = fs::remove_dir_all(&profile_target);

        let py_native_target = target.with_file_name(format!(
            "gemstone-rs-py-native-scaffold-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&py_native_target);
        scaffold_example_project(py_native, &py_native_target, false).unwrap();
        assert!(py_native_target.join("pyproject.toml").exists());
        assert!(py_native_target.join("PYTHON.md").exists());
        assert!(py_native_target
            .join("python")
            .join("gemstone_py_native_compat.py")
            .exists());
        assert!(py_native_target
            .join("tests")
            .join("test_smoke.py")
            .exists());
        let lib_rs = fs::read_to_string(py_native_target.join("src").join("lib.rs")).unwrap();
        assert!(lib_rs.contains("PyNativeSession::login_from_env"));
        assert!(lib_rs.contains("samples_json"));
        assert!(lib_rs.contains("compatibility_json"));
        assert!(lib_rs.contains("conformance_json"));
        assert!(lib_rs.contains("handoff_json"));
        assert!(lib_rs.contains("fn eval_json"));
        assert!(lib_rs.contains("fn value_to_oop_symbol"));
        assert!(lib_rs.contains("fn perform_json"));
        assert!(lib_rs.contains("#[pyclass(unsendable)]"));
        let compat_py = fs::read_to_string(
            py_native_target
                .join("python")
                .join("gemstone_py_native_compat.py"),
        )
        .unwrap();
        assert!(compat_py.contains("compatibility_json"));
        assert!(compat_py.contains("def raw_oop"));
        assert!(compat_py.contains("def eval_value"));
        assert!(compat_py.contains("def perform_value"));
        let _ = fs::remove_dir_all(&py_native_target);
        let _ = fs::remove_dir_all(&target);
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
                "value",
                "BookingDraft",
                "--depth=4",
                "--root",
                "DemoRoot"
            ]))
            .unwrap(),
            Command::BridgeValue {
                root: "DemoRoot".to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::String,
                depth: 4,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "shape",
                "BookingDraft",
                "--depth=3",
                "--root=DemoRoot"
            ]))
            .unwrap(),
            Command::BridgeShape {
                root: "DemoRoot".to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::String,
                depth: 3,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "mapping-preview",
                "BookingDraft",
                "--mapped=BookingDraft",
                "--depth",
                "5",
                "--symbol",
                "--root",
                "DemoRoot"
            ]))
            .unwrap(),
            Command::BridgeMappingPreview {
                root: "DemoRoot".to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::Symbol,
                depth: 5,
                mapped_name: "BookingDraft".to_string(),
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

    #[test]
    fn py_native_capabilities_json_is_stable() {
        let json = py_native::capabilities().to_json();
        assert!(json.contains(r#""contractVersion":1"#));
        assert!(json.contains(r#""operations":["login","logout","eval""#));
        assert!(json.contains(r#""valueKinds":["nil","bool","smallInt""#));
        assert!(json.contains(r#""errorKinds":["gci","missingEnvironment""#));
        assert!(json.contains(r#""oopConstants":"#));
    }

    #[test]
    fn py_native_check_detects_contract_drift() {
        let path = std::env::temp_dir().join(format!(
            "gemstone-rs-py-native-check-{}.json",
            std::process::id()
        ));
        fs::write(&path, py_native::capabilities().to_json()).unwrap();
        run_py_native_check(&path, OutputFormat::Human).unwrap();

        fs::write(&path, r#"{"name":"wrong"}"#).unwrap();
        let err = run_py_native_check(&path, OutputFormat::Human)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn py_native_smoke_check_detects_contract_drift() {
        let path = std::env::temp_dir().join(format!(
            "gemstone-rs-py-native-smoke-check-{}.json",
            std::process::id()
        ));
        fs::write(&path, py_native::smoke_dry_run_report().to_json()).unwrap();
        run_py_native_smoke_check(&path, OutputFormat::Human).unwrap();

        fs::write(&path, r#"{"name":"wrong"}"#).unwrap();
        let err = run_py_native_smoke_check(&path, OutputFormat::Human)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn py_native_samples_check_detects_contract_drift() {
        let path = std::env::temp_dir().join(format!(
            "gemstone-rs-py-native-samples-check-{}.json",
            std::process::id()
        ));
        fs::write(&path, py_native::samples_report().to_json()).unwrap();
        run_py_native_samples_check(&path, OutputFormat::Human).unwrap();

        fs::write(&path, r#"{"contractVersion":999,"values":[],"errors":[]}"#).unwrap();
        let err = run_py_native_samples_check(&path, OutputFormat::Human)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn py_native_compatibility_check_detects_contract_drift() {
        let path = std::env::temp_dir().join(format!(
            "gemstone-rs-py-native-compat-check-{}.json",
            std::process::id()
        ));
        fs::write(&path, py_native::compatibility_report().to_json()).unwrap();
        run_py_native_compatibility_check(&path, OutputFormat::Human).unwrap();

        fs::write(&path, r#"{"contractVersion":999,"methods":[]}"#).unwrap();
        let err = run_py_native_compatibility_check(&path, OutputFormat::Human)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn py_native_conformance_check_detects_contract_drift() {
        let path = std::env::temp_dir().join(format!(
            "gemstone-rs-py-native-conformance-check-{}.json",
            std::process::id()
        ));
        fs::write(&path, py_native::conformance_report().to_json()).unwrap();
        run_py_native_conformance_check(&path, OutputFormat::Human).unwrap();

        fs::write(&path, r#"{"contractVersion":999,"moduleFunctions":[]}"#).unwrap();
        let err = run_py_native_conformance_check(&path, OutputFormat::Human)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn py_native_handoff_check_detects_contract_drift() {
        let path = std::env::temp_dir().join(format!(
            "gemstone-rs-py-native-handoff-check-{}.json",
            std::process::id()
        ));
        fs::write(&path, py_native::handoff_report().to_json()).unwrap();
        run_py_native_handoff_check(&path, OutputFormat::Human).unwrap();

        fs::write(&path, r#"{"contractVersion":999,"artifacts":[]}"#).unwrap();
        let err = run_py_native_handoff_check(&path, OutputFormat::Human)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn py_native_check_all_reports_fixture_drift() {
        let root = std::env::temp_dir().join(format!(
            "gemstone-rs-py-native-check-all-{}",
            std::process::id()
        ));
        let fixture_dir = root.join("examples").join("py-native");
        fs::create_dir_all(&fixture_dir).unwrap();
        fs::write(
            root.join(default_py_native_fixture_path()),
            py_native::capabilities().to_json(),
        )
        .unwrap();
        fs::write(
            root.join(default_py_native_samples_fixture_path()),
            py_native::samples_report().to_json(),
        )
        .unwrap();
        fs::write(
            root.join(default_py_native_smoke_fixture_path()),
            py_native::smoke_dry_run_report().to_json(),
        )
        .unwrap();
        fs::write(
            root.join(default_py_native_compatibility_fixture_path()),
            py_native::compatibility_report().to_json(),
        )
        .unwrap();
        fs::write(
            root.join(default_py_native_conformance_fixture_path()),
            py_native::conformance_report().to_json(),
        )
        .unwrap();
        fs::write(
            root.join(default_py_native_handoff_fixture_path()),
            py_native::handoff_report().to_json(),
        )
        .unwrap();
        fs::write(
            root.join(default_py_native_publish_receipt_fixture_path()),
            py_native::publish_receipt().to_json(),
        )
        .unwrap();

        let report = build_py_native_check_all_report(Some(&root));
        assert!(report.ok());
        assert_eq!(report.ok_count(), 7);
        run_py_native_check_all(Some(&root), OutputFormat::Human).unwrap();

        fs::write(root.join(default_py_native_handoff_fixture_path()), "{}").unwrap();
        let report = build_py_native_check_all_report(Some(&root));
        assert!(!report.ok());
        assert_eq!(report.error_count(), 1);
        assert!(report.to_json().contains(r#""name":"handoff""#));
        let err = run_py_native_check_all(Some(&root), OutputFormat::Human)
            .unwrap_err()
            .to_string();
        assert!(err.contains("shared-core gate failed"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn py_native_smoke_contract_steps_pass_without_live_stone() {
        let report = py_native::smoke_dry_run_report();
        assert!(report.ok());
        assert!(report.dry_run);
        assert_eq!(report.contract_version, 1);
        assert_eq!(report.steps.len(), 5);
        assert!(report
            .steps
            .iter()
            .any(|step| step.name == "value_conversion"));
        assert!(report
            .steps
            .iter()
            .any(|step| step.name == "structured_error_mapping"));
    }

    #[test]
    fn py_native_samples_report_includes_wrapper_payload_shapes() {
        let report = py_native::samples_report();
        let json = report.to_json();
        assert_eq!(report.values.len(), py_native::PY_NATIVE_VALUE_KINDS.len());
        assert!(json.contains(r#""kind":"symbol""#));
        assert!(json.contains(r#""kind":"mapping""#));
    }
}
