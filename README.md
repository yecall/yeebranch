# yeebranch

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)

> Official implementation of the YeeCo Branch Chain (Layer 2)

YeeCo is a permissionless, secure, high performance and scalable public blockchain platform powered by full sharding technology on PoW consensus.

## Table of Contents

- [Description](#description)
- [Install](#install)
- [Usage](#usage)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

## Description

Yeebranch is designed as layer 2 of YeeCo.

| Layer   | Name            |  Consensus   |  Transaction types   | 
| --------| --------------- | ------------ |--------------| 
| 1       | [Yeeroot chain](https://github.com/yeeco/yeeroot)   |  <ul><li>POW for block generation </li><li>CRFG for block finalization</li></ul> |  <ul><li> Native coin tx </li><li> Token tx </li><li> Meta tx of branch chains </li></ul>  |
| 2       | [Yeebranch chain](https://github.com/yeeco/yeebranch) |  Optional <br> <ul><li>DPOS</li><li>POA</li></ul> | <ul><li> Basic tx </li><li> Smart contract tx </li>

## Install

### Requirements
1. Rust
    ```sh
    curl https://sh.rustup.rs -sSf | sh
    ```
1. Openssl
1. Rust nightly
    ```sh
    rustup toolchain add nightly
    ```
1. rust nightly wasm
    ```sh
    rustup target add wasm32-unknown-unknown
    rustup target add wasm32-unknown-unknown --toolchain nightly
    ```
1. wasm-gc
    ```sh
    cargo install wasm-gc
    ```
1. Rust components: clippy rls docs src rustfmt
    ```sh
    rustup component list # list all the components installed
    rustup component add <name> # install component
    ```

### Building
```sh
$ cd <project_base_dir>/runtime/wasm
$ sh build.sh
$ cd <project_base_dir>
$ cargo build
```

## Usage

### Start

Start the node
    ```sh
    $ ./yee-branch --dev --alice
    ```

## Contributing

Feel free to dive in! [Open an issue](https://github.com/yeeco/yeebranch/issues).

### Contributors


## License

[GPL](LICENSE)