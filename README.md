# trojan-rust

![Build](https://github.com/cty123/TrojanRust/actions/workflows/build.yml/badge.svg) ![publish](https://github.com/cty123/TrojanRust/actions/workflows/publish.yml/badge.svg) ![Version](https://img.shields.io/badge/Version_0.1.0-blue.svg) ![Stage](https://img.shields.io/badge/beta-blue.svg)

[中文](https://github.com/cty123/TrojanRust/README.zh-CN.md)
Trojan-rust is a rust implementation for [Trojan protocol](https://trojan-gfw.github.io/trojan/protocol.html) that is targeted to circumvent [GFW](https://en.wikipedia.org/wiki/Great_Firewall). This implementation focus on performance and stability above everything else.

# Why trojan-rust

* Depends on [tokio-rs](https://github.com/tokio-rs/tokio) to achieve high performance async io. Tokio io provides better async IO performance by using lightweight threads that is somewhat similar to the runtime environment of Golang.

* Uses [rustls](https://github.com/ctz/rustls) to handle TLS protocol. rustls is an implemention written in native rust, and is considered to be more secure compared and performant compared to Openssl implementation.

* Performance focused. This implementation only aims at a few mainstream proxy protocols like [Trojan protocol](https://trojan-gfw.github.io/trojan/protocol.html), so that we have more capacity to improve the performance and bugfixes rather than keep adding useless features. 

* Easy to use/configure. Make this project beginner friendly, minimize the amount of configurations one needs to write.

# How to compile

Currently there is no existing binary file that you can just download and use, and it is recommanded to compile and build yourself. To do so, first you need to set up the Rust environment, by installing through here https://www.rust-lang.org/. Once you have rust installed, you can simply go to command line and run,

    cargo build --release

and it should generate a binary program under ./target/release/trojan-rust. 

Alternatively, you can also run it directly through,

    cargo run --release

To enable logs, on MacOs or Linux, run,

    RUST_LOG=info cargo run --release

On windows powershell, run,

    $Env:RUST_LOG = "info"
    cargo run --release

# Examples



## Create Certificate
Quick short script for your convenience,
    
    openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

## Sample config file
```json
{
    "inbound": {
        "protocol": "SOCKS",
        "address": "0.0.0.0",
        "port": 8081,
        "tls": true,
        "cert_path": "/path/to/file/cert.pem",
        "key_path": "/path/to/file/key.pem"
    },
    "outbound": {
        "protocol": "DIRECT"
    }
}
```

## Run the program

```
trojan-rust -h

Trojan Rust 0.0.1
Anonymous
Trojan Rust is a rust implementation of the trojan protocol to circumvent GFW

USAGE:
    trojan-rust [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <FILE>    Sets the config file, readers ./config/config.json by default
```

Run trojan-rust with specified config file

    trojan-rust --config ./config.json


# Roadmap

## Beta stage 0.0.1 - 1.0.0
- [x] Build up the framework for this project and support basic server side SOCKS5 protocol.

- [x] Support server side Trojan protocol for handling Trojan traffic.

- [ ] Implement client side Trojan protocol so that trojan-rust and be used as an client for using Trojan. - Work in progress.

- [ ] Implement UDP over TCP for Trojan protocol on both client side and server side.

- [ ] Performance profiling and bottleneck resolving. Will also include benchmarks versus other implementations.

## Official release 1.0.0 and above
- [ ] Build the package into kernel module release

- [ ] Support other protocols, gRPC, websocket etc.
