use clap::Parser;
use git_rs::cmd::GitCommand;
fn main() {
    let command = GitCommand::parse();
    command.execute();
}
