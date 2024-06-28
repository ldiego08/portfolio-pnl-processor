use std::{env, io};

use events::{process_pnl, IncomingAssetCollectionFloorPriceEvent, IncomingAssetTradeEvent};
use json::{read_json_file, write_json_file};

mod events;
mod wallet;
mod nft;
mod json;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <trade_events_file> <floor_price_events_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let trade_events_file_path = &args[1];
    let floor_price_events_file_path = &args[2];
    let output_file_path = &args[3];

    let trade_events: Vec<IncomingAssetTradeEvent> = read_json_file(trade_events_file_path)?;
    let floor_price_events: Vec<IncomingAssetCollectionFloorPriceEvent> = read_json_file(floor_price_events_file_path)?;

    let pnl_events = process_pnl(trade_events, floor_price_events);

    write_json_file(output_file_path, &pnl_events)?;

    Ok(())
}
