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

include!("../helpers/macros.rs");

use console::{
    algorithms::{ECDSASignature, Keccak256, Keccak384, Keccak512, Sha3_256, Sha3_384, Sha3_512},
    network::MainnetV0,
    prelude::*,
    program::{ArrayType, Identifier, Literal, LiteralType, Plaintext, PlaintextType, Register, Value},
    types::{Boolean, U8, U32},
};
use snarkvm_synthesizer_process::{FinalizeRegisters, Process, Stack};
use snarkvm_synthesizer_program::{
    ECDSAVerify,
    ECDSAVerifyKeccak256,
    ECDSAVerifyKeccak256Eth,
    ECDSAVerifyKeccak256Raw,
    ECDSAVerifyKeccak384,
    ECDSAVerifyKeccak384Eth,
    ECDSAVerifyKeccak384Raw,
    ECDSAVerifyKeccak512,
    ECDSAVerifyKeccak512Eth,
    ECDSAVerifyKeccak512Raw,
    ECDSAVerifySha3_256,
    ECDSAVerifySha3_256Eth,
    ECDSAVerifySha3_256Raw,
    ECDSAVerifySha3_384,
    ECDSAVerifySha3_384Eth,
    ECDSAVerifySha3_384Raw,
    ECDSAVerifySha3_512,
    ECDSAVerifySha3_512Eth,
    ECDSAVerifySha3_512Raw,
    FinalizeGlobalState,
    Opcode,
    Operand,
    Program,
    RegistersTrait as _,
};

use k256::ecdsa::{SigningKey, VerifyingKey};

type CurrentNetwork = MainnetV0;

const ITERATIONS: usize = 25;

/// Samples the stack. Note: Do not replicate this for real program use, it is insecure.
#[allow(clippy::type_complexity)]
fn sample_stack(
    opcode: Opcode,
    type_0: PlaintextType<CurrentNetwork>,
    type_1: PlaintextType<CurrentNetwork>,
    type_2: LiteralType,
    mode: circuit::Mode,
) -> Result<(Stack<CurrentNetwork>, Vec<Operand<CurrentNetwork>>, Register<CurrentNetwork>)> {
    // Initialize the opcode.
    let opcode = opcode.to_string();

    // Initialize the function name.
    let function_name = Identifier::<CurrentNetwork>::from_str("run")?;

    // Initialize the registers.
    let r0 = Register::Locator(0);
    let r1 = Register::Locator(1);
    let r2 = Register::Locator(2);
    let r3 = Register::Locator(3);

    // Initialize the program.
    let program = Program::from_str(&format!(
        "program testing.aleo;
            function {function_name}:
                input {r0} as {type_0}.{mode};
                input {r1} as {type_1}.{mode};
                input {r2} as {type_2}.{mode};
                async {function_name} {r0} {r1} {r2} into r3;
                output r3 as testing.aleo/{function_name}.future;
            finalize {function_name}:
                input {r0} as {type_0}.public;
                input {r1} as {type_1}.public;
                input {r2} as {type_2}.public;
                {opcode} {r0} {r1} {r2} into {r3};
        "
    ))?;

    // Initialize the operands.
    let operands = vec![Operand::Register(r0), Operand::Register(r1), Operand::Register(r2)];

    // Initialize the stack.
    let stack = Stack::new(&Process::load()?, &program)?;

    Ok((stack, operands, r3))
}

/// Samples the finalize registers. Note: Do not replicate this for real program use, it is insecure.
pub fn sample_ecdsa_finalize_registers(
    stack: &Stack<CurrentNetwork>,
    function_name: &Identifier<CurrentNetwork>,
    signature: &[u8],
    pub_key: &[u8],
    expected_length: usize,
    message: &Literal<CurrentNetwork>,
) -> Result<FinalizeRegisters<CurrentNetwork>> {
    // Initialize the registers.
    let mut finalize_registers = FinalizeRegisters::<CurrentNetwork>::new(
        FinalizeGlobalState::from(1, 1, [0; 32]),
        <CurrentNetwork as Network>::TransitionID::default(),
        *function_name,
        stack.get_finalize_types(function_name)?.clone(),
        0u64,
    );

    // Initialize the signature.
    let signature: [U8<CurrentNetwork>; 65] =
        signature.iter().copied().map(U8::<CurrentNetwork>::new).collect::<Vec<_>>().try_into().unwrap();

    // Initialize the public key.
    let pub_key_bytes = pub_key.iter().copied().map(U8::<CurrentNetwork>::new).collect::<Vec<_>>();
    let plaintext_pub_key = match expected_length {
        20 => {
            let pub_key: [U8<CurrentNetwork>; 20] = pub_key_bytes.try_into().unwrap();
            Plaintext::<CurrentNetwork>::from(pub_key)
        }
        ECDSASignature::VERIFYING_KEY_SIZE_IN_BYTES => {
            let pub_key: [U8<CurrentNetwork>; ECDSASignature::VERIFYING_KEY_SIZE_IN_BYTES] =
                pub_key_bytes.try_into().unwrap();

            Plaintext::<CurrentNetwork>::from(pub_key)
        }
        invalid_length => bail!("Invalid public key length: {invalid_length}"),
    };

    // Initialize the registers
    let register_0 = Register::Locator(0);
    let register_1 = Register::Locator(1);
    let register_2 = Register::Locator(2);
    // Initialize the console value.
    let value_0 = Value::Plaintext(Plaintext::<CurrentNetwork>::from(signature));
    let value_1 = Value::Plaintext(plaintext_pub_key);
    let value_2 = Value::Plaintext(Plaintext::<CurrentNetwork>::from(message));
    // Store the value in the console registers.
    finalize_registers.store(stack, &register_0, value_0)?;
    finalize_registers.store(stack, &register_1, value_1)?;
    finalize_registers.store(stack, &register_2, value_2)?;

    Ok(finalize_registers)
}

fn check_ecdsa<const VARIANT: u8, H: Hash<Input = bool, Output = Vec<bool>>>(
    operation: impl FnOnce(Vec<Operand<CurrentNetwork>>, Register<CurrentNetwork>) -> ECDSAVerify<CurrentNetwork, VARIANT>,
    hasher: &H,
    opcode: Opcode,
    literal: &Literal<CurrentNetwork>,
    mode: &circuit::Mode,
    rng: &mut TestRng,
) {
    // Generate the ecdsa signing keys.
    let signing_key = SigningKey::random(rng);
    let verifying_key = VerifyingKey::from(&signing_key);

    let (expected_length, vk) = if matches!(VARIANT, 1 | 4 | 7 | 10 | 13 | 16 | 19) || opcode.ends_with("eth") {
        // Ethereum address variant expects a 20-byte array.
        (20, ECDSASignature::ethereum_address_from_public_key(&verifying_key).unwrap().to_vec())
    } else {
        // Non-Ethereum address variant expects a compressed verifying key.
        (ECDSASignature::VERIFYING_KEY_SIZE_IN_BYTES, verifying_key.to_encoded_point(true).as_bytes().to_vec())
    };

    println!("Checking '{opcode}' for '{literal}.{mode}'");

    // Initialize the types.
    let type_0 =
        PlaintextType::Array(ArrayType::new(PlaintextType::Literal(LiteralType::U8), vec![U32::new(65)]).unwrap());
    let type_1 = PlaintextType::Array(
        ArrayType::new(PlaintextType::Literal(LiteralType::U8), vec![U32::new(expected_length as u32)]).unwrap(),
    );
    let type_2 = literal.to_type();

    // Initialize the stack.
    let (stack, operands, destination) = sample_stack(opcode, type_0, type_1, type_2, *mode).unwrap();

    // Initialize the operation.
    let operation = operation(operands, destination.clone());
    // Initialize the function name.
    let function_name = Identifier::from_str("run").unwrap();
    // Initialize a destination operand.
    let destination_operand = Operand::Register(destination);

    // Construct the signature.
    let wrapped_literal = Plaintext::<CurrentNetwork>::from(literal.clone());
    let message_bits = match opcode.ends_with(".raw") || opcode.ends_with(".eth") {
        true => wrapped_literal.to_bits_raw_le(),
        false => wrapped_literal.to_bits_le(),
    };
    let signature = ECDSASignature::sign::<H>(&signing_key, hasher, &message_bits).unwrap();
    let signature_bytes = signature.to_bytes_le().unwrap();

    // Attempt to finalize the valid operand case.
    let mut finalize_registers =
        sample_ecdsa_finalize_registers(&stack, &function_name, &signature_bytes, &vk, expected_length, literal)
            .unwrap();
    let result_a = operation.finalize(&stack, &mut finalize_registers);
    // Enforce that the signature verifies successfully.
    assert!(result_a.is_ok(), "The finalization should succeed for a valid operand");
    let output = finalize_registers.load(&stack, &destination_operand).unwrap();
    assert_eq!(
        output,
        Value::Plaintext(Plaintext::from(Literal::Boolean(Boolean::new(true)))),
        "The output should be true for a valid operand"
    );

    // Create an invalid signature by using a different signature.
    let invalid_signature =
        ECDSASignature::sign::<H>(&signing_key, hasher, &[message_bits, vec![true]].concat()).unwrap();
    let invalid_signature_bytes = invalid_signature.to_bytes_le().unwrap();
    let mut finalize_registers = sample_ecdsa_finalize_registers(
        &stack,
        &function_name,
        &invalid_signature_bytes,
        &vk,
        expected_length,
        literal,
    )
    .unwrap();
    let result_b = operation.finalize(&stack, &mut finalize_registers);
    // Enforce that the signature verification fails.
    assert!(result_b.is_ok(), "The finalization should succeed for the operand");
    let output = finalize_registers.load(&stack, &destination_operand).unwrap();
    assert_eq!(
        output,
        Value::Plaintext(Plaintext::from(Literal::Boolean(Boolean::new(false)))),
        "The output should be false for an invalid message"
    );
}

macro_rules! test_ecdsa {
    ($name: tt, $hash:ident, $ecdsa:ident, $iterations:expr) => {
        paste::paste! {
            #[test]
            fn [<test _ $name _ is _ correct>]() {
                // Initialize the operation.
                let operation = |operands, destination| $ecdsa::<CurrentNetwork>::new(operands, destination).unwrap();
                // Initialize the opcode.
                let opcode = $ecdsa::<CurrentNetwork>::opcode();

                // Prepare the rng.
                let mut rng = TestRng::default();

                // Prepare the hasher.
                let hasher = $hash::default();

                // Prepare the test.
                let modes = [circuit::Mode::Public, circuit::Mode::Private];

                for _ in 0..$iterations {
                    let literals = sample_literals!(CurrentNetwork, &mut rng);
                    for literal in literals.iter() {
                        for mode in modes.iter() {
                            check_ecdsa(
                                operation,
                                &hasher,
                                opcode,
                                literal,
                                mode,
                                &mut rng,
                            );
                        }
                    }
                }
            }
        }
    };
}

test_ecdsa!(ecdsa_verify_keccak256, Keccak256, ECDSAVerifyKeccak256, ITERATIONS);
test_ecdsa!(ecdsa_verify_keccak256_raw, Keccak256, ECDSAVerifyKeccak256Raw, ITERATIONS);
test_ecdsa!(ecdsa_verify_keccak256_eth, Keccak256, ECDSAVerifyKeccak256Eth, ITERATIONS);

test_ecdsa!(ecdsa_verify_keccak384, Keccak384, ECDSAVerifyKeccak384, ITERATIONS);
test_ecdsa!(ecdsa_verify_keccak384_raw, Keccak384, ECDSAVerifyKeccak384Raw, ITERATIONS);
test_ecdsa!(ecdsa_verify_keccak384_eth, Keccak384, ECDSAVerifyKeccak384Eth, ITERATIONS);

test_ecdsa!(ecdsa_verify_keccak512, Keccak512, ECDSAVerifyKeccak512, ITERATIONS);
test_ecdsa!(ecdsa_verify_keccak512_raw, Keccak512, ECDSAVerifyKeccak512Raw, ITERATIONS);
test_ecdsa!(ecdsa_verify_keccak512_eth, Keccak512, ECDSAVerifyKeccak512Eth, ITERATIONS);

test_ecdsa!(ecdsa_verify_sha3_256, Sha3_256, ECDSAVerifySha3_256, ITERATIONS);
test_ecdsa!(ecdsa_verify_sha3_256_raw, Sha3_256, ECDSAVerifySha3_256Raw, ITERATIONS);
test_ecdsa!(ecdsa_verify_sha3_256_eth, Sha3_256, ECDSAVerifySha3_256Eth, ITERATIONS);

test_ecdsa!(ecdsa_verify_sha3_384, Sha3_384, ECDSAVerifySha3_384, ITERATIONS);
test_ecdsa!(ecdsa_verify_sha3_384_raw, Sha3_384, ECDSAVerifySha3_384Raw, ITERATIONS);
test_ecdsa!(ecdsa_verify_sha3_384_eth, Sha3_384, ECDSAVerifySha3_384Eth, ITERATIONS);

test_ecdsa!(ecdsa_verify_sha3_512, Sha3_512, ECDSAVerifySha3_512, ITERATIONS);
test_ecdsa!(ecdsa_verify_sha3_512_raw, Sha3_512, ECDSAVerifySha3_512Raw, ITERATIONS);
test_ecdsa!(ecdsa_verify_sha3_512_eth, Sha3_512, ECDSAVerifySha3_512Eth, ITERATIONS);
