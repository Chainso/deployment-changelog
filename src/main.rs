use deployment_changelog::{changelog::{Changelog, CommitSpecifier}, api::{jira::JiraClient, bitbucket::BitbucketClient}};
use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::Verbosity;

use git2::{Error, Oid, Repository, Revwalk, Object};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    commit_specifier: CommitSpecifier,

    project: String,
    repo: String,

    #[clap(flatten)]
    verbose: Verbosity
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
    // let repo = match Repository::open(".") {
    //     Ok(repository) => repository,
    //     Err(error) => panic!("Problem opening the current repository: {error}")
    // };
    //
    // let revlist = match RevList::new(&repo, &args.start_commit, &args.end_commit) {
    //     Ok(revs) => revs,
    //     Err(error) => panic!("Error opening revlist for the repo: {error}")
    // };
    //
    // println!("Walking from commit {0} to {1}", revlist.start_commit.id(), revlist.end_commit.id());
    //
    // for commit in revlist {
    //     println!("{commit}");
    // }
    log::info!("Getting changelog for args: {:?}", args);

    let bitbucket_url = "https://opensource.ncsa.illinois.edu/bitbucket/";
    let bitbucket_client = BitbucketClient::new(bitbucket_url)?;
    //
    // let commit_diff = bitbucket_client.compare_commits(&args.project, &args.repo, &args.commit_specifier.start_commit, &args.commit_specifier.end_commit).await;
    // println!("Commits:\n{}\n", commit_diff);
    //
    // let pull_requests: Vec<BitbucketPullRequest> = futures::future::join_all(
    //     commit_diff.values.iter()
    //         .map(|commit| bitbucket_client.get_pull_requests(&args.project, &args.repo, &commit.id))
    // )
    //     .await
    //     .into_iter()
    //     .flat_map(|pull_request_page| pull_request_page.values)
    //     .collect();
    //
    // println!("Pull Requests for each commit:\n{:?}\n", pull_requests);
    //
    // let issues: Vec<BitbucketPullRequestIssue> = futures::future::join_all(
    //     pull_requests.iter()
    //         .map(|pull_request| bitbucket_client.get_pull_request_issues(&args.project, &args.repo, pull_request.id))
    // )
    //     .await
    //     .into_iter()
    //     .flatten()
    //     .collect();
    //
    // println!("Issues for commits:\n{:?}\n", issues);
    //
    let jira_url = "https://issues.apache.org/jira/";
    let jira_client = JiraClient::new(jira_url)?;
    //
    // let issue_key = "CASSANDRA-18339";
    // let issue = jira_client.get_issue(issue_key).await;
    // println!("Issue:\n{}\n", issue);
    //
    let changelog: Changelog = Changelog::new(
        &bitbucket_client,
        &jira_client,
        &args.project,
        &args.repo,
        &args.commit_specifier
    ).await?;

    println!("{}", changelog);
    Ok(())
}


struct RevList<'repo> {
    repo: &'repo Repository,
    revwalk: Revwalk<'repo>,
    start_commit: Object<'repo>,
    end_commit: Object<'repo>
}

impl<'repo> RevList<'repo> {
    pub fn new(repo: &'repo Repository, start_commit: &String, end_commit: &String) -> Result<Self, Error> {
        let start = repo.revparse_single(&start_commit)?;
        let end =  repo.revparse_single(&end_commit)?;

        let mut revwalk = repo.revwalk()?;
        revwalk.push(start.id())?;
        
        let mut revlist = Self {
            repo,
            revwalk,
            start_commit: start,
            end_commit: end,
        };

        Ok(revlist)
    }

    fn get_next(&mut self) -> Option<Oid> {
        self.revwalk.next()
            .map(|commit_id| {
                match commit_id {
                    Ok(commit) => commit,
                    Err(error) => panic!("Error walking over commit: {error}")
                }
            })
            .filter(|commit_id| self.end_commit.id() != *commit_id)
    }
}
 
impl Iterator for RevList<'_> {
    type Item = Oid;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next()
    }
}


