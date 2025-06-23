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

use console::{account::ViewKey, network::ConsensusVersion};
use ledger_store::ConsensusStore;
use synthesizer_program::Program;

use aleo_std::StorageMode;

// This test checks that:
//  - an existing program cannot be redeployed before `ConsensusVersion::V8`
//  - an existing program cannot be redeployed with different code after `ConsensusVersion::V8`
//  - an existing program can be redeployed with the same code after `ConsensusVersion::V8`
//  - an existing program can only be redeployed once after `ConsensusVersion::V8`
//  - after `ConsensusVersion::V8`, existing programs cannot be executed until they are redeployed.
//  - the VM can be loaded from a store at the very end.
#[test]
fn test_redeployment() -> Result<()> {
    let rng = &mut TestRng::default();

    // Initialize a new caller.
    let caller_private_key = sample_genesis_private_key(rng);

    // Initialize the storage.
    let store = ConsensusStore::<CurrentNetwork, LedgerType>::open(StorageMode::new_test(None)).unwrap();

    // Initialize the VM.
    let vm = VM::<CurrentNetwork, LedgerType>::from(store.clone())?;
    let genesis = sample_genesis_block(rng);
    vm.add_next_block(&genesis)?;

    // Get the genesis private key.
    let genesis_private_key = sample_genesis_private_key(rng);

    // Advance the VM to 3 blocks before `ConsensusVersion::V8`.
    let desired_height = CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V8)? - 3;
    for _ in 0..desired_height {
        let block = sample_next_block(&vm, &genesis_private_key, &[], rng).unwrap();
        vm.add_next_block(&block).unwrap();
    }

    // Initialize the programs
    let program = Program::from_str(
        r"
program test_redeployment.aleo;
function dummy:
    ",
    )?;
    let program_diff = Program::from_str(
        r"
program test_redeployment.aleo;
function dummy:
function dummy2:
    ",
    )?;

    // Attempt to deploy the program.
    let deployment = vm.deploy(&caller_private_key, &program, None, 0, None, rng)?;
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng)?;
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block)?;

    // Execute the program.
    let execute = vm.execute(
        &caller_private_key,
        ("test_redeployment.aleo", "dummy"),
        Vec::<Value<CurrentNetwork>>::new().iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block)?;

    // Attempt to redeploy the program before `ConsensusVersion::V8`.
    let deployment = vm.deploy(&caller_private_key, &program, None, 0, None, rng)?;
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng)?;
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 1);
    vm.add_next_block(&block)?;

    // Check that the consensus version is `V8`.
    let block_height = vm.store.block_store().current_block_height();
    let consensus_version = CurrentNetwork::CONSENSUS_VERSION(block_height)?;
    assert_eq!(consensus_version, ConsensusVersion::V8);

    // Attempt to execute the program after `ConsensusVersion::V8`.
    let execute = vm.execute(
        &caller_private_key,
        ("test_redeployment.aleo", "dummy"),
        Vec::<Value<CurrentNetwork>>::new().iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 1);
    vm.add_next_block(&block)?;

    // Attempt to redeploy the program with different code after `ConsensusVersion::V8`.
    let result = vm.deploy(&caller_private_key, &program_diff, None, 0, None, rng);
    assert!(result.is_err());

    // Redeploy the program with the same code after `ConsensusVersion::V8`.
    let deployment = vm.deploy(&caller_private_key, &program, None, 0, None, rng)?;
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng)?;
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block)?;

    // Verify that the program can be executed after redeployment.
    let execute = vm.execute(
        &caller_private_key,
        ("test_redeployment.aleo", "dummy"),
        Vec::<Value<CurrentNetwork>>::new().iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block)?;

    // Attempt to redeploy the program again after `ConsensusVersion::V8`.
    let deployment = vm.deploy(&caller_private_key, &program, None, 0, None, rng)?;
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng)?;
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 1);
    vm.add_next_block(&block)?;

    // Drop the VM.
    drop(vm);

    // Load the VM from the store.
    let vm = VM::<CurrentNetwork, LedgerType>::from(store)?;

    // Check that the latest block.
    let latest_block = vm.store.block_store().current_block_height();
    assert_eq!(latest_block, CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V8)? + 4);

    // Check that the program can still be executed.
    let execute = vm.execute(
        &caller_private_key,
        ("test_redeployment.aleo", "dummy"),
        Vec::<Value<CurrentNetwork>>::new().iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block)?;

    Ok(())
}

// This test checks that the `credits.aleo` program cannot be redeployed.
#[test]
fn test_credits_cannot_be_redeployed() -> Result<()> {
    let rng = &mut TestRng::default();

    // Initialize a new caller.
    let caller_private_key = sample_genesis_private_key(rng);

    // Initialize the VM.
    let vm = sample_vm_at_height(CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V8)?, rng);

    // Initialize the `credits.aleo` program.
    let program = Program::credits()?;

    // Attempt to redeploy the `credits.aleo` program.
    let result = vm.deploy(&caller_private_key, &program, None, 0, None, rng);
    assert!(result.is_err());

    Ok(())
}

// This test:
// - verifies that a program calling `credits.aleo/upgrade` can be deployed before `ConsensusVersion::V8`
// - verifies that a program calling `credits.aleo/upgrade` cannot be executed before `ConsensusVersion::V8`
// - verifies that `credits.aleo/upgrade` cannot be executed before `ConsensusVersion::V8`
// - verifies that a program calling `credits.aleo/upgrade` cannot be executed after `ConsensusVersion::V8`
// - verifies that a program calling `credits.aleo/upgrade` cannot be deployed after `ConsensusVersion::V8`.
// - verifies that `credits.aleo/upgrade` can be executed after `ConsensusVersion::V8`.
#[test]
fn test_credits_upgrade() -> Result<()> {
    let rng = &mut TestRng::default();

    // Initialize a new caller.
    let caller_private_key = sample_genesis_private_key(rng);

    // Initialize the VM.
    let vm = sample_vm_at_height(CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V8)? - 4, rng);

    // A helper closure to create a program with an upgrade call.
    let sample_program = |i: usize| {
        Program::from_str(&format!(
            r"
import credits.aleo;

program test_upgrade_call_{i}.aleo;

function run:
    input r0 as credits.aleo/credits.record;
    call credits.aleo/upgrade r0 into r1 r2;
    async run r2 into r3;
    output r1 as credits.aleo/credits.record;
    output r3 as test_upgrade_call_{i}.aleo/run.future;
    
finalize run:
    input r0 as credits.aleo/upgrade.future;
    await r0;
    ",
        ))
    };

    // Deploy the program before `ConsensusVersion::V8`.
    let deployment = vm.deploy(&caller_private_key, &sample_program(0)?, None, 0, None, rng)?;
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng)?;
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block)?;

    // Extract a record from the genesis block and `split` it into two smaller records.
    let genesis_hash = vm.store.block_store().get_block_hash(0)?.unwrap();
    let genesis_block = vm.store.block_store().get_block(&genesis_hash)?.unwrap();
    let caller_view_key = ViewKey::try_from(&caller_private_key)?;
    let mut records = genesis_block
        .transactions()
        .records()
        .map(|(_, record)| record.decrypt(&caller_view_key))
        .collect::<Result<Vec<_>>>()?;
    let record = records.pop().unwrap();
    let execute = vm.execute(
        &caller_private_key,
        ("credits.aleo", "split"),
        vec![Value::Record(record), Value::from_str("12345u64")?].into_iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block)?;

    // Attempt to use the first record for the upgrade call, before `ConsensusVersion::V8`.
    let record = block.transactions().records().next().unwrap().1.decrypt(&caller_view_key)?;
    let execute = vm.execute(
        &caller_private_key,
        ("test_upgrade_call_0.aleo", "run"),
        vec![Value::Record(record.clone())].into_iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 1);
    vm.add_next_block(&block)?;

    // Attempt to execute the `upgrade` function directly before `ConsensusVersion::V8`.
    let execute = vm.execute(
        &caller_private_key,
        ("credits.aleo", "upgrade"),
        vec![Value::Record(record.clone())].into_iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 1);
    vm.add_next_block(&block)?;

    // Check that the consensus version is `V8`.
    let block_height = vm.store.block_store().current_block_height();
    let consensus_version = CurrentNetwork::CONSENSUS_VERSION(block_height)?;
    assert_eq!(consensus_version, ConsensusVersion::V8);

    // Attempt to execute the program after `ConsensusVersion::V8`.
    let execute = vm.execute(
        &caller_private_key,
        ("test_upgrade_call_0.aleo", "run"),
        vec![Value::Record(record.clone())].into_iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 1);

    // Attempt to deploy the program after `ConsensusVersion::V8`.
    let deployment = vm.deploy(&caller_private_key, &sample_program(1)?, None, 0, None, rng)?;
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng)?;
    assert_eq!(block.transactions().num_accepted(), 0);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 1);
    vm.add_next_block(&block)?;

    // Execute the `upgrade` function directly.
    let execute = vm.execute(
        &caller_private_key,
        ("credits.aleo", "upgrade"),
        vec![Value::Record(record)].into_iter(),
        None,
        0,
        None,
        rng,
    )?;
    let block = sample_next_block(&vm, &caller_private_key, &[execute], rng)?;
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);

    Ok(())
}
