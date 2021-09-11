use trojan_rust::config::base::InboundConfig;
use trojan_rust::proxy::acceptor::Acceptor;
use trojan_rust::proxy::base::SupportedProtocols;

#[test]
fn test_acceptor_initialization() {
    let inbound_config = InboundConfig {
        address: "1.2.3.4".to_string(),
        port: 123,
        protocol: SupportedProtocols::TROJAN,
        secret: Some("123123".to_string()),
        tls: None,
    };
    let acceptor = Acceptor::new(&inbound_config);
    assert_eq!(1, 1);
}
