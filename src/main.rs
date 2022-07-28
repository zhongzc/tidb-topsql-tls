use futures_util::StreamExt;

fn main() {
    let ca_path = std::env::var("CA").expect("please set ca path via `CA=/path/to/ca`");
    let crt_path = std::env::var("CRT").expect("please set ca path via `CRT=/path/to/crt`");
    let key_path = std::env::var("KEY").expect("please set ca path via `KEY=/path/to/key`");
    let addr = std::env::var("ADDR").expect("please set address via `ADDR=localhost:10080`");
    let instance = std::env::var("INSTANCE").unwrap_or("tidb".to_owned());

    let ca = std::fs::read(ca_path).expect("can not read ca path");
    let crt = std::fs::read(crt_path).expect("can not read crt path");
    let key = std::fs::read(key_path).expect("can not read key path");

    let instance = instance.to_ascii_lowercase();
    match instance.as_str() {
        "tidb" | "tikv" => {}
        _ => panic!("instance can only be tidb or tikv"),
    }

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

    match instance.as_str() {
        "tidb" => {
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
