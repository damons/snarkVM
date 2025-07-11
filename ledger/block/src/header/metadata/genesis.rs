// Copyright (c) 2019-2025 Provable Inc.
// This file is part of the snarkVM library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:

// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

impl<N: Network> Metadata<N> {
    /// Initializes the genesis metadata.
    pub fn genesis() -> Result<Self> {
        // Prepare a genesis metadata.
        let network = N::ID;
        let round = 0;
        let height = 0;
        let cumulative_weight = 0;
        let cumulative_proof_target = 0;
        let coinbase_target = N::GENESIS_COINBASE_TARGET;
        let proof_target = N::GENESIS_PROOF_TARGET;
        let last_coinbase_target = N::GENESIS_COINBASE_TARGET;
        let last_coinbase_timestamp = N::GENESIS_TIMESTAMP;
        let timestamp = N::GENESIS_TIMESTAMP;

        // Return the genesis metadata.
        Self::new(
            network,
            round,
            height,
            cumulative_weight,
            cumulative_proof_target,
            coinbase_target,
            proof_target,
            last_coinbase_target,
            last_coinbase_timestamp,
            timestamp,
        )
    }

    /// Returns `true` if the metadata is a genesis metadata.i
    ///
    /// Return an error if the block is at height 0 but not a valid genesis header.
    pub fn is_genesis(&self) -> Result<bool> {
        // Only blocks at height 0 are genesis blocks.
        if self.round != 0u64 {
            return Ok(false);
        }

        // Check the genesis block header is valid otherwise.
        ensure!(self.network == N::ID, "Invalid network ID");
        ensure!(self.round == 0, "Invalid round");
        ensure!(self.cumulative_weight == 0u128, "Invalid cumulative weight");
        ensure!(self.cumulative_proof_target == 0u128, "Invalid proof target");
        ensure!(self.coinbase_target == N::GENESIS_COINBASE_TARGET, "Invalid coinbase target");
        ensure!(self.proof_target == N::GENESIS_PROOF_TARGET, "Invalid proof target");
        ensure!(self.last_coinbase_target == N::GENESIS_COINBASE_TARGET, "Invalid last coinbase target");
        ensure!(self.last_coinbase_timestamp == N::GENESIS_TIMESTAMP, "Invalid last coinbase timestamp");
        ensure!(self.timestamp == N::GENESIS_TIMESTAMP, "Invalid timestamp");

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::network::MainnetV0;

    type CurrentNetwork = MainnetV0;

    /// Returns the expected metadata size by summing its subcomponent sizes.
    /// Update this method if the contents of the metadata have changed.
    fn get_expected_size() -> usize {
        // Metadata size.
        1 + 8 + 4 + 16 + 16 + 8 + 8 + 8 + 8 + 8
            // Add an additional 2 bytes for versioning.
            + 2
    }

    #[test]
    fn test_genesis_metadata_size() {
        let rng = &mut TestRng::default();

        // Prepare the expected size.
        let expected_size = get_expected_size();
        // Prepare the genesis metadata.
        let genesis_metadata = crate::header::metadata::test_helpers::sample_block_metadata(rng);
        // Ensure the size of the genesis metadata is correct.
        assert_eq!(expected_size, genesis_metadata.to_bytes_le().unwrap().len());
    }

    #[test]
    fn test_genesis_metadata() {
        let rng = &mut TestRng::default();

        // Prepare the genesis metadata.
        let metadata = crate::header::metadata::test_helpers::sample_block_metadata(rng);
        // Ensure the metadata is a genesis metadata.
        assert!(metadata.is_genesis().unwrap());

        // Ensure the genesis block contains the following.
        assert_eq!(metadata.network(), CurrentNetwork::ID);
        assert_eq!(metadata.round(), 0);
        assert_eq!(metadata.height(), 0);
        assert_eq!(metadata.cumulative_weight(), 0);
        assert_eq!(metadata.cumulative_proof_target(), 0);
        assert_eq!(metadata.coinbase_target(), CurrentNetwork::GENESIS_COINBASE_TARGET);
        assert_eq!(metadata.proof_target(), CurrentNetwork::GENESIS_PROOF_TARGET);
        assert_eq!(metadata.last_coinbase_target(), CurrentNetwork::GENESIS_COINBASE_TARGET);
        assert_eq!(metadata.last_coinbase_timestamp(), CurrentNetwork::GENESIS_TIMESTAMP);
        assert_eq!(metadata.timestamp(), CurrentNetwork::GENESIS_TIMESTAMP);
    }
}
