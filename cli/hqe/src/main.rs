//! HQE Workbench CLI

use clap::{Parser, Subcommand};
use console::style;
use hqe_core::models::*;
use hqe_core::scan::ScanPipeline;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::Level;
use serde_json::json;

#[derive(Parser)]
#[command(name = "hqe")]
#[command(about = "HQE Engineer Protocol - Codebase Health Scanner")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate the HQE protocol files
    ValidateProtocol,

    /// Execute an MCP Prompt Tool
    Prompt {
        /// Name of the prompt tool to execute
        #[arg(value_name = "TOOL_NAME")]
        name: String,

        /// JSON arguments for the prompt (e.g., '{"arg": "value"}')
        #[arg(short, long, default_value = "{}")]
        args: String,

        /// Provider profile to use
        #[arg(short, long)]
        profile: Option<String>,
    },

    /// Scan a repository
    Scan {
        /// Path to repository
        #[arg(value_name = "REPO_PATH")]
        repo: PathBuf,

        /// Provider profile to use
        #[arg(short, long)]
        profile: Option<String>,

        /// Run in local-only mode (no LLM)
        #[arg(long)]
        local_only: bool,

        /// Output directory for artifacts
        #[arg(short, long, default_value = "./hqe-output")]
        out: PathBuf,

        /// Maximum files to analyze
        #[arg(long)]
        max_files: Option<usize>,

        /// Timeout in seconds for LLM operations
        #[arg(long, default_value = "120")]
        timeout: u64,
    },

    /// Export a specific run
    Export {
        /// Run ID to export
        #[arg(value_name = "RUN_ID")]
        run_id: String,

        /// Output directory
        #[arg(short, long, default_value = "./hqe-exports")]
        out: PathBuf,
    },

    /// Generate or apply patches
    Patch {
        /// Run ID
        #[arg(value_name = "RUN_ID")]
        run_id: String,

        /// TODO ID to patch
        #[arg(short, long)]
        todo: String,

        /// Preview the patch (dry-run)
        #[arg(long)]
        preview: bool,

        /// Apply the patch
        #[arg(long)]
        apply: bool,
    },

    /// Configure provider profiles
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// List configured profiles
    List,

    /// Add a new provider profile
    Add {
        /// Profile name
        name: String,

        /// Base URL
        #[arg(short, long)]
        url: String,

        /// API key (will be stored in keychain)
        #[arg(short, long)]
        key: String,

        /// Default model
        #[arg(short, long, default_value = "gpt-4o-mini")]
        model: String,
    },

    /// Test a provider connection
    Test {
        /// Profile name
        name: String,
    },

    /// Remove a profile
    Remove {
        /// Profile name
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::ValidateProtocol => validate_protocol().await,
        Commands::Prompt {
            name,
            args,
            profile,
        } => handle_prompt(name, args, profile).await,
        Commands::Scan {
            repo,
            profile,
            local_only,
            out,
            max_files,
            timeout,
        } => scan_repo(repo, profile, local_only, out, max_files, timeout).await,
        Commands::Export { run_id, out } => export_run(run_id, out).await,
        Commands::Patch {
            run_id,
            todo,
            preview,
            apply,
        } => handle_patch(run_id, todo, preview, apply).await,
        Commands::Config { command } => handle_config(command).await,
    }
}

async fn handle_prompt(
    tool_name: String,
    args_json: String,
    profile_name: Option<String>,
) -> anyhow::Result<()> {
    println!("{}", style(format!("🤖 Executing Prompt: {}", tool_name)).bold().cyan());

    // 1. Initialize OpenAI Client
    let config_dir = dirs::data_local_dir()
        .map(|d| d.join("hqe-workbench"))
        .unwrap_or_else(|| PathBuf::from("~/.local/share/hqe-workbench"));
    let profiles_path = config_dir.join("profiles.json");

    let client = if profiles_path.exists() {
        let content = tokio::fs::read_to_string(&profiles_path).await?;
        let profiles: Vec<hqe_openai::ProviderProfile> = serde_json::from_str(&content)?;
        
        let profile = if let Some(p_name) = &profile_name {
            profiles.iter().find(|p| &p.name == p_name)
        } else {
            profiles.first()
        };

        if let Some(profile) = profile {
            println!("  Using Profile: {}", profile.name);
            let entry = keyring::Entry::new("hqe-workbench", &profile.api_key_id)?;
            let api_key = entry.get_password()?;
            
            let config = hqe_openai::ClientConfig {
                base_url: profile.base_url.clone(),
                api_key: secrecy::SecretString::new(api_key),
                default_model: profile.default_model.clone(),
                timeout_seconds: 120, // Longer timeout for detailed prompts
                max_retries: 1,
                rate_limit_config: None,
            };
            Some(hqe_openai::OpenAIClient::new(config)?)
        } else {
            None
        }
    } else {
        None
    };

    let client = client.ok_or_else(|| anyhow::anyhow!("No provider profile found. Use 'hqe config add' to configure one."))?;
    let client = std::sync::Arc::new(client);

    // 2. Locate Prompts Directory
    let mut prompts_dir = PathBuf::from("./prompts");
    if !prompts_dir.exists() {
        if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
            let try_path = exe_dir.join("prompts");
            if try_path.exists() {
                prompts_dir = try_path;
            } else if let Some(parent) = exe_dir.parent() {
                let try_path_parent = parent.join("prompts");
                if try_path_parent.exists() {
                    prompts_dir = try_path_parent;
                }
            }
        }
    }
    
    if !prompts_dir.exists() {
        return Err(anyhow::anyhow!("Could not locate 'prompts' directory."));
    }
    println!("  Prompts Dir: {}", prompts_dir.display());

    // 3. Load and Register Tools
    let loader = hqe_mcp::PromptLoader::new(&prompts_dir);
    let loaded_tools = loader.load()?;
    let registry = hqe_mcp::ToolRegistry::new();

    for tool in loaded_tools {
        let template = tool.template.clone();
        let client_clone = client.clone();
        
        // Create async execution handler
        let handler = Box::new(move |args: serde_json::Value| -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<serde_json::Value>> + Send>> {
            let template = template.clone();
            let client_clone = client_clone.clone();
            
            Box::pin(async move {
                let prompt_text = substitute_template(&template, &args);
                
                let response = client_clone
                    .chat(hqe_openai::ChatRequest {
                        model: client_clone.default_model().to_string(),
                        messages: vec![
                            hqe_openai::Message {
                                role: hqe_openai::Role::User,
                                content: prompt_text,
                            }
                        ],
                        temperature: Some(0.2),
                        max_tokens: None,
                        response_format: None,
                    })
                    .await?;
                
                Ok(json!({ "result": response.choices[0].message.content }))
            })
        });

        let tool_name = tool.definition.name.clone();
        if let Err(e) = registry.register_tool("prompts", tool.definition, handler).await {
            tracing::warn!("Failed to register tool '{}': {}", tool_name, e);
        }
    }

    // 4. Execute the requested tool
    let args_val: serde_json::Value = serde_json::from_str(&args_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON args: {}", e))?;

    // The registry expects tool names like "topic__name" or just "name" if loaded that way.
    // Our loader registers them under topic "prompts", so the key is "prompts__tool_name".
    // We should try both or standardized format. 
    // Registry implementation: key = format!("{}__{}", topic_id, def.name);
    let lookup_name = if tool_name.contains("__") {
        tool_name
    } else {
        format!("prompts__{}", tool_name)
    };

    println!("  Running tool: {}...", lookup_name);
    let result: serde_json::Value = registry.call_tool(&lookup_name, args_val).await?;

    println!("\n{}", style("📝 Result:").bold().green());
    if let Some(text) = result.get("result").and_then(|v| v.as_str()) {
        println!("{}", text);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

fn substitute_template(template: &str, args: &serde_json::Value) -> String {
    let mut result = template.to_string();
    
    if let Some(obj) = args.as_object() {
        for (k, v) in obj {
            let key = format!("{{{{{}}}}}", k); // {{key}}
            let val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
            result = result.replace(&key, &val);
        }
    }
    
    result
}

// Embed protocol files at compile time for standalone binary distribution
const PROTOCOL_YAML: &str = include_str!("../../../protocol/hqe-engineer.yaml");
const PROTOCOL_SCHEMA: &str = include_str!("../../../protocol/hqe-schema.json");
const PROTOCOL_VERSION: &str = "3.1.0";

async fn validate_protocol() -> anyhow::Result<()> {
    println!("{}", style("🔍 Validating HQE Protocol...").bold());

    // Try to find protocol files in multiple locations
    let mut possible_paths: Vec<PathBuf> = vec![
        // Development: relative to project root
        PathBuf::from("./protocol"),
    ];

    // Installed: relative to executable
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    {
        possible_paths.push(exe_dir.join("protocol"));
        // Installed: one level up for cargo install
        if let Some(parent) = exe_dir.parent() {
            possible_paths.push(parent.join("protocol"));
        }
    }

    // System paths
    possible_paths.push(PathBuf::from("/usr/local/share/hqe-workbench/protocol"));

    // User path
    if let Some(data_dir) = dirs::data_dir() {
        possible_paths.push(data_dir.join("hqe-workbench/protocol"));
    }

    let mut proto_dir: Option<PathBuf> = None;

    for path in possible_paths {
        let yaml = path.join("hqe-engineer.yaml");
        let schema = path.join("hqe-schema.json");

        if yaml.exists() && schema.exists() {
            println!("  Found protocol at: {}", path.display());
            proto_dir = Some(path);
            break;
        }
    }

    // If not found, extract embedded protocol to temp directory
    let temp_proto_dir: Option<(tempfile::TempDir, PathBuf)> = if proto_dir.is_none() {
        println!("{}", style("  Using embedded protocol...").yellow());

        let temp = tempfile::tempdir()?;
        let proto_path = temp.path().join("protocol");
        std::fs::create_dir_all(&proto_path)?;

        tokio::fs::write(proto_path.join("hqe-engineer.yaml"), PROTOCOL_YAML).await?;
        tokio::fs::write(proto_path.join("hqe-schema.json"), PROTOCOL_SCHEMA).await?;

        Some((temp, proto_path.clone()))
    } else {
        None
    };

    let proto_path = proto_dir.unwrap_or_else(|| temp_proto_dir.as_ref().unwrap().1.clone());
    let yaml_path = proto_path.join("hqe-engineer.yaml");
    let schema_path = proto_path.join("hqe-schema.json");

    println!("  YAML: {}", yaml_path.display());
    println!("  Schema: {}", schema_path.display());
    println!("  Version: {}", PROTOCOL_VERSION);

    // Check if Python validator is available
    let python_check = tokio::process::Command::new("python3")
        .arg("--version")
        .output()
        .await;

    if python_check.is_err() {
        // Basic validation without Python
        println!(
            "{}",
            style("⚠️  Python not available, performing basic validation...").yellow()
        );

        let yaml_content = tokio::fs::read_to_string(&yaml_path).await?;
        let schema_content = tokio::fs::read_to_string(&schema_path).await?;

        // Check YAML is valid
        let _: serde_yaml::Value = serde_yaml::from_str(&yaml_content)
            .map_err(|e| anyhow::anyhow!("Invalid YAML: {}", e))?;

        // Check JSON schema is valid
        let _: serde_json::Value = serde_json::from_str(&schema_content)
            .map_err(|e| anyhow::anyhow!("Invalid JSON schema: {}", e))?;

        println!(
            "{}",
            style("\n✅ Basic validation passed (syntax OK)")
                .green()
                .bold()
        );
        println!(
            "{}",
            style("   Note: Install Python with pyyaml and jsonschema for full validation").dim()
        );
        return Ok(());
    }

    // Run Python validator
    let verify_py = proto_path.join("verify.py");

    // If verify.py doesn't exist, write it from embedded content
    if !verify_py.exists() {
        const VERIFY_PY: &str = include_str!("../../../protocol/verify.py");
        tokio::fs::write(&verify_py, VERIFY_PY).await?;
    }

    let output = tokio::process::Command::new("python3")
        .args([
            verify_py.to_str().unwrap(),
            "--yaml",
            yaml_path.to_str().unwrap(),
            "--schema",
            schema_path.to_str().unwrap(),
        ])
        .output()
        .await?;

    if output.status.success() {
        println!(
            "{}",
            style("\n✅ Protocol validation passed").green().bold()
        );
        Ok(())
    } else {
        println!("{}", style("\n❌ Protocol validation failed").red().bold());
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
}

async fn scan_repo(
    repo: PathBuf,
    profile: Option<String>,
    local_only: bool,
    out: PathBuf,
    max_files: Option<usize>,
    timeout: u64,
) -> anyhow::Result<()> {
    println!("{}", style("🔍 HQE Repository Scan").bold().cyan());
    println!("  Repository: {}", repo.display());
    let mode_str = if local_only {
        style("local-only").yellow().to_string()
    } else {
        style(format!("LLM ({})", profile.as_deref().unwrap_or("default")))
            .green()
            .to_string()
    };
    println!("  Mode: {}", mode_str);
    println!("  Timeout: {}s", timeout);
    println!("  Output: {}", out.display());
    println!();

    // Setup progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    // Build config
    let mut limits = ScanLimits::default();
    if let Some(max) = max_files {
        limits.max_files_sent = max;
    }

    let config = ScanConfig {
        llm_enabled: !local_only,
        provider_profile: profile,
        limits,
        local_only,
        timeout_seconds: timeout,
    };

    // Run scan
    pb.set_message("Initializing scan pipeline...");
    let mut pipeline = ScanPipeline::new(&repo, config)?;

    pb.set_message("Phase: Ingestion...");
    let result = pipeline.run().await?;

    pb.finish_with_message("Scan complete!");

    // Write artifacts
    println!("\n{}", style("📁 Writing artifacts...").bold());

    let run_dir = out.join(format!("hqe_run_{}", result.manifest.run_id));
    std::fs::create_dir_all(&run_dir)?;

    let writer = hqe_artifacts::ArtifactWriter::new(&run_dir);
    let paths = writer.write_all(&result).await?;

    // Print summary
    println!("\n{}", style("📊 Scan Summary").bold().green());
    println!("  Run ID: {}", result.manifest.run_id);
    println!(
        "  Health Score: {}/10",
        result.report.executive_summary.health_score
    );
    println!("  TODO Items: {}", result.report.master_todo_backlog.len());

    if !result.report.executive_summary.blockers.is_empty() {
        println!("\n{}", style("⚠️  Blockers:").yellow());
        for blocker in &result.report.executive_summary.blockers {
            println!("  - {}", blocker.description);
        }
    }

    println!("\n{}", style("📄 Artifacts:").bold());
    println!("  {}", paths.manifest_json.display());
    println!("  {}", paths.report_json.display());
    println!("  {}", paths.report_md.display());

    println!("\n{}", style("✅ Done!").green().bold());

    Ok(())
}

async fn export_run(run_id: String, out: PathBuf) -> anyhow::Result<()> {
    println!("{}", style(format!("📦 Exporting run: {}", run_id)).bold());
    println!("  Output: {}", out.display());

    // This would search for the run and export it
    // For now, placeholder
    println!("\n{}", style("Not yet implemented").yellow());

    Ok(())
}

async fn handle_patch(
    run_id: String,
    todo: String,
    preview: bool,
    apply: bool,
) -> anyhow::Result<()> {
    println!(
        "{}",
        style(format!("🔧 Patch: {} for run {}", todo, run_id)).bold()
    );

    if !preview && !apply {
        println!("{}", style("Use --preview or --apply").yellow());
        return Ok(());
    }

    if preview {
        println!("  Mode: {}", style("Preview (dry-run)").cyan());
    }

    if apply {
        println!("  Mode: {}", style("Apply").green());
    }

    // Placeholder
    println!(
        "\n{}",
        style("Patch generation not yet implemented").yellow()
    );

    Ok(())
}

async fn handle_config(command: ConfigCommands) -> anyhow::Result<()> {
    let config_dir = dirs::data_local_dir()
        .map(|d| d.join("hqe-workbench"))
        .unwrap_or_else(|| PathBuf::from("~/.local/share/hqe-workbench"));

    std::fs::create_dir_all(&config_dir)?;
    let profiles_path = config_dir.join("profiles.json");

    match command {
        ConfigCommands::List => {
            println!("{}", style("📋 Provider Profiles").bold());

            if profiles_path.exists() {
                let content = tokio::fs::read_to_string(&profiles_path).await?;
                let profiles: Vec<hqe_openai::ProviderProfile> = serde_json::from_str(&content)?;

                // Validate each profile after deserialization
                for profile in &profiles {
                    profile.validate_base_url().map_err(|e| {
                        anyhow::anyhow!("Invalid profile base URL '{}': {}", profile.name, e)
                    })?;
                    profile.validate_headers().map_err(|e| {
                        anyhow::anyhow!("Invalid profile headers '{}': {}", profile.name, e)
                    })?;
                }

                for profile in profiles {
                    println!("  • {} ({})", profile.name, profile.base_url);
                    println!("    Model: {}", profile.default_model);
                }
            } else {
                println!("  No profiles configured.");
                println!("  Use: hqe config add <name> --url <url> --key <key>");
            }
        }
        ConfigCommands::Add {
            name,
            url,
            key,
            model,
        } => {
            println!(
                "{}",
                style(format!("➕ Adding profile: {}", name)).bold().green()
            );

            // Store API key in keychain
            let entry = keyring::Entry::new("hqe-workbench", &format!("api_key:{}", name))?;
            entry.set_password(&key)?;

            // Load existing profiles
            let mut profiles: Vec<hqe_openai::ProviderProfile> = if profiles_path.exists() {
                let content = tokio::fs::read_to_string(&profiles_path).await?;
                let loaded_profiles: Vec<hqe_openai::ProviderProfile> =
                    serde_json::from_str(&content)?;

                // Validate each profile after deserialization
                for profile in &loaded_profiles {
                    profile.validate_base_url().map_err(|e| {
                        anyhow::anyhow!("Invalid profile base URL '{}': {}", profile.name, e)
                    })?;
                    profile.validate_headers().map_err(|e| {
                        anyhow::anyhow!("Invalid profile headers '{}': {}", profile.name, e)
                    })?;
                }

                loaded_profiles
            } else {
                vec![]
            };

            // Add or update profile using builder pattern
            let profile = hqe_openai::ProviderProfile::new(name.clone(), url)
                .with_model(model);

            profiles.retain(|p| p.name != name);
            profiles.push(profile);

            // Save
            let json = serde_json::to_string_pretty(&profiles)?;
            tokio::fs::write(&profiles_path, json).await?;

            println!("{}", style("✅ Profile saved").green());
        }
        ConfigCommands::Test { name } => {
            println!(
                "{}",
                style(format!("🧪 Testing connection: {}", name)).bold()
            );

            // Load profile
            let content = tokio::fs::read_to_string(&profiles_path).await?;
            let profiles: Vec<hqe_openai::ProviderProfile> = serde_json::from_str(&content)?;

            // Validate each profile after deserialization
            for profile in &profiles {
                profile.validate_base_url().map_err(|e| {
                    anyhow::anyhow!("Invalid profile base URL '{}': {}", profile.name, e)
                })?;
                profile.validate_headers().map_err(|e| {
                    anyhow::anyhow!("Invalid profile headers '{}': {}", profile.name, e)
                })?;
            }

            let profile = profiles.iter().find(|p| p.name == name);

            if let Some(profile) = profile {
                // Get API key from keychain
                let entry = keyring::Entry::new("hqe-workbench", &profile.api_key_id)?;
                let api_key = entry.get_password()?;

                // Create client and test
                let config = hqe_openai::ClientConfig {
                    base_url: profile.base_url.clone(),
                    api_key: secrecy::SecretString::new(api_key),
                    default_model: profile.default_model.clone(),
                    timeout_seconds: 30,
                    max_retries: 1,
                    rate_limit_config: None,
                };

                let client = hqe_openai::OpenAIClient::new(config)?;

                println!("  Connecting to {}...", profile.base_url);

                match client.test_connection().await {
                    Ok(true) => println!("{}", style("✅ Connection successful!").green()),
                    Ok(false) => println!("{}", style("❌ Connection failed").red()),
                    Err(e) => println!("{}", style(format!("❌ Error: {}", e)).red()),
                }
            } else {
                println!("{}", style(format!("Profile not found: {}", name)).red());
            }
        }
        ConfigCommands::Remove { name } => {
            println!(
                "{}",
                style(format!("🗑️  Removing profile: {}", name)).bold()
            );

            if profiles_path.exists() {
                let content = tokio::fs::read_to_string(&profiles_path).await?;
                let mut profiles: Vec<hqe_openai::ProviderProfile> =
                    serde_json::from_str(&content)?;

                profiles.retain(|p| p.name != name);

                let json = serde_json::to_string_pretty(&profiles)?;
                tokio::fs::write(&profiles_path, json).await?;

                // Also remove from keychain
                let _ = keyring::Entry::new("hqe-workbench", &format!("api_key:{}", name))?
                    .delete_password();

                println!("{}", style("✅ Profile removed").green());
            }
        }
    }

    Ok(())
}
