# Demo of a TLS issue for topsql

gRPC client written in Rust cannot connect to topsql pubsub endpoint with TLS, but which written in Golang works well. :(

## Deploy tidb cluster via tiup-cluster (Linux support only)

My `topo.yaml`:

```yaml
global:
  user: "zhongzc"
  deploy_dir: "/home/zhongzc/tmp/tmpcluster/deploy"
  data_dir: "/home/zhongzc/tmp/tmpcluster/data"
  enable_tls: true

pd_servers:
  - host: localhost
    client_port: 2379
    peer_port: 2380

tidb_servers:
  - host: localhost
    port: 4000
    status_port: 10080

tikv_servers:
  - host: localhost
    port: 20160
    status_port: 20180
```

Deploy & Start:
```sh
tiup cluster deploy tmp v6.1.0 topo.yaml -y && tiup cluster start tmp -y
```

## Get TLS certs

Run `tiup cluster display tmp`:

```
Starting component `cluster`: /home/zhongzc/.tiup/components/cluster/v1.8.2/tiup-cluster display tmp
Cluster type:       tidb
Cluster name:       tmp
Cluster version:    v6.1.0
Deploy user:        zhongzc
SSH type:           builtin
TLS encryption:     enabled
CA certificate:     /home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/ca.crt
Client private key: /home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.pem
Client certificate: /home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.crt
```

What we need:
```
CA certificate:     /home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/ca.crt
Client private key: /home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.pem
Client certificate: /home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.crt
```

## Build Clients

Build the Rust one:
```sh
cargo build
```

Build the Golang one:
```sh
go build -o target/debug/tidb-topsql-tls-go
```

## Run All

Connect to _TiDB_ from the **Rust client**:
```sh
CA=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/ca.crt \
CRT=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.crt \
KEY=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.pem \
ADDR=localhost:10080 \
INSTANCE=tidb \
target/debug/tidb-topsql-tls
```

Got errors:
```
recv Some(Err(RpcFailure(RpcStatus { code: 14-UNAVAILABLE, message: "failed to connect to all addresses", details: [] })))
get error, reconnecting
recv Some(Err(RpcFailure(RpcStatus { code: 14-UNAVAILABLE, message: "failed to connect to all addresses", details: [] })))
get error, reconnecting
...
```

Connect to _TiKV_ from the **Rust client**:
```sh
CA=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/ca.crt \
CRT=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.crt \
KEY=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.pem \
ADDR=localhost:20160 \
INSTANCE=tikv \
target/debug/tidb-topsql-tls
```

Succeeded:
```
recv Some(Ok(record_oneof { resource_group_tag: 0A20B95A604794F9EFF17A1A6A37D754324BE11EDE348A0D1E53DA2BC3C32D6A414212209449388A4EFBC35C8ECA1639AEC164392DF687869239F9AD16EA37887D98C42A1802 items { timestamp_sec: 1659011944 read_keys: 3 } items { timestamp_sec: 1659011947 read_keys: 3 } items { timestamp_sec: 1659011950 read_keys: 3 } items { timestamp_sec: 1659011953 read_keys: 3 } }))
recv Some(Ok(record_oneof { resource_group_tag: 0A20D0DF18D1BF1327763C0CDBC95F5EBDDB19615094EF253F4951925A7EA3F129B912207CEE07289863E5F61E4232BCA41716BC770459D034FBFC8D76DBF75F905ED96B1801 items { timestamp_sec: 1659011945 read_keys: 1 } }))
recv Some(Ok(record_oneof { resource_group_tag: 0A20D0DF18D1BF1327763C0CDBC95F5EBDDB19615094EF253F4951925A7EA3F129B912207CEE07289863E5F61E4232BCA41716BC770459D034FBFC8D76DBF75F905ED96B1802 items { timestamp_sec: 1659011945 read_keys: 1 } }))
...
``` 

Connect to _TiDB_ from the **Golang client**:
```sh
CA=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/ca.crt \
CRT=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.crt \
KEY=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.pem \
ADDR=localhost:10080 \
INSTANCE=tidb \
target/debug/tidb-topsql-tls-go
```

Succeeded:
```
2022/07/28 12:41:20 recv record:<sql_digest:"\225\005\312\313|q\016\321q%\374\306\3136i\350\335\312l\214\330\257j1\366\263\315d`L0\230" items:<timestamp_sec:1659012075 cpu_time_ms:10 stmt_exec_count:1 stmt_duration_sum_ns:628282 stmt_duration_count:1 > items:<timestamp_sec:1659012071 stmt_exec_count:1 > items:<timestamp_sec:1659012077 stmt_exec_count:1 stmt_duration_sum_ns:44931 stmt_duration_count:1 > >
2022/07/28 12:41:20 recv record:<sql_digest:"\312\360\332e$\023\250W\261\336\327x\021p0C\345'S\312\212Fn \350\234kt\331f'\203" plan_digest:"\3038\303\001~\262\344\230\014\264\234\217\200O\352\037\267\301\020J\355\3428_\022\220\234\3357g\231\263" items:<timestamp_sec:1659012071 stmt_exec_count:1 > items:<timestamp_sec:1659012074 stmt_exec_count:1 stmt_kv_exec_count:<key:"vm:20160" value:1 > stmt_duration_sum_ns:2514953 stmt_duration_count:1 > items:<timestamp_sec:1659012077 stmt_exec_count:1 stmt_kv_exec_count:<key:"vm:20160" value:1 > stmt_duration_sum_ns:2730836 stmt_duration_count:1 > items:<timestamp_sec:1659012080 cpu_time_ms:10 > >
2022/07/28 12:41:20 recv record:<sql_digest:"\271Z`G\224\371\357\361z\032j7\327T2K\341\036\3364\212\r\036S\332+\303\303-jAB" plan_digest:"\224I8\212N\373\303\\\216\312\0269\256\301d9-\366\207\206\2229\371\255\026\3527\210}\230\304*" items:<timestamp_sec:1659012071 stmt_exec_count:1 > items:<timestamp_sec:1659012074 stmt_exec_count:1 stmt_kv_exec_count:<key:"vm:20160" value:1 > > items:<timestamp_sec:1659012075 stmt_duration_sum_ns:83876598 stmt_duration_count:1 > items:<timestamp_sec:1659012077 stmt_exec_count:1 stmt_kv_exec_count:<key:"vm:20160" value:1 > stmt_duration_sum_ns:4051958 stmt_duration_count:1 > >
...
```

Connect to _TiKV_ from the **Golang client**:
```sh
CA=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/ca.crt \
CRT=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.crt \
KEY=/home/zhongzc/.tiup/storage/cluster/clusters/tmp/tls/client.pem \
ADDR=localhost:20160 \
INSTANCE=tikv \
target/debug/tidb-topsql-tls-go
```

Succeeded:
```
2022/07/28 12:43:16 recv record:<resource_group_tag:"\n \271Z`G\224\371\357\361z\032j7\327T2K\341\036\3364\212\r\036S\332+\303\303-jAB\022 \224I8\212N\373\303\\\216\312\0269\256\301d9-\366\207\206\2229\371\255\026\3527\210}\230\304*\030\002" items:<timestamp_sec:1659012160 read_keys:3 > items:<timestamp_sec:1659012163 read_keys:3 > items:<timestamp_sec:1659012166 read_keys:3 > items:<timestamp_sec:1659012169 read_keys:3 > items:<timestamp_sec:1659012172 read_keys:3 > items:<timestamp_sec:1659012175 cpu_time_ms:9 read_keys:3 > items:<timestamp_sec:1659012178 read_keys:3 > items:<timestamp_sec:1659012181 read_keys:3 > items:<timestamp_sec:1659012184 read_keys:3 > items:<timestamp_sec:1659012187 read_keys:3 > items:<timestamp_sec:1659012190 read_keys:3 > items:<timestamp_sec:1659012193 read_keys:3 > >
...
```
