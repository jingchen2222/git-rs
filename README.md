# git-rs
git rust version

## Roadmap

- [ ] Setup project
- [ ] Git init
- [ ] Git add
- [ ] Git status
- [ ] Git commit
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
./target/debug/git-rs help                                                                                                                                                   ─╯
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

> ./target/debug/git-rs init                                                                                                                                                   ─╯
Initialized empty Git repository in /Users/chenjing/work/study/ruststudy/git-rs/.git-rs

### Add
```
./target/debug/git-rs add README.md                                                                                                           
./target/debug/git-rs add src/repo.rs    
```

```
> ls .git-rs  
HEAD       STAGED_ADD blobs      commits    staged

> ls .git-rs/staged
d9af0cc2956707e50f215493e5331f0424644b90 eb294dd16b68bfad2510ee7334526c86cfd3734c

> cat .git-rs/STAGED_ADD

{"blobs":{"src/repo.rs":"d9af0cc2956707e50f215493e5331f0424644b90"}}%

```
