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

use circuit::Circuit;
use console::{algorithms::U8, network::ConsensusVersion};
use snarkvm_synthesizer_program::Program;
use snarkvm_synthesizer_snark::UniversalSRS;
use snarkvm_utilities::TestRng;

use std::sync::OnceLock;

// This test verifies that:
// - programs using syntax introduced in `V13` cannot be deployed before `V13`.
// - programs using syntax introduced in `V13` can be deployed at and after `V13`.
// - a program with an array larger than 2048 cannot be deployed after `V13`.
#[test]
fn test_deployments_for_v13_features() {
    // Define the programs.
    let programs = vec![
        // A program with an array larger than 512 elements.
        r"
program uses_large_arrays.aleo;

mapping data:
    key as [u8; 513u32].public;
    value as u32.public;

function dummy:

constructor:
    assert.eq true true;
",
        // A program that uses the `snark.verify` opcode.
        r"
program uses_snark_verify.aleo;

function dummy:
    input r0 as  [u8; 8u32].public;
    input r1 as [field; 8u32].public;
    input r2 as [u8; 8u32].public;
    async dummy r0 r1 r2 into r3;
    output r3 as uses_snark_verify.aleo/dummy.future;

finalize dummy:
    input r0 as  [u8; 8u32].public;
    input r1 as [field; 8u32].public;
    input r2 as [u8; 8u32].public;
    snark.verify r0 r1 r2 into r3;
    assert.eq r3 true;

constructor:
    assert.eq true true;
",
    ];

    // Initialize an RNG.
    let rng = &mut TestRng::default();

    // Initialize a new caller.
    let caller_private_key = crate::vm::test_helpers::sample_genesis_private_key(rng);

    // Initialize the VM at one less than the V13 height.
    let v13_height = CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V13).unwrap();
    let num_programs = u32::try_from(programs.len()).unwrap();
    let vm = crate::vm::test_helpers::sample_vm_at_height(v13_height - num_programs, rng);

    // Deploy each program and ensure it fails.
    for program in &programs {
        let program = Program::from_str(program).unwrap();
        let deployment = vm.deploy(&caller_private_key, &program, None, 0, None, rng).unwrap();
        let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng).unwrap();
        assert_eq!(block.transactions().num_accepted(), 0);
        assert_eq!(block.transactions().num_rejected(), 0);
        assert_eq!(block.aborted_transaction_ids().len(), 1);
        vm.add_next_block(&block).unwrap();
    }

    // Verify that we are at the expected height.
    assert_eq!(vm.block_store().current_block_height(), v13_height);

    // Deploy each program and ensure it succeeds.
    for program in &programs {
        let program = Program::from_str(program).unwrap();
        let deployment = vm.deploy(&caller_private_key, &program, None, 0, None, rng).unwrap();
        let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng).unwrap();
        assert_eq!(block.transactions().num_accepted(), 1);
        assert_eq!(block.transactions().num_rejected(), 0);
        assert_eq!(block.aborted_transaction_ids().len(), 0);
        vm.add_next_block(&block).unwrap();
    }
}

#[test]
fn test_snark_verify() {
    // Define the verification program.
    let program = Program::from_str(
        r"
program test_snark_verify.aleo;

function verify_proof:
    input r0 as [u8; 673u32].private;
    input r1 as [field; 2u32].private;
    input r2 as [u8; 957u32].private;
    async verify_proof r0 r1 r2 into r3;
    output r3 as test_snark_verify.aleo/verify_proof.future;
finalize verify_proof:
    input r0 as [u8; 673u32].public;
    input r1 as [field; 2u32].public;
    input r2 as [u8; 957u32].public;
    snark.verify r0 r1 r2 into r3;
    assert.eq r3 true;

constructor:
    assert.eq true true;
    ",
    )
    .unwrap();

    // Initialize an RNG.
    let rng = &mut TestRng::default();

    // Initialize a new caller.
    let caller_private_key = crate::vm::test_helpers::sample_genesis_private_key(rng);

    // Initialize the VM at the V13 height.
    let v13_height = CurrentNetwork::CONSENSUS_HEIGHT(ConsensusVersion::V13).unwrap();
    let vm = crate::vm::test_helpers::sample_vm_at_height(v13_height, rng);

    // Deploy the program.
    let deployment = vm.deploy(&caller_private_key, &program, None, 0, None, rng).unwrap();
    let block = sample_next_block(&vm, &caller_private_key, &[deployment], rng).unwrap();
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 0);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block).unwrap();

    // Sample a Varuna assignment
    let assignment = snarkvm_synthesizer_snark::test_helpers::sample_assignment();

    // Varuna setup, prove, and verify.
    let srs = UniversalSRS::<CurrentNetwork>::load().unwrap();
    let (proving_key, verifying_key) = srs.to_circuit_key("test", &assignment).unwrap();
    let verifying_key_bytes = verifying_key.to_bytes_le().unwrap();

    // Construct the proof
    let varuna_version = VarunaVersion::V2;
    let proof = proving_key.prove("test", varuna_version, &assignment, &mut TestRng::default()).unwrap();
    let proof_bytes = proof.to_bytes_le().unwrap();

    println!("verifying_key_size: {}", verifying_key_bytes.len());
    println!("proof size: {}", proof_bytes.len());

    let one = <Circuit as circuit::Environment>::BaseField::one();
    let zero = <Circuit as circuit::Environment>::BaseField::zero();
    let public_inputs = vec![one, one];
    let invalid_public_inputs = vec![one, zero];
    assert!(verifying_key.verify("test", varuna_version, &public_inputs, &proof));
    assert!(!verifying_key.verify("test", varuna_version, &invalid_public_inputs, &proof));

    // Construct the inputs for the execution.
    let verifying_key_input = Value::Plaintext(Plaintext::Array(
        verifying_key_bytes
            .into_iter()
            .map(|byte| Plaintext::from(Literal::<CurrentNetwork>::U8(U8::new(byte))))
            .collect(),
        OnceLock::new(),
    ));
    let verification_inputs = Value::Plaintext(Plaintext::Array(
        public_inputs
            .into_iter()
            .map(|field| Plaintext::from(Literal::<CurrentNetwork>::Field(Field::new(field))))
            .collect(),
        OnceLock::new(),
    ));
    let invalid_verification_inputs = Value::Plaintext(Plaintext::Array(
        invalid_public_inputs
            .into_iter()
            .map(|field| Plaintext::from(Literal::<CurrentNetwork>::Field(Field::new(field))))
            .collect(),
        OnceLock::new(),
    ));
    let proof_input = Value::Plaintext(Plaintext::Array(
        proof_bytes.into_iter().map(|byte| Plaintext::from(Literal::<CurrentNetwork>::U8(U8::new(byte)))).collect(),
        OnceLock::new(),
    ));

    // Execute a transaction that verifies the proof correctly.
    let valid_execution = {
        let inputs = vec![verifying_key_input.clone(), verification_inputs.clone(), proof_input.clone()];

        vm.execute(
            &caller_private_key,
            ("test_snark_verify.aleo", "verify_proof"),
            inputs.into_iter(),
            None,
            0,
            None,
            rng,
        )
        .unwrap()
    };
    let valid_execution_id = valid_execution.id();

    // Execute a transaction that fails to verify the proof.
    let invalid_execution = {
        let inputs = vec![verifying_key_input, invalid_verification_inputs, proof_input];

        vm.execute(
            &caller_private_key,
            ("test_snark_verify.aleo", "verify_proof"),
            inputs.into_iter(),
            None,
            0,
            None,
            rng,
        )
        .unwrap()
    };
    let invalid_execution_id = invalid_execution.id();

    let block = sample_next_block(&vm, &caller_private_key, &[valid_execution, invalid_execution], rng).unwrap();
    assert_eq!(block.transactions().num_accepted(), 1);
    assert_eq!(block.transactions().num_rejected(), 1);
    assert_eq!(block.aborted_transaction_ids().len(), 0);
    vm.add_next_block(&block).unwrap();

    assert!(vm.transaction_store().contains_transaction_id(&valid_execution_id).unwrap());
    assert!(vm.block_store().contains_rejected_or_aborted_transaction_id(&invalid_execution_id).unwrap());
}
