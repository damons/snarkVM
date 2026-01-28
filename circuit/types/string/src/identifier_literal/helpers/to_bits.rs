// Copyright (c) 2019-2026 Provable Inc.
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

impl<E: Environment> ToBits for IdentifierLiteral<E> {
    type Boolean = Boolean<E>;

    /// Returns the little-endian bits of the identifier literal (248 bits from 31 bytes).
    fn write_bits_le(&self, vec: &mut Vec<Boolean<E>>) {
        // Write all 31 bytes (248 bits) in little-endian order.
        for byte in self.bytes.iter() {
            byte.write_bits_le(vec);
        }
    }

    /// Returns the big-endian bits of the identifier literal (248 bits from 31 bytes).
    fn write_bits_be(&self, vec: &mut Vec<Boolean<E>>) {
        // Write in LE order, then reverse the appended bits.
        let initial_len = vec.len();
        self.write_bits_le(vec);
        vec[initial_len..].reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;

    type CurrentEnvironment = Circuit;

    /// Test strings covering various identifier patterns.
    const TEST_STRINGS: &[&str] = &["a", "hello", "hello_world", "Test123", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcde"];

    fn check_to_bits_le(
        mode: Mode,
        num_constants: u64,
        num_public: u64,
        num_private: u64,
        num_constraints: u64,
    ) -> Result<()> {
        for string in TEST_STRINGS {
            // Construct a console identifier literal.
            let expected =
                console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::new(string).unwrap();

            Circuit::scope(format!("to_bits_le {mode}"), || {
                // Inject into the circuit.
                let candidate = IdentifierLiteral::<CurrentEnvironment>::new(mode, expected);

                // Get circuit bits and eject their values.
                let candidate_bits = candidate.to_bits_le();

                // Verify 248 bits (31 bytes).
                assert_eq!(candidate_bits.len(), 248);

                // Verify each ejected bit matches the expected byte-level bit decomposition.
                let expected_bits = expected.to_bits_le();
                for (expected_bit, candidate_bit) in expected_bits.iter().zip(candidate_bits.iter()) {
                    assert_eq!(*expected_bit, candidate_bit.eject_value());
                }

                // ToBits incurs no additional constraints (bits already exist in bytes).
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });

            Circuit::reset();
        }
        Ok(())
    }

    fn check_to_bits_be(
        mode: Mode,
        num_constants: u64,
        num_public: u64,
        num_private: u64,
        num_constraints: u64,
    ) -> Result<()> {
        for string in TEST_STRINGS {
            // Construct a console identifier literal.
            let expected =
                console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::new(string).unwrap();

            Circuit::scope(format!("to_bits_be {mode}"), || {
                // Inject into the circuit.
                let candidate = IdentifierLiteral::<CurrentEnvironment>::new(mode, expected);

                // Get circuit bits and eject their values.
                let candidate_bits = candidate.to_bits_be();

                // Verify 248 bits (31 bytes).
                assert_eq!(candidate_bits.len(), 248);

                // Verify each ejected bit matches the expected byte-level bit decomposition.
                let expected_bits = expected.to_bits_be();
                for (expected_bit, candidate_bit) in expected_bits.iter().zip(candidate_bits.iter()) {
                    assert_eq!(*expected_bit, candidate_bit.eject_value());
                }

                // ToBits incurs no additional constraints (bits already exist in bytes).
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });

            Circuit::reset();
        }
        Ok(())
    }

    // Note: Injection costs vary by mode. ToBits itself adds no constraints.
    // Constant: 248 constants (one per bit), no public/private/constraints.
    // Public: 248 public inputs, validation constraints from Inject.
    // Private: 248 private witnesses + validation constraints from Inject.

    #[test]
    fn test_to_bits_le_constant() -> Result<()> {
        // ToBits on already-injected constant: no additional scope cost.
        check_to_bits_le(Mode::Constant, 248, 0, 0, 0)
    }

    #[test]
    fn test_to_bits_le_public() -> Result<()> {
        // Scope includes Inject costs (validation) + ToBits (free).
        check_to_bits_le(Mode::Public, 0, 248, 810, 1275)
    }

    #[test]
    fn test_to_bits_le_private() -> Result<()> {
        // Scope includes Inject costs (validation) + ToBits (free).
        check_to_bits_le(Mode::Private, 0, 0, 1058, 1275)
    }

    #[test]
    fn test_to_bits_be_constant() -> Result<()> {
        check_to_bits_be(Mode::Constant, 248, 0, 0, 0)
    }

    #[test]
    fn test_to_bits_be_public() -> Result<()> {
        check_to_bits_be(Mode::Public, 0, 248, 810, 1275)
    }

    #[test]
    fn test_to_bits_be_private() -> Result<()> {
        check_to_bits_be(Mode::Private, 0, 0, 1058, 1275)
    }
}
