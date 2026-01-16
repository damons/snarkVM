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

use crate::vm::test_helpers::*;

use snarkvm_synthesizer_program::Program;

use console::network::ConsensusVersion;
use snarkvm_utilities::TestRng;

// This test verifies that a large program that is over the previous size limit can be deployed after V14.
#[test]
fn test_deploy_large_program_v14() {
    // Initialize an RNG.
    let rng = &mut TestRng::default();

    let large_program = Program::from_str(include_str!("./resources/large_program.aleo")).unwrap();

    println!("Large program size (string size): {}", large_program.to_string().len());

    // Initialize a new caller.
    let caller_private_key = crate::vm::test_helpers::sample_genesis_private_key(rng);

    // Initialize the VM at the V13 height.
    let v13_height = CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V13).unwrap();
    let vm = crate::vm::test_helpers::sample_vm_at_height(v13_height, rng);

    // Ensure that the program is too large to be deployed at V13.
    let deployment = vm.deploy(&caller_private_key, &large_program, None, 0, None, rng).unwrap();
    let deployment_id = deployment.id();
    assert!(vm.check_transaction(&deployment, None, rng).is_err());
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng).unwrap();
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids(), &[deployment_id]);
    assert_eq!(block.aborted_transaction_ids().len(), 1);

    // Advance to the V14 height.
    let v14_height = CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V14).unwrap();
    while vm.block_store().current_block_height() < v14_height {
        let block = sample_next_block(&vm, &caller_private_key, &[], rng).unwrap();
        vm.add_next_block(&block).unwrap();
    }

    // Ensure that the program can be deployed at V14.
    let deployment = vm.deploy(&caller_private_key, &large_program, None, 0, None, rng).unwrap();
    assert!(vm.check_transaction(&deployment, None, rng).is_ok());
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng).unwrap();
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
}

/// Generates a large program string that exceeds the V13 size limit (100KB) but fits within V14 (512KB).
fn generate_large_program() -> String {
    let mut program = String::from(
        "program large_program_generated.aleo;

constructor:
    assert.eq true true;

function compute:
    input r0 as u64.public;
",
    );

    // Generate cast instructions to create large arrays.
    // Each cast with 32 elements is ~200+ bytes, so we need fewer instructions.
    let mut reg = 1u32;
    while program.len() < 110_000 {
        // Create a 32-element array from r0.
        program.push_str(&format!(
            "    cast r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 r0 into r{reg} as [u64; 32u32];\n"
        ));
        reg += 1;
    }

    program
}

// This test verifies serialization round-trips for large program deployment transactions at V13 and V14.
#[test]
fn test_deploy_large_program_v14_serialization() {
    // Initialize an RNG.
    let rng = &mut TestRng::default();

    let large_program_str = generate_large_program();
    let large_program = Program::from_str(&large_program_str).unwrap();

    println!("Generated large program size: {} bytes", large_program_str.len());

    // Initialize a new caller.
    let caller_private_key = crate::vm::test_helpers::sample_genesis_private_key(rng);

    // Initialize the VM at the V13 height.
    let v13_height = CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V13).unwrap();
    let vm = crate::vm::test_helpers::sample_vm_at_height(v13_height, rng);

    // Create a deployment transaction for the large program at V13.
    let deployment = vm.deploy(&caller_private_key, &large_program, None, 0, None, rng).unwrap();
    let deployment_id = deployment.id();

    // Verify bytes serialization round-trip at V13.
    let deployment_bytes = deployment.to_bytes_le().unwrap();
    let recovered_from_bytes = Transaction::<CurrentNetwork>::read_le(&deployment_bytes[..]).unwrap();
    assert_eq!(deployment, recovered_from_bytes);

    // Verify string (JSON) serialization round-trip at V13.
    let deployment_string = deployment.to_string();
    let recovered_from_string = Transaction::<CurrentNetwork>::from_str(&deployment_string).unwrap();
    assert_eq!(deployment, recovered_from_string);

    // Ensure that the program is too large to pass check_transaction at V13.
    assert!(vm.check_transaction(&deployment, None, rng).is_err());

    // Create block and verify the transaction is aborted.
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng).unwrap();
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids(), &[deployment_id]);
    assert_eq!(block.aborted_transaction_ids().len(), 1);

    // Advance to the V14 height.
    let v14_height = CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V14).unwrap();
    while vm.block_store().current_block_height() < v14_height {
        let block = sample_next_block(&vm, &caller_private_key, &[], rng).unwrap();
        vm.add_next_block(&block).unwrap();
    }

    // Create a new deployment transaction for the large program at V14.
    let deployment = vm.deploy(&caller_private_key, &large_program, None, 0, None, rng).unwrap();

    // Verify bytes serialization round-trip at V14.
    let deployment_bytes = deployment.to_bytes_le().unwrap();
    let recovered_from_bytes = Transaction::<CurrentNetwork>::read_le(&deployment_bytes[..]).unwrap();
    assert_eq!(deployment, recovered_from_bytes);

    // Verify string (JSON) serialization round-trip at V14.
    let deployment_string = deployment.to_string();
    let recovered_from_string = Transaction::<CurrentNetwork>::from_str(&deployment_string).unwrap();
    assert_eq!(deployment, recovered_from_string);

    // Ensure that the program passes check_transaction at V14.
    assert!(vm.check_transaction(&deployment, None, rng).is_ok());

    // Create block and verify the transaction is accepted.
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng).unwrap();
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);

    // Add block to the VM.
    vm.add_next_block(&block).unwrap();
}
