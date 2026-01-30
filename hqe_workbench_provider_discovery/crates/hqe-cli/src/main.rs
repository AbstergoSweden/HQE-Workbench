use std::{collections::BTreeMap, time::Duration};

use clap::{Parser, Subcommand};

use hqe_openai::{
    profile::{DefaultProfilesStore, KeychainStore, ProfilesStore},
    provider_discovery::{DiskCache, ProviderDiscoveryClient},
};

#[derive(Debug, Parser)]
#[command(name="hqe", version, about="HQE Workbench CLI (provider discovery subset)")]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// List chat models for a stored provider profile.
    ListModels {
        /// Profile name in profiles.json
        #[arg(long)]
        profile: String,

        /// Disable disk cache
        #[arg(long)]
        no_cache: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.cmd {
        Cmd::ListModels { profile, no_cache } => {
            let store = DefaultProfilesStore;
            let profiles = store.load_profiles()?;
            let p = profiles.into_iter().find(|x| x.name == profile)
                .ok_or_else(|| anyhow::anyhow!("profile not found: {profile}"))?;

            let keychain = KeychainStore::default();
            let api_key = keychain.get_api_key(&p.name)?.unwrap_or_default();
            if api_key.is_empty() {
                anyhow::bail!("No API key in keychain for profile '{}'", p.name);
            }

            let cache = if no_cache { None } else { Some(DiskCache::default()) };

            let client = ProviderDiscoveryClient::new(
                &p.base_url,
                &p.headers,
                Some(api_key),
                Duration::from_secs(p.timeout_s),
                cache,
            )?;

            let list = client.discover_chat_models().await?;
            println!("{}", serde_json::to_string_pretty(&list)?);
        }
    }

    Ok(())
}
