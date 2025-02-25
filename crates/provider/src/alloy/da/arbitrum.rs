// This file is part of Rundler.
//
// Rundler is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later version.
//
// Rundler is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Rundler.
// If not, see https://www.gnu.org/licenses/.

use alloy_primitives::{Address, Bytes};
use alloy_provider::Provider as AlloyProvider;
use alloy_sol_types::sol;
use alloy_transport::Transport;
use rundler_types::da::{DAGasBlockData, DAGasUOData};
use tracing::instrument;
use NodeInterface::NodeInterfaceInstance;

use super::DAGasOracle;
use crate::{BlockHashOrNumber, ProviderResult};

// From https://github.com/OffchainLabs/nitro-contracts/blob/fbbcef09c95f69decabaced3da683f987902f3e2/src/node-interface/NodeInterface.sol#L112
sol! {
    #[sol(rpc)]
    interface NodeInterface {
        function gasEstimateL1Component(
            address to,
            bool contractCreation,
            bytes calldata data
        )
            external
            payable
            returns (
                uint64 gasEstimateForL1,
                uint256 baseFee,
                uint256 l1BaseFeeEstimate
            );
    }
}

pub(super) struct ArbitrumNitroDAGasOracle<AP, T> {
    node_interface: NodeInterfaceInstance<T, AP>,
}

impl<AP, T> ArbitrumNitroDAGasOracle<AP, T>
where
    AP: AlloyProvider<T>,
    T: Transport + Clone,
{
    pub(crate) fn new(oracle_address: Address, provider: AP) -> Self {
        Self {
            node_interface: NodeInterfaceInstance::new(oracle_address, provider),
        }
    }
}

#[async_trait::async_trait]
impl<AP, T> DAGasOracle for ArbitrumNitroDAGasOracle<AP, T>
where
    AP: AlloyProvider<T>,
    T: Transport + Clone,
{
    #[instrument(skip(self))]
    async fn estimate_da_gas(
        &self,
        uo_data: Bytes,
        to: Address,
        block: BlockHashOrNumber,
        _gas_price: u128,
        extra_data_len: usize,
    ) -> ProviderResult<(u128, DAGasUOData, DAGasBlockData)> {
        let data = if extra_data_len > 0 {
            super::extend_bytes_with_random(uo_data, extra_data_len)
        } else {
            uo_data
        };

        let ret = self
            .node_interface
            .gasEstimateL1Component(to, true, data)
            .block(block.into())
            .call()
            .await?;
        Ok((
            ret.gasEstimateForL1 as u128,
            DAGasUOData::Empty,
            DAGasBlockData::Empty,
        ))
    }
}
