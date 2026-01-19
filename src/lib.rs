mod abi;
#[allow(unused)]
mod pb;
use hex_literal::hex;
use pb::contract::v1 as contract;
use substreams::Hex;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

#[allow(unused_imports)] // Might not be needed depending on actual ABI, hence the allow
use {num_traits::cast::ToPrimitive, std::str::FromStr, substreams::scalar::BigDecimal};

substreams_ethereum::init!();

const CONDITIONAL_TOKENS_TRACKED_CONTRACT: [u8; 20] = hex!("4d97dcd97ec945f40cf65f87097ace5ea0476045");

fn map_conditional_tokens_events(blk: &eth::Block, events: &mut contract::Events) {
    events.conditional_tokens_transfer_batches.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == CONDITIONAL_TOKENS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::conditional_tokens_contract::events::TransferBatch::match_and_decode(log) {
                        return Some(contract::ConditionalTokensTransferBatch {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            from: event.from,
                            ids: event.ids.into_iter().map(|x| x.to_string()).collect::<Vec<_>>(),
                            operator: event.operator,
                            to: event.to,
                            values: event.values.into_iter().map(|x| x.to_string()).collect::<Vec<_>>(),
                        });
                    }

                    None
                })
        })
        .collect());
    events.conditional_tokens_transfer_singles.append(&mut blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter()
                .filter(|log| log.address == CONDITIONAL_TOKENS_TRACKED_CONTRACT)
                .filter_map(|log| {
                    if let Some(event) = abi::conditional_tokens_contract::events::TransferSingle::match_and_decode(log) {
                        return Some(contract::ConditionalTokensTransferSingle {
                            evt_tx_hash: Hex(&view.transaction.hash).to_string(),
                            evt_index: log.block_index,
                            evt_block_time: Some(blk.timestamp().to_owned()),
                            evt_block_number: blk.number,
                            from: event.from,
                            id: event.id.to_string(),
                            operator: event.operator,
                            to: event.to,
                            value: event.value.to_string(),
                        });
                    }

                    None
                })
        })
        .collect());
}
#[substreams::handlers::map]
fn map_events(blk: eth::Block) -> Result<contract::Events, substreams::errors::Error> {
    let mut events = contract::Events::default();
    map_conditional_tokens_events(&blk, &mut events);
    Ok(events)
}

#[substreams::handlers::map]
fn db_out(events: contract::Events) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut tables = Tables::new();

    for event in events.conditional_tokens_transfer_singles {
        let pk = format!("{}-{}", event.evt_tx_hash, event.evt_index);
        tables
            .create_row("transfers", pk)
            .set("evt_tx_hash", event.evt_tx_hash)
            .set("evt_index", event.evt_index)
            .set("evt_block_time", event.evt_block_time.as_ref().unwrap())
            .set("evt_block_number", event.evt_block_number)
            .set("from_addr", Hex(&event.from).to_string())
            .set("to_addr", Hex(&event.to).to_string())
            .set("operator", Hex(&event.operator).to_string())
            .set("token_id", event.id)
            .set("value", event.value)
            .set("index", 0);
    }

    for event in events.conditional_tokens_transfer_batches {
        for (i, (id, value)) in event.ids.iter().zip(event.values.iter()).enumerate() {
            let pk = format!("{}-{}-{}", event.evt_tx_hash, event.evt_index, i);
            tables
                .create_row("transfers", pk)
                .set("evt_tx_hash", &event.evt_tx_hash)
                .set("evt_index", event.evt_index)
                .set("evt_block_time", event.evt_block_time.as_ref().unwrap())
                .set("evt_block_number", event.evt_block_number)
                .set("from_addr", Hex(&event.from).to_string())
                .set("to_addr", Hex(&event.to).to_string())
                .set("operator", Hex(&event.operator).to_string())
                .set("token_id", id)
                .set("value", value)
                .set("index", i as i32);
        }
    }

    Ok(tables.to_database_changes())
}
