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

use snarkvm_console::prelude::{Network as _, TestRng};
use snarkvm_ledger::{Ledger, store::helpers::rocksdb::ConsensusDB, test_helpers::TestChainBuilder};

use aleo_std::StorageMode;

use anyhow::{Context, Result};
use clap::Parser;

type Network = snarkvm_console::prelude::TestnetV0;

#[derive(Parser)]
struct Args {
    num_validators: usize,
    num_blocks: usize,
    #[clap(long)]
    genesis_path: Option<String>,
}

/// Removes an existing ledger (if any) from the filesystem.
fn remove_ledger(network: u16) -> Result<()> {
    let path = aleo_std::aleo_ledger_dir(network, &StorageMode::Development(0));

    if path.exists() {
        std::fs::remove_dir_all(&path).with_context(|| "Failed to remove existing ledger")?;

        println!("Remove existing ledger data at {path:?}");
    }

    Ok(())
}

fn main() -> Result<()> {
    // Enable logging.
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut rng = TestRng::default();

    remove_ledger(Network::ID)?;

    let num_validators = args.num_validators;
    let num_blocks = args.num_blocks;

    println!("Initializing test chain builder with {num_validators} validators");
    let mut builder = match args.genesis_path {
        Some(genesis_path) => {
            TestChainBuilder::<Network>::new_with_quorum_size_and_genesis_block(num_validators, genesis_path)
                .with_context(|| "Failed to set up test chain builder")?
        }
        None => TestChainBuilder::<Network>::new_with_quorum_size(num_validators, &mut rng)
            .with_context(|| "Failed to set up test chain builder")?,
    };

    println!("Generating {num_blocks} blocks");

    let mut pos = 0;
    let mut blocks = vec![];

    while blocks.len() < num_blocks {
        let batch_size = (num_blocks - blocks.len()).min(100);
        let mut batch = builder.generate_blocks(batch_size, &mut rng).with_context(|| "Failed to generate blocks")?;

        println!("Generated {pos} of {num_blocks} blocks");
        pos += batch_size;
        blocks.append(&mut batch);
    }

    println!("Done. Storing blocks to disk.");
    let ledger =
        Ledger::<Network, ConsensusDB<Network>>::load(builder.genesis_block().clone(), StorageMode::Development(0))
            .with_context(|| "Failed to initialize ledger")?;

    // Ensure there is only one active ledger at a time.
    drop(builder);

    for block in &blocks {
        ledger.advance_to_next_block(block)?;
    }

    Ok(())
}
