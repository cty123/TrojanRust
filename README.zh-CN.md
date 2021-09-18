# trojan-rust

![Build](https://github.com/cty123/TrojanRust/actions/workflows/build.yml/badge.svg) ![publish](https://github.com/cty123/TrojanRust/actions/workflows/publish.yml/badge.svg) ![Version](https://img.shields.io/github/v/release/cty123/TrojanRust) ![Stage](https://img.shields.io/badge/beta-blue.svg)

Trojan-rust 是一个[Trojan协议](https://trojan-gfw.github.io/trojan/protocol.html)的Rust实现之一用于绕过[GFW](https://en.wikipedia.org/wiki/Great_Firewall)实现翻墙。这个项目专注于提升性能以及稳定性高于一的同时会尽量保证简洁易用。

# trojan-rust的优势

* 使用 [tokio-rs](https://github.com/tokio-rs/tokio) 来实现高性能的异步IO，吃满机器的IO资源。相比直接使用系统线程，Tokio 使用的轻量化线程通过减少系统线程的context switch来提升IO吞吐量，该项机制类似于Golang对于goroutine的处理。

* 使用 [rustls](https://github.com/ctz/rustls) 来处理TLS协议的通信，相比使用CVE频出Openssl来说更为安全，因为Rust编译器的特性能根除内存泄漏，并且也不需要GC。

* 项目的初衷就是为了能够最大化单机性能，所以按照计划该项目只会支持几个主流常用的翻墙协议，比如说[Trojan协议](https://trojan-gfw.github.io/trojan/protocol.html)而不是一味的添加新功能。这样能确保开发者能把更多的时间花在性能优化以及bug修复上面。另外会定期对项目做performance profiling以及benchmarking来了解现有代码的瓶颈。

* 易用性。另外一个项目初衷就是高易用性，项目最小化配置文件的代码量来做到对新人友好，又因为项目本身会比较简洁，所以这点应该不难做到。

# 编译此项目

目前项目还在非常早期阶段，目前的release里面只有Windows，Macos，Linux的64位版本的build。其他平台之后等到CI完全配置好之后会有的。另外由于Rust的缘故，编译的过程其实非常的傻瓜，所以一直会建议下载源代码自行编译。只需要安装Rust的SDK，https://www.rust-lang.org/,
    
    git clone https://github.com/cty123/TrojanRust.git
    cd ./TrojanRust
    cargo build --release

然后cargo就会自动编译所有不需要其他任何操作，编译好的文件会在 ./target/release/trojan-rust 目录下。

此外你也可以直接通过cargo运行，

    cargo run --release

如果需要显示log则要在环境变量里把RUST_LOG=info设置上，在Macos和Linux下可以这么做，

    RUST_LOG=info cargo run --release

在Windows Powershell下可以这么做，

    $Env:RUST_LOG = "info"
    cargo run --release

至于编译上的各种优化选项还没有进行过测试，所以暂时没有什么推荐的。
# Examples

## Create Certificate

要用开启TLS的话需要先生成自签证书，或者用现有注册过的，可以用以下命令来生成，

    openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

### Trojan 服务端配置样板
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
### Trojan 客户端配置样板
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

## 运行程序

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


# 项目规划 

## Beta stage 0.0.1 - 0.4.0
- [x] 搭建项目框架，先有个能用socks5服务端。

- [x] 支持服务端Trojan协议，目前只支持TCP，UDP over TCP已经有实现，但是性能堪忧，所以暂时不放出（其实源码里面有，但是全部注释掉了），在优化之中。

- [x] 支持客户端Trojan协议，这样该项目就是端到端可用了，既可以当服务端也可以当客户端。

- [x] 性能调优，已经测试完，之后会放出详细的性能测试报告

## Official release 0.4.0 以上
- [ ] 改善Trojan客户端的性能（根据性能测试得知，此项目的Trojan客户端是最关键的性能拖油瓶）

- [ ] 实现gRPC来传输数据，降低延迟，提升用户体验

- [ ] Build the package into kernel module release
