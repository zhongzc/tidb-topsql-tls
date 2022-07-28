package main

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"io/ioutil"
	"log"
	"os"
	"strings"

	"github.com/pingcap/kvproto/pkg/resource_usage_agent"
	"github.com/pingcap/tipb/go-tipb"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
)

func main() {
	caPath := os.Getenv("CA")
	if caPath == "" {
		log.Panic("CA is not set")
	}
	crtPath := os.Getenv("CRT")
	if crtPath == "" {
		log.Panic("CRT is not set")
	}
	keyPath := os.Getenv("KEY")
	if keyPath == "" {
		log.Panic("KEY is not set")
	}
	addr := os.Getenv("ADDR")
	if addr == "" {
		log.Panic("ADDR is not set")
	}
	instance := os.Getenv("INSTANCE")
	if instance == "" {
		instance = "tidb"
	}

	instance = strings.ToLower(instance)
	if instance != "tidb" && instance != "tikv" {
		log.Panic("INSTANCE must be tidb or tikv")
	}

	caCert, err := ioutil.ReadFile(caPath)
	if err != nil {
		log.Panic(err)
	}
	caCertPool := x509.NewCertPool()
	caCertPool.AppendCertsFromPEM(caCert)

	cert, err := tls.LoadX509KeyPair(crtPath, keyPath)
	if err != nil {
		log.Panic(err)
	}

	conn, err := grpc.Dial(
		addr,
		grpc.WithTransportCredentials(credentials.NewTLS(&tls.Config{
			RootCAs:      caCertPool,
			Certificates: []tls.Certificate{cert},
		})),
		grpc.WithBlock(),
	)
	if err != nil {
		log.Panic(err)
	}

	if instance == "tidb" {
		client := tipb.NewTopSQLPubSubClient(conn)

		stream, err := client.Subscribe(context.Background(), &tipb.TopSQLSubRequest{})
		if err != nil {
			log.Panic(err)
		}

		for {
			record, err := stream.Recv()
			if err != nil {
				log.Panic(err)
			}
			log.Println("recv", record)
		}
	} else if instance == "tikv" {
		client := resource_usage_agent.NewResourceMeteringPubSubClient(conn)

		stream, err := client.Subscribe(context.Background(), &resource_usage_agent.ResourceMeteringRequest{})
		if err != nil {
			log.Panic(err)
		}

		for {
			record, err := stream.Recv()
			if err != nil {
				log.Panic(err)
			}
			log.Println("recv", record)
		}
	}
}
