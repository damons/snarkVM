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
