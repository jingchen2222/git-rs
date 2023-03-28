# git-rs
git rust version

## Roadmap

- [x] Setup project
- [x] Git init
- [x] Git add
- [x] Git status (display modifications not staged and untracked files in the future)
- [x] Git commit
- [x] Git log
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
### Log
```
./target/debug/git-rs log
```

```
===
commit 4e8a0324b3e1fa9ed8f231e4cb7a2c2192993aa6
Date: Tue Mar 28 23:44:53 2023 +0000
remove v1



===
commit 1fb6db29a778fb16ef850d299f8f38dbf72668f5
Date: Tue Mar 28 23:44:26 2023 +0000
first commit test



===
commit 992f1e2eb8d68f9aa4bbb30f722de1f818831bc7
Date: Tue Mar 28 23:44:14 2023 +0000
initial commit
```

## Demo
