//! HQE Workbench CLI

use clap::{Parser, Subcommand};
use console::style;
use hqe_core::models::*;
use hqe_core::scan::ScanPipeline;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
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
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::ValidateProtocol => validate_protocol().await,
        Commands::Scan { repo, profile, local_only, out, max_files } => {
            scan_repo(repo, profile, local_only, out, max_files).await
        }
        Commands::Export { run_id, out } => export_run(run_id, out).await,
        Commands::Patch { run_id, todo, preview, apply } => {
            handle_patch(run_id, todo, preview, apply).await
        }
        Commands::Config { command } => handle_config(command).await,
    }
}

// Embed protocol files at compile time for standalone binary distribution
const PROTOCOL_YAML: &str = include_str!("../../../protocol/hqe-engineer.yaml");
const PROTOCOL_SCHEMA: &str = include_str!("../../../protocol/hqe-schema.json");
const PROTOCOL_VERSION: &str = "3.0.0";

async fn validate_protocol() -> anyhow::Result<()> {
    println!("{}", style("🔍 Validating HQE Protocol...").bold());
    
    // Try to find protocol files in multiple locations
    let mut possible_paths: Vec<PathBuf> = vec![
        // Development: relative to project root
        PathBuf::from("./protocol"),
    ];
    
    // Installed: relative to executable
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
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
        println!("{}", style("⚠️  Python not available, performing basic validation...").yellow());
        
        let yaml_content = tokio::fs::read_to_string(&yaml_path).await?;
        let schema_content = tokio::fs::read_to_string(&schema_path).await?;
        
        // Check YAML is valid
        let _: serde_yaml::Value = serde_yaml::from_str(&yaml_content)
            .map_err(|e| anyhow::anyhow!("Invalid YAML: {}", e))?;
        
        // Check JSON schema is valid
        let _: serde_json::Value = serde_json::from_str(&schema_content)
            .map_err(|e| anyhow::anyhow!("Invalid JSON schema: {}", e))?;
        
        println!("{}", style("\n✅ Basic validation passed (syntax OK)").green().bold());
        println!("{}", style("   Note: Install Python with pyyaml and jsonschema for full validation").dim());
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
        .args(&[
            verify_py.to_str().unwrap(),
            "--yaml", yaml_path.to_str().unwrap(),
            "--schema", schema_path.to_str().unwrap(),
        ])
        .output()
        .await?;
    
    if output.status.success() {
        println!("{}", style("\n✅ Protocol validation passed").green().bold());
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
) -> anyhow::Result<()> {
    println!("{}", style("🔍 HQE Repository Scan").bold().cyan());
    println!("  Repository: {}", repo.display());
    let mode_str = if local_only {
        style("local-only").yellow().to_string()
    } else {
        style(format!("LLM ({})", profile.as_deref().unwrap_or("default"))).green().to_string()
    };
    println!("  Mode: {}", mode_str);
    println!("  Output: {}", out.display());
    println!();
    
    // Setup progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap());
    
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
    println!("  Health Score: {}/10", result.report.executive_summary.health_score);
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

async fn handle_patch(run_id: String, todo: String, preview: bool, apply: bool) -> anyhow::Result<()> {
    println!("{}", style(format!("🔧 Patch: {} for run {}", todo, run_id)).bold());
    
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
    println!("\n{}", style("Patch generation not yet implemented").yellow());
    
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
                
                for profile in profiles {
                    println!("  • {} ({})", profile.name, profile.base_url);
                    println!("    Model: {}", profile.default_model);
                }
            } else {
                println!("  No profiles configured.");
                println!("  Use: hqe config add <name> --url <url> --key <key>");
            }
        }
        ConfigCommands::Add { name, url, key, model } => {
            println!("{}", style(format!("➕ Adding profile: {}", name)).bold().green());
            
            // Store API key in keychain
            let entry = keyring::Entry::new("hqe-workbench", &format!("api_key:{}", name))?;
            entry.set_password(&key)?;
            
            // Load existing profiles
            let mut profiles: Vec<hqe_openai::ProviderProfile> = if profiles_path.exists() {
                let content = tokio::fs::read_to_string(&profiles_path).await?;
                serde_json::from_str(&content)?
            } else {
                vec![]
            };
            
            // Add or update profile
            let profile = hqe_openai::ProviderProfile {
                name: name.clone(),
                base_url: url,
                api_key_id: format!("api_key:{}", name),
                default_model: model,
                headers: None,
                organization: None,
                project: None,
            };
            
            profiles.retain(|p| p.name != name);
            profiles.push(profile);
            
            // Save
            let json = serde_json::to_string_pretty(&profiles)?;
            tokio::fs::write(&profiles_path, json).await?;
            
            println!("{}", style("✅ Profile saved").green());
        }
        ConfigCommands::Test { name } => {
            println!("{}", style(format!("🧪 Testing connection: {}", name)).bold());
            
            // Load profile
            let content = tokio::fs::read_to_string(&profiles_path).await?;
            let profiles: Vec<hqe_openai::ProviderProfile> = serde_json::from_str(&content)?;
            
            let profile = profiles.iter().find(|p| p.name == name);
            
            if let Some(profile) = profile {
                // Get API key from keychain
                let entry = keyring::Entry::new("hqe-workbench", &profile.api_key_id)?;
                let api_key = entry.get_password()?;
                
                // Create client and test
                let config = hqe_openai::ClientConfig {
                    base_url: profile.base_url.clone(),
                    api_key: secrecy::SecretString::new(api_key.into()),
                    default_model: profile.default_model.clone(),
                    headers: profile.headers.clone(),
                    organization: profile.organization.clone(),
                    project: profile.project.clone(),
                    disable_system_proxy: false,
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
            println!("{}", style(format!("🗑️  Removing profile: {}", name)).bold());
            
            if profiles_path.exists() {
                let content = tokio::fs::read_to_string(&profiles_path).await?;
                let mut profiles: Vec<hqe_openai::ProviderProfile> = serde_json::from_str(&content)?;
                
                profiles.retain(|p| p.name != name);
                
                let json = serde_json::to_string_pretty(&profiles)?;
                tokio::fs::write(&profiles_path, json).await?;
                
                // Also remove from keychain
                let _ = keyring::Entry::new("hqe-workbench", &format!("api_key:{}", name))?.delete_password();
                
                println!("{}", style("✅ Profile removed").green());
            }
        }
    }
    
    Ok(())
}
