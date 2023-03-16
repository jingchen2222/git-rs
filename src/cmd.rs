use crate::repo::GitRepository;
use clap::Parser;
#[derive(Debug, Parser)]
#[clap(name = "git-rs")]
pub enum GitCommand {
    #[clap(name = "init")]
    Init {},
    #[command(arg_required_else_help = true)]
    Add {
        /// Stuff to add
        #[arg(required = true)]
        paths: Vec<String>,
    },
}

impl GitCommand {
    pub fn execute(self) {
        let mut repo = GitRepository::new();
        match self {
            GitCommand::Init {} => match repo.init() {
                Ok(_) => {
                    println!(
                        "Initialized empty Git repository in {}",
                        repo.repo_path.display()
                    );
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            },
            GitCommand::Add { paths } => match repo.add(&paths) {
                Ok(_) => {}
                Err(err) => {
                    println!("{:?}", err);
                }
            },
        }
    }
}
