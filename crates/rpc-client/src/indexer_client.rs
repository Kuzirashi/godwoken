#![allow(clippy::mutable_key_type)]

use crate::indexer_types::{Cell, Order, Pagination, ScriptType, SearchKey};
use crate::utils::{to_result, DEFAULT_QUERY_LIMIT};
use anyhow::{anyhow, Result};
use async_jsonrpc_client::{HttpClient, Params as ClientParams, Transport};
use ckb_types::prelude::Entity;
use gw_jsonrpc_types::ckb_jsonrpc_types::Uint32;
use gw_types::{
    offchain::CellInfo,
    packed::{CellOutput, OutPoint, Script},
    prelude::*,
};
use serde_json::json;

use std::collections::HashSet;

#[derive(Clone)]
pub struct CKBIndexerClient(HttpClient);

impl CKBIndexerClient {
    pub fn new(ckb_indexer_client: HttpClient) -> Self {
        Self(ckb_indexer_client)
    }

    pub fn with_url(url: &str) -> Result<Self> {
        let client = HttpClient::new(url)?;
        Ok(Self::new(client))
    }

    pub fn client(&self) -> &HttpClient {
        &self.0
    }

    /// query payment cells, the returned cells should provide at least required_capacity fee,
    /// and the remained fees should be enough to cover a charge cell
    pub async fn query_payment_cells(
        &self,
        lock: Script,
        required_capacity: u64,
        taken_outpoints: &HashSet<OutPoint>,
    ) -> Result<Vec<CellInfo>> {
        let search_key = SearchKey {
            script: {
                let lock = ckb_types::packed::Script::new_unchecked(lock.as_bytes());
                lock.into()
            },
            script_type: ScriptType::Lock,
            filter: None,
        };
        let order = Order::Desc;
        let limit = Uint32::from(DEFAULT_QUERY_LIMIT as u32);

        let mut collected_cells = Vec::new();
        let mut collected_capacity = 0u64;
        let mut cursor = None;
        while collected_capacity < required_capacity {
            let cells: Pagination<Cell> = to_result(
                self.0
                    .request(
                        "get_cells",
                        Some(ClientParams::Array(vec![
                            json!(search_key),
                            json!(order),
                            json!(limit),
                            json!(cursor),
                        ])),
                    )
                    .await?,
            )?;

            if cells.last_cursor.is_empty() {
                return Err(anyhow!("no enough payment cells"));
            }
            cursor = Some(cells.last_cursor);

            let cells = cells.objects.into_iter().filter_map(|cell| {
                let out_point = {
                    let out_point: ckb_types::packed::OutPoint = cell.out_point.into();
                    OutPoint::new_unchecked(out_point.as_bytes())
                };
                // delete cells with data & type
                if !cell.output_data.is_empty()
                    || cell.output.type_.is_some()
                    || taken_outpoints.contains(&out_point)
                {
                    return None;
                }
                let output = {
                    let output: ckb_types::packed::CellOutput = cell.output.into();
                    CellOutput::new_unchecked(output.as_bytes())
                };
                let data = cell.output_data.into_bytes();
                Some(CellInfo {
                    out_point,
                    output,
                    data,
                })
            });

            // collect least cells
            for cell in cells {
                collected_capacity =
                    collected_capacity.saturating_add(cell.output.capacity().unpack());
                collected_cells.push(cell);
                if collected_capacity >= required_capacity {
                    break;
                }
            }
        }
        Ok(collected_cells)
    }
}
