mod network;
pub mod error;


use bincode2;
use common::types::Market;
use common::types::{CreateOrder, OrderDTO, OrderId, OrderSide, OrderType, UserId, create_order};
use shared_protos::markets::markets_client::MarketsClient;
use socket2::{Domain, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use zerocopy::IntoBytes;

use shared_protos::markets::markets_server::Markets;
use shared_protos::markets::{GetMarketsReply, GetMarketsRequest};
use crate::network::outbound::start::run;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    socket.set_reuse_address(true)?;
    socket.bind(&SocketAddr::from(([0, 0, 0, 0], 0)).into())?;
    socket.set_multicast_if_v4(&Ipv4Addr::UNSPECIFIED)?; // ← add this
    socket.set_multicast_loop_v4(true)?;

    // 6. Multicast destination
    let mc = SocketAddr::from((Ipv4Addr::new(239, 1, 1, 1), 9001));
    
    let data = OrderDTO::new(
        23600,
        18,
        100,
      //  18,
       // 0,
       1,
        26222,
        1,
        1,
        OrderType::MARKET,
        common::types::OrderStatus::Pending,
        OrderSide::Buy,
    );

    let market: Market = "SOL-USDC".parse().unwrap(); // parse is implemented in types

    let create_order = create_order(data);

    let create_order_bytes = create_order.as_bytes();
    socket.send_to(create_order_bytes, &mc.into())?;

    if let Err(e) = connect_grpc().await {
        eprintln!("gRPC call failed: {:?}", e);
    }
    // todo 
    //1 pass order to ringbudder which  pushes to aeron transport publisher
    println!("starting publisher");
    run();
    Ok(())
}

async fn connect_grpc() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MarketsClient::connect("http://127.0.0.1:50051").await?;

    let r = client.get_markets(GetMarketsRequest {}).await?;

    for market in r.into_inner().markets {
        println!("MARKET={:?}", market);
    }
    

    Ok(())
}
