//! HQE Workbench CLI

use clap::{Parser, Subcommand};
use console::style;
use hqe_core::models::*;
use hqe_core::scan::ScanPipeline;
use hqe_openai::profile::ProfileManager;
use hqe_openai::provider_discovery::is_local_or_private_base_url;
use hqe_openai::{ClientConfig, OpenAIAnalyzer, OpenAIClient};
use indicatif::{ProgressBar, ProgressStyle};
use secrecy::SecretString;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::Level;

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

        /// Disable local semantic caching
        #[arg(long)]
        no_cache: bool,
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

        /// Venice-specific parameters as JSON (advanced)
        #[arg(long, value_name = "JSON")]
        venice_parameters: Option<String>,

        /// Override parallel tool calls (advanced)
        #[arg(long, value_name = "BOOL")]
        parallel_tool_calls: Option<bool>,

        /// Disable local semantic caching
        #[arg(long)]
        no_cache: bool,
    },

    /// Export a specific run
    Export {
        /// Run ID to export
        #[arg(value_name = "RUN_ID")]
        run_id: String,

        /// Output directory
        #[arg(short, long, default_value = "./hqe-exports")]
        out: PathBuf,

        /// Source directory to search for run (artifacts)
        #[arg(long)]
        from: Option<PathBuf>,
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
        key: Option<String>,

        /// Default model
        #[arg(short, long, default_value = "gpt-4o-mini")]
        model: String,

        /// Additional header (repeatable). Format: "Header-Name: value"
        #[arg(long, value_name = "HEADER", action = clap::ArgAction::Append)]
        header: Vec<String>,

        /// OpenAI organization ID (optional)
        #[arg(long)]
        organization: Option<String>,

        /// OpenAI project ID (optional)
        #[arg(long)]
        project: Option<String>,

        /// Request timeout in seconds
        #[arg(long, default_value_t = 60)]
        timeout: u64,
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

            no_cache,
        } => handle_prompt(name, args, profile, no_cache).await,
        Commands::Scan {
            repo,
            profile,
            local_only,
            out,
            max_files,
            timeout,
            venice_parameters,
            parallel_tool_calls,
            no_cache,
        } => {
            let venice_params = match venice_parameters {
                Some(raw) => Some(
                    serde_json::from_str(&raw)
                        .map_err(|e| anyhow::anyhow!("Invalid venice_parameters JSON: {e}"))?,
                ),
                None => None,
            };
            scan_repo(ScanRepoArgs {
                repo,
                profile,
                local_only,
                out,
                max_files,
                timeout,
                venice_parameters: venice_params,
                parallel_tool_calls,

                no_cache,
            })
            .await
        }
        Commands::Export { run_id, out, from } => export_run(run_id, out, from).await,
        Commands::Patch {
            run_id,
            todo,
            preview,
            apply,
        } => handle_patch(run_id, todo, preview, apply).await,
        Commands::Config { command } => handle_config(command).await,
    }
}

struct ScanRepoArgs {
    repo: PathBuf,
    profile: Option<String>,
    local_only: bool,
    out: PathBuf,
    max_files: Option<usize>,
    timeout: u64,
    venice_parameters: Option<serde_json::Value>,
    parallel_tool_calls: Option<bool>,

    no_cache: bool,
}

async fn handle_prompt(
    tool_name: String,
    args_json: String,
    profile_name: Option<String>,

    no_cache: bool,
) -> anyhow::Result<()> {
    println!(
        "{}",
        style(format!("🤖 Executing Prompt: {}", tool_name))
            .bold()
            .cyan()
    );

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
            let allow_missing_key =
                is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
            let api_key = match entry.get_password() {
                Ok(key) => SecretString::new(key),
                Err(err) if allow_missing_key => SecretString::new(String::new()),
                Err(err) => return Err(err.into()),
            };

            let config = hqe_openai::ClientConfig {
                base_url: profile.base_url.clone(),
                api_key,
                default_model: profile.default_model.clone(),
                headers: profile.headers.clone(),
                organization: profile.organization.clone(),
                project: profile.project.clone(),
                disable_system_proxy: false,
                timeout_seconds: 120, // Longer timeout for detailed prompts
                max_retries: 1,
                rate_limit_config: None,
                cache_enabled: !no_cache,
            };
            Some(hqe_openai::OpenAIClient::new(config)?)
        } else {
            None
        }
    } else {
        None
    };

    let client = client.ok_or_else(|| {
        anyhow::anyhow!("No provider profile found. Use 'hqe config add' to configure one.")
    })?;
    let client = std::sync::Arc::new(client);

    // 2. Locate Prompts Directory
    let mut prompts_dir = PathBuf::from("./prompts");
    if !prompts_dir.exists() {
        if let Some(exe_dir) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        {
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
        let handler = Box::new(
            move |args: serde_json::Value| -> std::pin::Pin<
                Box<dyn std::future::Future<Output = anyhow::Result<serde_json::Value>> + Send>,
            > {
                let template = template.clone();
                let client_clone = client_clone.clone();

                Box::pin(async move {
                    let prompt_text = substitute_template(&template, &args);

                    let response = client_clone
                        .chat(hqe_openai::ChatRequest {
                            model: client_clone.default_model().to_string(),
                            messages: vec![hqe_openai::Message {
                                role: hqe_openai::Role::User,
                                content: Some(prompt_text.into()),
                                tool_calls: None,
                            }],
                            frequency_penalty: None,
                            presence_penalty: None,
                            repetition_penalty: None,
                            logprobs: None,
                            top_logprobs: None,
                            temperature: Some(0.2),
                            min_temp: None,
                            max_temp: None,
                            top_p: None,
                            top_k: None,
                            max_tokens: None,
                            max_completion_tokens: None,
                            n: None,
                            stop: None,
                            stop_token_ids: None,
                            seed: None,
                            user: None,
                            prompt_cache_key: None,
                            prompt_cache_retention: None,
                            reasoning_effort: None,
                            reasoning: None,
                            stream: None,
                            stream_options: None,
                            tool_choice: None,
                            tools: None,
                            venice_parameters: None,
                            parallel_tool_calls: None,
                            response_format: None,
                        })
                        .await?;

                    let text = response
                        .choices
                        .first()
                        .and_then(|c| c.message.content.as_ref().and_then(|c| c.to_text_lossy()))
                        .unwrap_or_default();

                    Ok(json!({ "result": text }))
                })
            },
        );

        let tool_name = tool.definition.name.clone();
        if let Err(e) = registry
            .register_tool("prompts", tool.definition, handler)
            .await
        {
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
            let val = v
                .as_str()
                .map(|s| validate_template_value(s))  // Validate string values
                .unwrap_or_else(|| validate_template_value(&v.to_string())); // Validate non-string values
            result = result.replace(&key, &val);
        }
    }

    result
}

// Validate that a template value doesn't contain dangerous patterns
fn validate_template_value(value: &str) -> String {
    // If the value contains template-like expressions, escape them to prevent processing
    let mut result = value.to_string();

    // Escape template delimiters to prevent them from being processed as templates
    result = result.replace("{{", "\\{\\{");
    result = result.replace("{%", "\\{%");
    result = result.replace("{#", "\\{#");
    result = result.replace("}}", "\\}\\}");
    result = result.replace("%}", "%\\}");
    result = result.replace("#}", "#\\}");

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

    // Check Python dependencies for full validation (pyyaml + jsonschema).
    // If missing, fall back to the basic syntax validation path instead of failing CI/users.
    let deps_check = tokio::process::Command::new("python3")
        .args(["-c", "import yaml, jsonschema"])
        .output()
        .await?;

    if !deps_check.status.success() {
        println!(
            "{}",
            style("⚠️  Python dependencies missing (pyyaml/jsonschema), performing basic validation...")
                .yellow()
        );

        let yaml_content = tokio::fs::read_to_string(&yaml_path).await?;
        let schema_content = tokio::fs::read_to_string(&schema_path).await?;

        let _: serde_yaml::Value = serde_yaml::from_str(&yaml_content)
            .map_err(|e| anyhow::anyhow!("Invalid YAML: {}", e))?;

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

async fn scan_repo(args: ScanRepoArgs) -> anyhow::Result<()> {
    let ScanRepoArgs {
        repo,
        profile,
        local_only,
        out,
        max_files,
        timeout,
        venice_parameters,
        parallel_tool_calls,
        no_cache,
    } = args;
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
        venice_parameters: venice_parameters.clone(),
        parallel_tool_calls,
    };

    // Run scan
    pb.set_message("Initializing scan pipeline...");
    let mut pipeline = ScanPipeline::new(&repo, config.clone())?;
    if config.llm_enabled && !config.local_only {
        let profile_name = config
            .provider_profile
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Provider profile required for LLM scans"))?;
        let manager = ProfileManager::default();
        let (profile, api_key) = manager
            .get_profile_with_key(&profile_name)?
            .ok_or_else(|| anyhow::anyhow!("Profile not found"))?;
        let allow_missing_key = is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
        let api_key = match api_key {
            Some(key) => key,
            None if allow_missing_key => SecretString::new(String::new()),
            None => return Err(anyhow::anyhow!("No API key stored for profile")),
        };

        let base_url = profile.base_url.clone();
        let default_model = profile.default_model.clone();

        pipeline.set_provider_info(ProviderInfo {
            name: profile.name.clone(),
            base_url: Some(base_url.clone()),
            model: Some(default_model.clone()),
            llm_enabled: true,
        });

        let llm_client = OpenAIClient::new(ClientConfig {
            base_url,
            api_key,
            default_model,
            headers: profile.headers.clone(),
            organization: profile.organization.clone(),
            project: profile.project.clone(),
            disable_system_proxy: false,
            timeout_seconds: timeout,
            max_retries: 2,
            rate_limit_config: None,
            cache_enabled: !no_cache,
        })?;
        let analyzer = OpenAIAnalyzer::new(llm_client)
            .with_venice_parameters(venice_parameters)
            .with_parallel_tool_calls(parallel_tool_calls);
        pipeline = pipeline.with_llm_analyzer(Arc::new(analyzer));
    }

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

async fn export_run(
    run_id: String,
    out_dir: PathBuf,
    from_dir: Option<PathBuf>,
) -> anyhow::Result<()> {
    println!(
        "{}",
        style(format!("📦 Exporting run: {}", run_id)).bold().cyan()
    );
    println!("  Output: {}", out_dir.display());

    if !is_valid_run_id(&run_id) {
        return Err(anyhow::anyhow!("Invalid run ID format"));
    }

    let source = locate_run_dir(&run_id, from_dir)?;

    std::fs::create_dir_all(&out_dir)?;

    for entry in std::fs::read_dir(&source)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }
        let target_file = out_dir.join(entry.file_name());
        tokio::fs::copy(entry.path(), target_file).await?;
    }

    println!("\n{}", style("✅ Export complete").green().bold());
    println!("  Source: {}", source.display());

    Ok(())
}

fn is_valid_run_id(run_id: &str) -> bool {
    run_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
}

fn locate_run_dir(run_id: &str, from_dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(dir) = from_dir {
        candidates.push(dir);
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("hqe-output"));
    } else {
        candidates.push(PathBuf::from("./hqe-output"));
    }

    if let Some(data_dir) = dirs::data_local_dir() {
        candidates.push(data_dir.join("hqe-workbench").join("hqe-output"));
    }

    for root in candidates {
        let candidate = root.join(format!("hqe_run_{}", run_id));
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(anyhow::anyhow!(
        "Artifacts not found for run ID (searched ./hqe-output and app data dir)"
    ))
}

async fn handle_patch(
    run_id: String,
    todo_id: String,
    preview: bool,
    apply: bool,
) -> anyhow::Result<()> {
    println!(
        "{}",
        style(format!("🔧 Patch: {} for run {}", todo_id, run_id)).bold()
    );

    if !preview && !apply {
        println!("{}", style("Use --preview or --apply").yellow());
        return Ok(());
    }

    // Locate report
    let run_dir = locate_run_dir(&run_id, None)?;
    let report_path = run_dir.join("hqe_report.json");

    if !report_path.exists() {
        return Err(anyhow::anyhow!(
            "Report not found at {}",
            report_path.display()
        ));
    }

    let content = tokio::fs::read_to_string(&report_path).await?;
    let report: HqeReport = serde_json::from_str(&content)?;

    // Find patch
    let patch = report
        .immediate_actions
        .iter()
        .find(|p| p.todo_id == todo_id);

    match patch {
        Some(p) => {
            println!("  Found patch: {}", style(&p.title).bold());
            println!("  Risk: {}", p.risk);
            println!();

            for diff in &p.diffs {
                println!(
                    "{}",
                    style(format!("File: {}", diff.file_path)).underlined()
                );

                if preview {
                    println!("{}", style(&diff.diff_content).dim());
                }

                if apply {
                    println!("  Applying patch...");

                    // Create temp file for the diff
                    let mut temp = tempfile::Builder::new().suffix(".patch").tempfile()?;
                    std::io::Write::write_all(&mut temp, diff.diff_content.as_bytes())?;

                    let status = std::process::Command::new("patch")
                        .arg("-p1")
                        .arg("-i")
                        .arg(temp.path())
                        .status();

                    match status {
                        Ok(s) if s.success() => {
                            println!("  {}", style("Success").green());
                        }
                        Ok(s) => {
                            println!("  {}", style(format!("Failed with exit code: {}", s)).red());
                        }
                        Err(e) => {
                            println!(
                                "  {}",
                                style(format!("Failed to execute patch command: {}", e)).red()
                            );
                            println!("  Ensure 'patch' utility is installed.");
                        }
                    }
                }
            }
        }
        None => {
            println!(
                "{}",
                style(format!("No patch found for TODO ID: {}", todo_id)).red()
            );
            println!("Available patches:");
            for p in &report.immediate_actions {
                println!("  - {} ({})", p.todo_id, p.title);
            }
        }
    }

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
            header,
            organization,
            project,
            timeout,
        } => {
            println!(
                "{}",
                style(format!("➕ Adding profile: {}", name)).bold().green()
            );

            let allow_missing_key = is_local_or_private_base_url(&url).unwrap_or(false);
            let key_value = key.as_deref().map(|k| k.trim()).filter(|k| !k.is_empty());

            if let Some(key_value) = key_value {
                // Store API key in keychain
                let entry = keyring::Entry::new("hqe-workbench", &format!("api_key:{}", name))?;
                entry.set_password(key_value)?;
            } else if !allow_missing_key {
                return Err(anyhow::anyhow!(
                    "API key is required for non-local providers. Use --key or select a local base URL."
                ));
            }

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

            let mut profile = hqe_openai::ProviderProfile::new(name.clone(), url).with_model(model);
            profile.timeout_s = timeout;
            profile.organization = organization;
            profile.project = project;

            if !header.is_empty() {
                let mut headers = std::collections::HashMap::new();
                for raw in header {
                    let (name, value) = raw
                        .split_once(':')
                        .or_else(|| raw.split_once('='))
                        .ok_or_else(|| {
                            anyhow::anyhow!("Invalid header format '{}'. Use 'Header: value'", raw)
                        })?;
                    headers.insert(name.trim().to_string(), value.trim().to_string());
                }
                profile.headers = Some(headers);
            }

            profile.validate_headers().map_err(|e| anyhow::anyhow!(e))?;

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
                let allow_missing_key =
                    is_local_or_private_base_url(&profile.base_url).unwrap_or(false);
                let api_key = match entry.get_password() {
                    Ok(key) => SecretString::new(key),
                    Err(err) if allow_missing_key => SecretString::new(String::new()),
                    Err(err) => return Err(err.into()),
                };

                // Create client and test
                let config = hqe_openai::ClientConfig {
                    base_url: profile.base_url.clone(),
                    api_key,
                    default_model: profile.default_model.clone(),
                    headers: profile.headers.clone(),
                    organization: profile.organization.clone(),
                    project: profile.project.clone(),
                    disable_system_proxy: false,
                    timeout_seconds: 30,
                    max_retries: 1,
                    rate_limit_config: None,
                    cache_enabled: true,
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
