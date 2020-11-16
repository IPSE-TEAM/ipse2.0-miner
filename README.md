# Introduction

This project is tool for [ipse project miner](https://github.com/IPSE-TEAM/ipse-core).



# Development

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```


Install required tools:

```bash
make init
```

Build all native code:

```bash
make build
```

install command tools:

```bash
make install
```

# Production

init project folder

```bash
miner init xxx
```


Modify `config.toml` under `xxx` directory

```
cd xxx
vim config.toml
```

launch production server

```bash
miner server
```

scheduling tasks  for miner

```bash
miner job
```
