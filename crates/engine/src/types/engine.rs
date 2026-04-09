use common::types::{
    OrderId, UserId,
    dto::OrderDTO,
    models::{OrderSide, OrderStatus, OrderType},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum Asset {
    USDC,
    USDT,
    BTC,
    ETH,
    SOL,
}

impl Asset {
    pub fn from_str(asset_str: &str) -> Result<Asset, &'static str> {
        let asset = match asset_str {
            "USDC" => Asset::USDC,
            "USDT" => Asset::USDT,
            "BTC" => Asset::BTC,
            "ETH" => Asset::ETH,
            "SOL" => Asset::SOL,
            _ => return Err("unsupported asset"),
        };

        Ok(asset)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetPair {
    pub base: Asset,
    pub quote: Asset,
}

// #[derive(Debug, Serialize, Deserialize)]

#[derive(Debug)]
#[repr(C)]
pub struct Order {
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub order_id: OrderId,
    pub user_id: UserId,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub order_status: OrderStatus,
    pub timestamp: i64, // chrono::Utc::now().timestamp_millis();
}

// impl From<Order> for OrderDTO {
//     fn from(md: Order) -> Self {
//         Self {
//             price: md.price.mantissa() as u128,
//             price_scale: md.price.scale() as u64,
//             quantity: md.quantity.mantissa() as u128,
//             quantity_scale: md.quantity.scale() as u64,
//             filled_quantity: md.filled_quantity.mantissa() as u128,
//             order_id: md.order_id,
//             user_id: md.user_id,
//             side: md.side,
//             order_type: md.order_type,
//             order_status: md.order_status,
//             timestamp: md.timestamp,
//         }
//     }
// }

// impl From<OrderDTO> for Order {
//     fn from(dto: OrderDTO) -> Self {
//         Self {
//             price: Decimal::from_i128_with_scale(dto.price as i128, dto.price_scale as u32),
//             quantity: Decimal::from_i128_with_scale(
//                 dto.quantity as i128,
//                 dto.quantity_scale as u32,
//             ),
//             filled_quantity: Decimal::from_i128_with_scale(
//                 dto.filled_quantity as i128,
//                 dto.quantity_scale as u32,
//             ),
//             order_id: dto.order_id,
//             user_id: dto.user_id,
//             side: dto.side,
//             order_type: dto.order_type,
//             order_status: dto.order_status,
//             timestamp: dto.timestamp,
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub price: Decimal,
    pub quantity: Decimal,
    pub trade_id: i64,
    pub other_user_id: String,
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOrderResult {
    pub executed_quantity: Decimal,
    pub fills: Vec<Fill>,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CreateOrder {
//     pub market: String,
//     pub price: Decimal,
//     pub quantity: Decimal,
//     pub side: OrderSide,
//     pub user_id: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub pubsub_id: Option<Uuid>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CancelOrder {
//     pub order_id: String,
//     pub user_id: String,
//     pub price: Decimal,
//     pub side: OrderSide,
//     pub market: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub pubsub_id: Option<Uuid>,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOpenOrder {
    pub user_id: String,
    pub order_id: String,
    pub market: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubsub_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOpenOrders {
    pub user_id: String,
    pub market: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubsub_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAllOrders {
    pub user_id: String,
    pub market: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubsub_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDepth {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubsub_id: Option<Uuid>,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum OrderRequests {
//     CreateOrder(CreateOrder),
//     GetOpenOrder(GetOpenOrder),
//     CancelOrder(CancelOrder),
//     GetOpenOrders(GetOpenOrders),
//     GetDepth(GetDepth),
//     CancelAllOrders(CancelAllOrders),
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubsub_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRequests {
    CreateUser(CreateUserInput),
}
