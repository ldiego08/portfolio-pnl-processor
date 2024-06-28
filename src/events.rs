use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{nft::NftInfo, wallet::WalletPnL};

#[derive(Debug, Deserialize)]
pub(crate) struct IncomingAssetTradeEvent {
    pub time: u32,
    pub buyer: String,
    pub seller: String,
    pub nft: String,
    pub collection: String,
    pub price: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IncomingAssetCollectionFloorPriceEvent {
    pub time: u32,
    pub collection: String,
    #[serde(rename = "floorPrice")]
    pub new_floor_price: f64,
}

#[derive(Debug, Serialize)]
pub(crate) struct OutgoingWalletPnLEvent {
    pub time: u32,
    pub wallet: String,
    pub realized: f64,
    pub unrealized: f64,
}

fn process_trade_event(
  trade_event: &IncomingAssetTradeEvent,
  floor_prices: &HashMap<String, f64>,
  wallet_pnl: &mut HashMap<String, WalletPnL>,
  nft_owners: &mut HashMap<String, NftInfo>,
  events: &mut Vec<OutgoingWalletPnLEvent>,
) {
  let IncomingAssetTradeEvent {
      time, 
      buyer,
      seller,
      nft,
      collection,
      price
  } = trade_event;

  let current_floor_price = floor_prices.get(collection).copied().unwrap_or(0.0);

  wallet_pnl
      .entry(buyer.clone())
      .or_default()
      .unrealized += current_floor_price - price;

  if let Some(owner_info) = nft_owners.get(nft) {
      let buy_price = owner_info.price;

      wallet_pnl
          .entry(seller.clone())
          .or_default()
          .realized += price - buy_price;

      wallet_pnl
          .entry(seller.clone())
          .or_default()
          .unrealized -= current_floor_price - buy_price;
  }

  nft_owners.insert(nft.clone(), NftInfo { 
      wallet: buyer.clone(), 
      price: *price, 
      collection: collection.clone() 
  });

  if let Some(buyer_pnl) = wallet_pnl.get(buyer) {
      events.push(OutgoingWalletPnLEvent { 
          time: *time, 
          wallet: buyer.clone(), 
          realized: buyer_pnl.realized, 
          unrealized: buyer_pnl.unrealized 
      });
  }

  if let Some(seller_pnl) = wallet_pnl.get(seller) {
      events.push(OutgoingWalletPnLEvent {
          time: *time,
          wallet: seller.clone(),
          realized: seller_pnl.realized,
          unrealized: seller_pnl.unrealized,
      });
  }
}

fn process_floor_price_event(
  floor_price_event: &IncomingAssetCollectionFloorPriceEvent,
  floor_prices: &mut HashMap<String, f64>,
  wallet_pnl: &mut HashMap<String, WalletPnL>,
  nft_owners: &mut HashMap<String, NftInfo>,
  events: &mut Vec<OutgoingWalletPnLEvent>,
) {
  let IncomingAssetCollectionFloorPriceEvent { 
      time, 
      collection, 
      new_floor_price 
  } = floor_price_event;

  let previous_floor_price = *floor_prices.get(collection).unwrap_or(&0.0);

  floor_prices.insert(collection.clone(), *new_floor_price);

  for (_, owner_info) in nft_owners {
      if &owner_info.collection == collection {
          let owner_wallet = &owner_info.wallet;
          
          wallet_pnl
              .entry(owner_wallet.clone())
              .or_default()
              .unrealized += new_floor_price - previous_floor_price;

          if let Some(owner_pnl) = wallet_pnl.get(owner_wallet) {
              events.push(OutgoingWalletPnLEvent { 
                  time: *time, 
                  wallet: owner_wallet.clone(), 
                  realized: owner_pnl.realized, 
                  unrealized: owner_pnl.unrealized 
              });
          }
      }
  }
}

pub fn process_pnl(
  trade_events: Vec<IncomingAssetTradeEvent>,
  floor_price_events: Vec<IncomingAssetCollectionFloorPriceEvent>,
) -> Vec<OutgoingWalletPnLEvent> {
  let mut floor_prices: HashMap<String, f64> = HashMap::new();
  let mut wallet_pnl: HashMap<String, WalletPnL> = HashMap::new();
  let mut nft_owners: HashMap<String, NftInfo> = HashMap::new();
  
  let mut events = vec![];

  let mut trade_event_index = 0;
  let mut floor_price_event_index = 0;

  while trade_event_index < trade_events.len() || floor_price_event_index < floor_price_events.len() {
      let next_trade_event = trade_events.get(trade_event_index);
      let next_floor_price_event = floor_price_events.get(floor_price_event_index);

      if trade_event_index >= trade_events.len() ||
         (next_floor_price_event.is_some() && next_floor_price_event.unwrap().time < next_trade_event.unwrap().time) {
          if let Some(floor_price_event) = next_floor_price_event {
              process_floor_price_event(floor_price_event, &mut floor_prices, &mut wallet_pnl, &mut nft_owners, &mut events);
              floor_price_event_index += 1;
          }
      } else {
          if let Some(trade_event) = next_trade_event {
              process_trade_event(trade_event, &floor_prices, &mut wallet_pnl, &mut nft_owners, &mut events);
              trade_event_index += 1;
          }
      }
  }

  events
}