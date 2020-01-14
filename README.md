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

### Prepare yeeroot chain resources

Yeebranch runs a light node of [yeeroot chain](https://github.com/yeeco/yeeroot) internally, so it needs the following resources of the yeeroot chain:

 - Runtime WASM
    
    After you [build yeeroot project](https://github.com/yeeco/yeeroot#building), 
    You will get the `Runtime WASM` on `<yeeroot_project_base_dir>/runtime/wasm/target/wasm32-unknown-unknown/release/yee_runtime_wasm.compact.wasm`
 
 - Chain spec 
 
    Run `build-spec` to export chain spec file:
     ```sh
     $ yee build-spec --dev > root_chain_sepc.json
     ```
   
### Building

```sh
$ cd <project_base_dir>/runtime/wasm
$ sh build.sh
$ cd <project_base_dir>
$ WASM_CODE_PATH=<yeeroot_project_base_dir>/runtime/wasm/target/wasm32-unknown-unknown/release/yee_runtime_wasm.compact.wasm cargo build
```

## Usage

### Start

1. Deploy yeeroot chain sepc
    
   ```sh
   $ mkdir -p <yeebranch_run_base_path>/conf
   $ cp root_chain_sepc.json <yeebranch_run_base_path>/conf
   ``` 
   
   <yeebranch_run_base_path> is: 
   `~/Library/Application\ Support/YeeBranch/`
   or the one you specify by `./yee-branch --base-path=<yeebranch_run_base_path>`, 

1. Start the node
    ```sh
    $ ./yee-branch --base-path=<yeebranch_run_base_path> --dev --alice
    ```

## Contributing

Feel free to dive in! [Open an issue](https://github.com/yeeco/yeebranch/issues).

### Contributors


## License

[GPL](LICENSE)