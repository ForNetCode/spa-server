use futures::future::join;
use hyper::Server;
use socket2::{Domain, Socket, Type};
use std::convert::Infallible;
use std::future::Future;
use std::net::{SocketAddr, TcpListener};
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::oneshot;
use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //test for reuse port, it's ok!
    //let _ret = join(start_test_server("hello world_1"), start_test_server("hello world_2")).await;

    //test if new server can replace old server. in Linux it's ok, in MacOS, the server_1 will stop receiving requests once server 2 bind ok.
    let (tx, rx) = oneshot::channel::<()>();
    let server_1 = start_test_server("hello 1", async {
        rx.await.ok();
        println!("server 1 Close")
    });
    let server_2 = async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("begin to start server 2");
        let _re = join(
            start_test_server("hello 2", tokio::time::sleep(Duration::from_secs(100))),
            async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                println!("begin to close server 1");
                tx.send(()).ok();
            },
        )
        .await;
    };
    let _re = join(server_1, server_2).await;
    Ok(())
}

async fn start_test_server(
    resp: &'static str,
    signal: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let routes = warp::any().map(move || resp);
    let listener = get_socket()?;
    let svc = warp::service(routes);
    let make_svc = hyper::service::make_service_fn(|_: _| {
        let svc = svc.clone();
        async move { Ok::<_, Infallible>(svc) }
    });
    let server = Server::from_tcp(listener)?;
    server
        .serve(make_svc)
        .with_graceful_shutdown(signal)
        .await?;
    Ok(())
}

fn get_socket() -> anyhow::Result<TcpListener> {
    let addr = SocketAddr::from_str("127.0.0.1:3030")?;
    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;
    socket.set_nodelay(true)?;
    // socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;
    let listener: TcpListener = socket.into();
    Ok(listener)
}
// just test graceful download
#[allow(dead_code)]
async fn simple_test() -> anyhow::Result<()> {
    let routes = warp::any().map(|| "hello world");
    let (tx, rx) = oneshot::channel::<()>();
    let (_adder, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], 3030), async {
            rx.await.ok();
        });
    let cancel_task = async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("begin to end");
        tx.send(())
    };

    let _ret = join(server, cancel_task).await;
    Ok(())
}
