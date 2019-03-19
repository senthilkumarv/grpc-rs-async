package main

import (
	"flag"
	"fmt"
	"golang.org/x/net/context"
	"google.golang.org/grpc"
	"google.golang.org/grpc/examples/helloworld/helloworld"
	"log"
	"sync"
	"time"
)

var serverAddr = flag.String("server_addr", "127.0.0.1:50051", "The server address in the format of host:port")

func sayHello(ctx context.Context, idx int) {
	conn, err := grpc.Dial(*serverAddr, grpc.WithInsecure())
	if err != nil {
		log.Panic(err)
	}
	client := helloworld.NewGreeterClient(conn)
	reply, err := client.SayHello(ctx, &helloworld.HelloRequest{
		Name: fmt.Sprintf("%v", idx),
	})
	if err != nil {
		log.Panic(err)
	}
	log.Println(reply.Message)
	wg.Done()
	_ = conn.Close()
}

var wg sync.WaitGroup

func main() {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	for i := 1; i <= 12; i++ {
		wg.Add(1)
		go sayHello(ctx, i)
	}
	wg.Wait()
	cancel()
}
