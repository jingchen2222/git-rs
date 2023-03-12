use crate::repo::GitRepository;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
#[derive(Debug, Parser)]
#[clap(name = "git-rs")]
pub enum GitCommand {
    #[clap(name = "init")]
    Init {},
    #[command(arg_required_else_help = true)]
    Add {
        /// Stuff to add
        #[arg(required = true)]
        path: Vec<PathBuf>,
    },
}

impl GitCommand {
    pub fn execute(self) {
        let repo = GitRepository::new();
        match self {
            GitCommand::Init {} => match repo.init() {
                Ok(_) => {
                    println!(
                        "Initialized empty Git repository in {}",
                        repo.repo_dir.display()
                    );
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            },
            GitCommand::Add { path } => {
                println!("{:?}", path);
            }
        }
    }
}
