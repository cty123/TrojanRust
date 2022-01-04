# trojan-rust

![Build](https://github.com/cty123/TrojanRust/actions/workflows/build.yml/badge.svg) ![publish](https://github.com/cty123/TrojanRust/actions/workflows/publish.yml/badge.svg) ![Version](https://img.shields.io/github/v/release/cty123/TrojanRust) ![Stage](https://img.shields.io/badge/beta-blue.svg)

[中文zh-CN](https://github.com/cty123/TrojanRust/blob/main/README.zh-CN.md)

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

## Sample for Trojan
### Server config
```json
{
    "inbound": {
        "protocol": "TROJAN",
        "address": "0.0.0.0",
        "secret": "123123",
        "port": 8081,
        "tls": {
            "cert_path": "./cert.pem",
            "key_path": "./key.pem"
        }
    },
    "outbound": {
        "protocol": "DIRECT"
    }
}
```

### Client config
```json
{
    "inbound": {
        "protocol": "SOCKS",
        "address": "0.0.0.0",
        "port": 8081
    },
    "outbound": {
        "protocol": "TROJAN",
        "address": "0.0.0.0",
        "port": 8082,
        "secret": "123123",
        "tls": {
            "host_name": "example.com",
            "allow_insecure": true
        }
    }
}
```
### For using GRPC as transport layer
Just add GRPC to transport under inbound or outbound
```json
    "inbound": {
        "protocol": "TROJAN",
        "address": "0.0.0.0",
        "secret": "123123",
        "port": 8081,
        "tls": {
            "cert_path": "./cert.pem",
            "key_path": "./key.pem"
        },
        "transport": "GRPC"
    },
    "outbound": {
        "protocol": "DIRECT"
    }
```

## Run the program

```bash
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

## Beta stage 0.0.1 - 1.0.0(For developers)
- [x] Build up the framework for this project and support basic server side SOCKS5 protocol.

- [x] Support server side Trojan protocol for handling Trojan traffic.

- [x] Implement UDP over TCP for Trojan protocol on server side.

- [x] Implement client side Trojan protocol so that trojan-rust and be used as a Trojan client. - Work in progress.
    - [x] Implement client side Trojan protocol with TCP
    - [x] Implement client side Trojan protocol with TLS
    - [ ] -[Delayed After Beta] Implement client side Trojan protocol with UDP over TCP.

- [x] Performance profiling and bottleneck resolving. Will also include benchmarks versus other implementations. (Benchmark report coming up soon)

## Official release 0.4.0 and above(For general users)
- [ ] Improve client mode performance.

- [ ] Implement gRPC for transporting data
 
- [ ] +[Delayed After Beta] Implement client side Trojan protocol with UDP over TCP.   

- [ ] Build the package into kernel module release

- [ ] Support other protocols, gRPC, websocket etc.
