use deployment_changelog::{changelog::{Changelog, CommitSpecifier, SpinnakerEnvironment, GitCommitRange}, api::{jira::JiraClient, bitbucket::BitbucketClient, spinnaker::SpinnakerClient}};
use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    commit_specifier: CommitSpecifierSubcommand,

    #[clap(long, short = 'b', help = "The URL to your Bitbucket server", env = "BITBUCKET_URL")]
    bitbucket_url: String,

    #[clap(long, short = 'j', help = "The URL to your JIRA server", env = "JIRA_URL")]
    jira_url: String,

    #[clap(flatten)]
    verbose: Verbosity
}

#[derive(Parser, Debug)]
enum CommitSpecifierSubcommand {
    Spinnaker(SpinnakerArgs),
    CommitRange(CommitRangeArgs)
}

#[derive(Parser, Debug)]
struct SpinnakerArgs {
    #[clap(long, short = 's', help = "The URL to your Spinnaker server", env = "SPINNAKER_URL")]
    spinnaker_url: String,

    #[clap(help = "The Spinnaker app name")]
    app_name: String,

    #[clap(help = "The Spinnaker environment")]
    env: String
}

#[derive(Parser, Debug)]
struct CommitRangeArgs {
    #[clap(help = "The Bitbucket project")]
    project: String,

    #[clap(help = "The Bitbucket repository")]
    repo: String,

    #[clap(help = "The start commit to get the changelog for, this commit should be more recent than the end commit")]
    start_commit: String,

    #[clap(help = "The end commit to get the changelog for, this commit should be older than the start commit")]
    end_commit: String
}

impl TryFrom<&CommitSpecifierSubcommand> for CommitSpecifier {
    type Error = anyhow::Error;

    fn try_from(commit_specifier_subcommand: &CommitSpecifierSubcommand) -> Result<Self> {
        match commit_specifier_subcommand {
            CommitSpecifierSubcommand::Spinnaker(spinnaker_args) => Ok(CommitSpecifier::Spinnaker(SpinnakerEnvironment {
                client: SpinnakerClient::new(&spinnaker_args.spinnaker_url)?,
                app_name: spinnaker_args.app_name.clone(),
                env: spinnaker_args.env.clone()
            })),
            CommitSpecifierSubcommand::CommitRange(commit_range) => Ok(CommitSpecifier::CommitRange(GitCommitRange {
                project: commit_range.project.clone(),
                repo: commit_range.repo.clone(),
                start_commit: commit_range.start_commit.clone(),
                end_commit: commit_range.end_commit.clone()
            }))
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Parsing arguments");

    let args = Args::parse();
    match print_changelog(&args).await {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {error}")
    }
}

async fn print_changelog(args: &Args) -> Result<()> {
    log::info!("Getting changelog for args: {:?}", args);

    let bitbucket_client = BitbucketClient::new(&args.bitbucket_url)?;
    let jira_client = JiraClient::new(&args.jira_url)?;

    let commit_specifier = CommitSpecifier::try_from(&args.commit_specifier)?;

    let changelog: Changelog = Changelog::new(
        &bitbucket_client,
        &jira_client,
        &commit_specifier
    ).await?;

    println!("{}", changelog);
    Ok(())
}

