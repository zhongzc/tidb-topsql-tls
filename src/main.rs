use futures_util::StreamExt;
use std::os::unix::prelude::IntoRawFd;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{copy, split, stdin as tokio_stdin, stdout as tokio_stdout, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{self, OwnedTrustAnchor};
use tokio_rustls::{webpki, TlsConnector, TlsStream};

fn main() {
    let ca_path = std::env::var("CA").expect("please set ca path via `CA=/path/to/ca`");
    let crt_path = std::env::var("CRT").expect("please set ca path via `CRT=/path/to/crt`");
    let key_path = std::env::var("KEY").expect("please set ca path via `KEY=/path/to/key`");
    let addr = std::env::var("ADDR").expect("please set address via `ADDR=localhost:10080`");
    let instance = std::env::var("INSTANCE").unwrap_or("tidb".to_owned());

    let ca_path2 = ca_path.clone();

    let ca = std::fs::read(ca_path).expect("can not read ca path");
    let crt = std::fs::read(crt_path).expect("can not read crt path");
    let key = std::fs::read(key_path).expect("can not read key path");

    let instance = instance.to_ascii_lowercase();
    match instance.as_str() {
        "tidb" | "tikv" => {}
        _ => panic!("instance can only be tidb or tikv"),
    }

    let addr2 = addr.clone();
    tokio::spawn(async move {
        let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
        let mut pem = BufReader::new(File::open(&ca_path2).unwrap());
        let certs = rustls_pemfile::certs(&mut pem).unwrap();
        let trust_anchors = certs.iter().map(|cert| {
            let ta = webpki::TrustAnchor::try_from_cert_der(&cert[..]).unwrap();
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        });
        root_cert_store.add_server_trust_anchors(trust_anchors);
        let client_config = tokio_rustls::rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(client_config));
        let stream = TcpStream::connect(&addr2).await.unwrap();
        let outbound = connector.connect("localhost".try_into().unwrap(), stream).await.unwrap();

        let listener = tokio::net::TcpListener::bind("0.0.0.0:9527").await.unwrap();
        let (inbound, _) = listener.accept().await.unwrap();
        transfer(inbound, outbound).await.unwrap();
    });

    match instance.as_str() {
        "tidb" => {
            let env = std::sync::Arc::new(grpcio::Environment::new(2));
            let channel = {
                let cb = grpcio::ChannelBuilder::new(env.clone());
                cb.connect("127.0.0.1:9527")
            };
            futures::executor::block_on(async move {
                let client = tipb::TopSqlPubSubClient::new(channel);

                loop {
                    let mut stream = client
                        .subscribe(&tipb::TopSqlSubRequest::default())
                        .expect("can not call subscribe");

                    loop {
                        let r = stream.next().await;
                        println!("recv {:?}", &r);

                        if r.is_none() {
                            println!("end of stream, reconnecting");
                            break;
                        }

                        if r.unwrap().is_err() {
                            println!("get error, reconnecting");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            break;
                        }
                    }
                }
            });
        }
        "tikv" => {
            let env = std::sync::Arc::new(grpcio::Environment::new(2));
            let channel = {
                let cb = grpcio::ChannelBuilder::new(env.clone());
                cb.secure_connect(
                    &addr,
                    grpcio::ChannelCredentialsBuilder::new()
                        .root_cert(ca)
                        .cert(crt, key)
                        .build(),
                )
            };
            futures::executor::block_on(async move {
                let client =
                    kvproto::resource_usage_agent_grpc::ResourceMeteringPubSubClient::new(channel);

                loop {
                    let mut stream = client
                        .subscribe(
                            &kvproto::resource_usage_agent::ResourceMeteringRequest::default(),
                        )
                        .expect("can not call subscribe");

                    loop {
                        let r = stream.next().await;
                        println!("recv {:?}", &r);

                        if r.is_none() {
                            println!("end of stream, reconnecting");
                            break;
                        }

                        if r.unwrap().is_err() {
                            println!("get error, reconnecting");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            break;
                        }
                    }
                }
            });
        }
        _ => unreachable!(),
    }
}

async fn transfer(mut inbound: tokio::net::TcpStream, mut outbound: tokio_rustls::client::TlsStream<tokio::net::TcpStream>) -> Result<(), Box<dyn std::error::Error>> {
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = tokio::io::split(outbound);

    let client_to_server = async {
       tokio::io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        tokio::io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}