use std::path::PathBuf;

use anyhow::anyhow;
use release_plz::{GitHub, UpdateRequest};
use secrecy::SecretString;
use url::Url;

#[derive(clap::Parser, Debug)]
#[clap(about, version, author)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Update crates version based on commit messages.
    Update(Update),
    /// Update crates version based on commit messages and create a Pull Request.
    UpdateWithPr(UpdateWithPr),
}

#[derive(clap::Parser, Debug)]
pub struct Update {
    /// Path to the Cargo.toml of the project you want to update.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    #[clap(long)]
    project_manifest: Option<PathBuf>,
    /// Path to the Cargo.toml contained in the released version of the project you want to update.
    /// If not provided, the crates of your project will be compared with the
    /// ones published in the cargo registry (only crates.io at the moment).
    /// Normally, this parameter is used only if the published version of
    /// your project is already available locally.
    /// For example, it could be the path to the project with a `git checkout` on its latest tag.
    /// The git history of this project should be behind the one of the project you want to update.
    #[clap(long)]
    reference_project_manifest: Option<PathBuf>,
}

#[derive(clap::Parser, Debug)]
pub struct UpdateWithPr {
    #[clap(flatten)]
    pub update: Update,
    /// GitHub token used to create the pull request.
    #[clap(long)]
    pub github_token: SecretString,
    /// GitHub repository url where your project is hosted.
    #[clap(long)]
    pub repo_url: Url,
}

impl Update {
    pub fn update_request(&self) -> UpdateRequest {
        UpdateRequest {
            local_manifest: self.local_manifest(),
            remote_manifest: self.reference_project_manifest.clone(),
        }
    }
    fn local_manifest(&self) -> PathBuf {
        match &self.project_manifest {
            Some(manifest) => manifest.clone(),
            None => std::env::current_dir()
                .expect("cannot retrieve current directory")
                .join("Cargo.toml"),
        }
    }
}

impl UpdateWithPr {
    pub fn github(&self) -> anyhow::Result<GitHub> {
        let segments = self
            .repo_url
            .path_segments()
            .map(|c| c.collect::<Vec<_>>())
            .ok_or_else(|| {
                anyhow!(
                    "cannot find github owner and repo from url {}",
                    self.repo_url
                )
            })?;
        let owner = segments
            .get(0)
            .ok_or_else(|| anyhow!("cannot find github owner from url {}", self.repo_url))?
            .to_string();
        let repo = segments
            .get(1)
            .ok_or_else(|| anyhow!("cannot find github repo from url {}", self.repo_url))?
            .to_string();
        Ok(GitHub {
            owner,
            repo,
            token: self.github_token.clone(),
        })
    }
}
