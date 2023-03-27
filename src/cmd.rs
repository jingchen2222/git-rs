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
    /// rm [file name]
    ///
    /// Description: Unstage the file if it is currently staged for addition. If the file is tracked in the current commit, stage it for removal and remove the file from the working directory if the user has not already done so (do not remove it unless it is tracked in the current commit).
    ///
    /// Runtime: Should run in constant time relative to any significant measure.
    ///
    /// Failure cases: If the file is neither staged nor tracked by the head commit, print the error message No reason to remove the file.
    ///
    /// Dangerous: Yes (although if you use our utility methods, you will only hurt your repository files, and not all the other files in your directory.)
    ///
    Rm {
        /// Stuff to remove
        #[arg(required = true)]
        paths: Vec<String>,
    },
    ///
    /// commit [message]
    /// Description: Saves a snapshot of tracked files in the current commit and staging area
    /// so they can be restored at a later time, creating a new commit.
    /// The commit is said to be tracking the saved files.
    /// By default, each commit’s snapshot of files will be exactly the same as its parent
    /// commit’s snapshot of files; it will keep versions of files exactly as they are,
    /// and not update them. A commit will only update the contents of files it is tracking
    /// that have been staged for addition at the time of commit, in which case the commit
    /// will now include the version of the file that was staged instead of the version
    /// it got from its parent.
    /// A commit will save and start tracking any files that were staged for addition but
    /// weren’t tracked by its parent. Finally, files tracked in the current commit may be
    /// untracked in the new commit as a result being staged for removal by the rm command (below).
    ///
    /// Failure cases: If no files have been staged, abort.
    /// Print the message No changes added to the commit.
    /// Every commit must have a non-blank message.
    /// If it doesn’t, print the error message Please enter a commit message.
    /// It is not a failure for tracked files to be missing from the working directory or
    /// changed in the working directory.
    #[command(arg_required_else_help = true)]
    Commit {
        #[arg(required = true)]
        message: String,
    },

    /// Usage: java gitlet.Main status
    /// Description: Displays what branches currently exist, and marks the current branch with a *.
    /// Also displays what files have been staged for addition or removal. An example of the exact
    /// format it should follow is as follows.
    /// Example:
    /// === Branches ===
    /// *master
    /// other-branch
    ///
    /// === Staged Files ===
    /// wug.txt
    /// wug2.txt
    ///
    /// === Removed Files ===
    /// goodbye.txt
    ///
    /// === Modifications Not Staged For Commit ===
    /// junk.txt (deleted)
    /// wug3.txt (modified)
    ///
    /// === Untracked Files ===
    /// random.stuff
    ///
    #[clap(name = "status")]
    Status {},
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
            GitCommand::Rm { paths } => match repo.remove(&paths) {
                Ok(_) => {}
                Err(err) => {
                    println!("{:?}", err);
                }
            },
            GitCommand::Commit { message } => match repo.commit(message.as_str()) {
                Ok(_) => {}
                Err(err) => {
                    println!("{:?}", err);
                }
            },
            GitCommand::Status {} => match repo.status() {
                Ok(msg) => {
                    println!("{}", msg);
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            },
        }
    }
}
