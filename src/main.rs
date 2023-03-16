use clap::Parser;
use git2::{Error, Oid, Repository, Revwalk, Object};

#[derive(Parser)]
struct Args {
    start_commit: String,
    end_commit: String
}

fn main() {
    println!("Hello, world!");
    let args = Args::parse();

    let repo = match Repository::open(".") {
        Ok(repository) => repository,
        Err(error) => panic!("Problem opening the current repository: {error}")
    };

    let revlist = match RevList::new(&repo, &args.start_commit, &args.end_commit) {
        Ok(revs) => revs,
        Err(error) => panic!("Error opening revlist for the repo: {error}")
    };

    for commit in revlist {
        println!("{commit}");
    }
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
        let end =  repo.revparse_single(&start_commit)?;

        let mut revwalk = repo.revwalk()?;
        revwalk.push(start.id())?;

        Ok(Self {
            repo,
            revwalk,
            start_commit: start,
            end_commit: end
        })
    }
}

impl Iterator for RevList<'_> {
    type Item = Oid;

    fn next(&mut self) -> Option<Self::Item> {
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
