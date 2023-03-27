# git-rs
git rust version

## Roadmap

- [x] Setup project
- [x] Git init
- [x] Git add
- [x] Git status (display modifications not staged and untracked files in the future)
- [x] Git commit
- [ ] Git revert
- [ ] Git branch
- [ ] Git checkout

## Compile

```
cargo build
```

## Test

```
cargo test
```

## Run git-rs Command
### Help
```
./target/debug/git-rs help                                                                                                                     

Usage: git-rs <COMMAND>

Commands:
  init
  add
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Init

```
./target/debug/git-rs init  

Initialized empty Git repository in /Users/chenjing/work/study/ruststudy/git-rs/.git-rs
```


### Add
```
> touch v1 v2 v3
> ./target/debug/git-rs add README.md src/cmd.rs v1  
```
### Status
```
./target/debug/git-rs status                                                                                                                   
```

```
=== Branches ===
*main

=== Staged Files ===
README.md
src/cmd.rs
v1

=== Removed Files ===

=== modifications not staged for commit ===

=== untracked files ===

```

### Commit
```
./target/debug/git-rs commit "first commit test"

./target/debug/git-rs status     
```
```
=== Branches ===
*main

=== Staged Files ===
=== Removed Files ===

=== modifications not staged for commit ===

=== untracked files ===
```
