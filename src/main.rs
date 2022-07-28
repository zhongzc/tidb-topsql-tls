use futures_util::StreamExt;

fn main() {
    let ca_path = std::env::var("CA_PATH").expect("please set ca path via `CA_PATH=/path/to/ca`");
    let crt_path =
        std::env::var("CRT_PATH").expect("please set ca path via `CRT_PATH=/path/to/crt`");
    let key_path =
        std::env::var("KEY_PATH").expect("please set ca path via `KEY_PATH=/path/to/key`");
    let tidb_addr =
        std::env::var("TIDB_ADDR").expect("please set TIDB_ADDR via `TIDB_ADDR=localhost:10080`");

    let ca = std::fs::read(ca_path).expect("can not read ca path");
    let crt = std::fs::read(crt_path).expect("can not read crt path");
    let key = std::fs::read(key_path).expect("can not read key path");

    let env = std::sync::Arc::new(grpcio::Environment::new(2));
    let channel = {
        let cb = grpcio::ChannelBuilder::new(env.clone());
        cb.secure_connect(
            &tidb_addr,
            grpcio::ChannelCredentialsBuilder::new()
                .root_cert(ca)
                .cert(crt, key)
                .build(),
        )
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
                    break;
                }
            }
        }
    });
}
