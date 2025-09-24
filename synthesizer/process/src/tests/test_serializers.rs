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

use crate::{CallStack, Process, Trace};
use circuit::network::AleoV0;
use console::{
    account::PrivateKey,
    network::{MainnetV0, prelude::*},
    program::{ArrayType, Identifier, LiteralType, PlaintextType, U32, Value},
};
use snarkvm_synthesizer_program::Program;

#[cfg(feature = "locktick")]
use locktick::parking_lot::RwLock;
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;
use rayon::prelude::*;
use std::sync::Arc;

type CurrentNetwork = MainnetV0;
type CurrentAleo = AleoV0;

#[test]
fn test_serialize_deserialize_equivalence() {
    // Number of iterations to run for each type.
    const ITERATIONS: usize = 50;

    // A helper function to define the program.
    fn construct_program(
        variant: &str,
        src_type: PlaintextType<CurrentNetwork>,
        bits_type: ArrayType<CurrentNetwork>,
    ) -> Program<CurrentNetwork> {
        Program::<CurrentNetwork>::from_str(&format!(
            r"
program test.aleo;

function test_serde_equivalence:
    input r0 as {src_type}.private;
    serialize.{variant} r0 ({src_type}) into r1 ({bits_type});
    deserialize.{variant} r1 ({bits_type}) into r2 ({src_type});
    assert.eq r0 r2;
    "
        ))
        .unwrap()
    }

    // A helper function defining the types to be tested.
    fn test_types(is_raw: bool) -> Vec<PlaintextType<CurrentNetwork>> {
        let mut types = vec![
            PlaintextType::Literal(LiteralType::Address),
            PlaintextType::Literal(LiteralType::Boolean),
            PlaintextType::Literal(LiteralType::Field),
            PlaintextType::Literal(LiteralType::Group),
            PlaintextType::Literal(LiteralType::I8),
            PlaintextType::Literal(LiteralType::I16),
            PlaintextType::Literal(LiteralType::I32),
            PlaintextType::Literal(LiteralType::I64),
            PlaintextType::Literal(LiteralType::I128),
            PlaintextType::Literal(LiteralType::U8),
            PlaintextType::Literal(LiteralType::U16),
            PlaintextType::Literal(LiteralType::U32),
            PlaintextType::Literal(LiteralType::U64),
            PlaintextType::Literal(LiteralType::U128),
            PlaintextType::Literal(LiteralType::Scalar),
            PlaintextType::Array(ArrayType::new(PlaintextType::Literal(LiteralType::U8), vec![U32::new(8)]).unwrap()),
        ];

        // Add additional types for the raw variant.
        if is_raw {
            types.push(PlaintextType::Array(
                ArrayType::new(PlaintextType::Literal(LiteralType::U8), vec![U32::new(32)]).unwrap(),
            ));
            types.push(PlaintextType::Array(
                ArrayType::new(PlaintextType::Literal(LiteralType::U8), vec![U32::new(64)]).unwrap(),
            ))
        }

        types
    }

    // A helper function to run tests for a given variant (either raw or not).
    fn run_test(type_: PlaintextType<CurrentNetwork>, is_raw: bool, iterations: usize) {
        // Initailize an RNG.
        let rng = &mut TestRng::default();

        // Load the process.
        let mut process = Process::<CurrentNetwork>::load().unwrap();

        // Structs are not supported.
        let fail_get_struct = |_: &Identifier<CurrentNetwork>| bail!("structs are not supported");

        // Get the bits type.
        let num_bits = match is_raw {
            true => type_.plaintext_size_in_raw_bits(&fail_get_struct).unwrap(),
            false => type_.plaintext_size_in_bits(&fail_get_struct).unwrap(),
        };
        let num_bits = u32::try_from(num_bits).unwrap();
        let bits_type = ArrayType::new(PlaintextType::Literal(LiteralType::Boolean), vec![U32::new(num_bits)]).unwrap();

        // Sample the program.
        let variant = if is_raw { "bits.raw" } else { "bits" };
        let program = construct_program(variant, type_.clone(), bits_type.clone());

        // Add the program to the process.
        process.add_program(&program).unwrap();

        // Get the stack.
        let stack = process.get_stack(program.id()).unwrap();

        // Sample a private key.
        let private_key = PrivateKey::new(rng).unwrap();

        // Get the function name.
        let function_name = Identifier::from_str("test_serde_equivalence").unwrap();

        // Run the test for a desired number of iterations.
        for _ in 0..iterations {
            // Sample the plaintext.
            let plaintext = stack.sample_plaintext(&type_, rng).unwrap();

            // Get the bits of the plaintext.
            let bits = match is_raw {
                false => plaintext.to_bits_le(),
                true => plaintext.to_bits_raw_le(),
            };

            // Check that the number of bits matches.
            assert_eq!(bits.len(), num_bits as usize, "The number of bits does not match the expected size");

            // Construct the inputs.
            let inputs = vec![Value::Plaintext(plaintext.clone())];

            // Generate an authorixation.
            let authorization =
                stack.authorize::<CurrentAleo, _>(&private_key, function_name, inputs.iter(), rng).unwrap();

            // Evaluate the function.
            let res_eval = stack.evaluate_function::<CurrentAleo, _>(
                CallStack::evaluate(authorization.replicate()).unwrap(),
                None,
                None,
                rng,
            );
            let eval_is_ok = res_eval.is_ok();

            // Execute the function.
            let trace = Trace::new();
            let res_exec = stack.execute_function::<CurrentAleo, _>(
                CallStack::execute(authorization.replicate(), Arc::new(RwLock::new(trace))).unwrap(),
                None,
                None,
                rng,
            );
            let exec_is_ok = res_exec.is_ok() || <CurrentAleo as circuit::Environment>::is_satisfied();

            // Check that either all operations succeeded.
            assert!(
                eval_is_ok && exec_is_ok,
                "The results of the evaluation and execution should either all succeed or all fail"
            );
            // Reset the circuit.
            <CurrentAleo as circuit::Environment>::reset();
        }
    }

    // Run the tests for both variants.
    for is_raw in [false, true] {
        test_types(is_raw).into_par_iter().for_each(|type_| {
            run_test(type_, is_raw, ITERATIONS);
        })
    }
}
