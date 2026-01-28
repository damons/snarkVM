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

impl<E: Environment> FromBits for IdentifierLiteral<E> {
    type Boolean = Boolean<E>;

    /// Creates an identifier literal from a list of little-endian bits.
    ///
    /// - If more than 248 bits are provided, upper bits are asserted to be zero.
    /// - If fewer than 248 bits are provided, the input is zero-padded.
    /// - Validates the identifier format (character set, first char is letter, trailing nulls).
    fn from_bits_le(bits_le: &[Self::Boolean]) -> Self {
        let size_in_bits = Self::size_in_bits();
        let max_bytes = console::IdentifierLiteral::<E::Network>::MAX_BYTES;

        // If more bits than needed, assert upper bits are zero.
        if bits_le.len() > size_in_bits {
            Boolean::assert_bits_are_zero(&bits_le[size_in_bits..]);
        }

        // Extract/pad to exactly 248 bits.
        let mut padded = bits_le.iter().take(size_in_bits).cloned().collect::<Vec<_>>();
        padded.resize(size_in_bits, Boolean::constant(false));

        // Convert bits to bytes.
        let mut bytes_vec = Vec::with_capacity(max_bytes);
        for i in 0..max_bytes {
            let chunk = &padded[i * 8..(i + 1) * 8];
            bytes_vec.push(U8::from_bits_le(chunk));
        }
        // Safety: max_bytes is always 31, matching the array size.
        let bytes: [U8<E>; 31] = bytes_vec.try_into().unwrap_or_else(|_| E::halt("Failed to convert to byte array"));

        // Determine mode from the first bit.
        let mode = bits_le.first().map(|b| b.eject_mode()).unwrap_or(Mode::Constant);

        // Validate the identifier.
        if mode.is_constant() {
            // For constants, validate via the console type.
            let mut raw_bytes = [0u8; 31];
            for (i, byte) in bytes.iter().enumerate() {
                raw_bytes[i] = *byte.eject_value();
            }
            console::IdentifierLiteral::<E::Network>::from_bytes_array(raw_bytes).expect("Invalid identifier literal");
        } else {
            // For non-constants, validate via circuit constraints.
            validate_identifier_bytes::<E>(&bytes);
        }

        Self { bytes }
    }

    /// Creates an identifier literal from a list of big-endian bits.
    fn from_bits_be(bits_be: &[Self::Boolean]) -> Self {
        // Reverse the bits to get little-endian order.
        let bits_le: Vec<_> = bits_be.iter().rev().cloned().collect();
        Self::from_bits_le(&bits_le)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;

    type CurrentEnvironment = Circuit;

    /// Test strings covering various identifier patterns.
    const TEST_STRINGS: &[&str] = &["a", "hello", "hello_world", "Test123", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcde"];

    fn check_from_bits_le(
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

            // Get the bits from a circuit representation.
            let injected = IdentifierLiteral::<CurrentEnvironment>::new(mode, expected);
            let bits = injected.to_bits_le();

            Circuit::scope(format!("from_bits_le {mode}"), || {
                // Reconstruct from bits.
                let candidate = IdentifierLiteral::<CurrentEnvironment>::from_bits_le(&bits);

                // Verify the value matches.
                assert_eq!(expected, candidate.eject_value());
                assert_eq!(mode, candidate.eject_mode());

                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });

            Circuit::reset();
        }
        Ok(())
    }

    fn check_from_bits_be(
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

            // Get the bits from a circuit representation.
            let injected = IdentifierLiteral::<CurrentEnvironment>::new(mode, expected);
            let bits = injected.to_bits_be();

            Circuit::scope(format!("from_bits_be {mode}"), || {
                // Reconstruct from bits.
                let candidate = IdentifierLiteral::<CurrentEnvironment>::from_bits_be(&bits);

                // Verify the value matches.
                assert_eq!(expected, candidate.eject_value());
                assert_eq!(mode, candidate.eject_mode());

                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });

            Circuit::reset();
        }
        Ok(())
    }

    #[test]
    fn test_from_bits_le_constant() -> Result<()> {
        // Constants: no circuit constraints, validation is done via console.
        check_from_bits_le(Mode::Constant, 0, 0, 0, 0)
    }

    #[test]
    fn test_from_bits_le_public() -> Result<()> {
        // Non-constants: validation constraints.
        check_from_bits_le(Mode::Public, 0, 0, 810, 1027)
    }

    #[test]
    fn test_from_bits_le_private() -> Result<()> {
        // Non-constants: validation constraints.
        check_from_bits_le(Mode::Private, 0, 0, 810, 1027)
    }

    #[test]
    fn test_from_bits_be_constant() -> Result<()> {
        check_from_bits_be(Mode::Constant, 0, 0, 0, 0)
    }

    #[test]
    fn test_from_bits_be_public() -> Result<()> {
        check_from_bits_be(Mode::Public, 0, 0, 810, 1027)
    }

    #[test]
    fn test_from_bits_be_private() -> Result<()> {
        check_from_bits_be(Mode::Private, 0, 0, 810, 1027)
    }

    #[test]
    fn test_from_bits_le_with_excess_zero_bits() -> Result<()> {
        // Test that excess zero bits are accepted.
        let expected =
            console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::new("hello").unwrap();

        // Get the bits and add excess zeros.
        let injected = IdentifierLiteral::<CurrentEnvironment>::new(Mode::Private, expected);
        let mut bits = injected.to_bits_le();
        // Add 5 extra zero bits (simulate field bits > 248).
        for _ in 0..5 {
            bits.push(Boolean::new(Mode::Private, false));
        }

        Circuit::scope("from_bits_le with excess", || {
            let candidate = IdentifierLiteral::<CurrentEnvironment>::from_bits_le(&bits);
            assert_eq!(expected, candidate.eject_value());
            assert!(Circuit::is_satisfied());
        });

        Circuit::reset();
        Ok(())
    }

    #[test]
    fn test_from_bits_le_with_excess_one_bits_unsatisfied() {
        // Test that excess non-zero bits cause unsatisfied circuit.
        let expected =
            console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::new("hello").unwrap();

        // Get the bits and add excess one.
        let injected = IdentifierLiteral::<CurrentEnvironment>::new(Mode::Private, expected);
        let mut bits = injected.to_bits_le();
        // Add an extra one bit.
        bits.push(Boolean::new(Mode::Private, true));

        Circuit::scope("from_bits_le with excess one", || {
            let _candidate = IdentifierLiteral::<CurrentEnvironment>::from_bits_le(&bits);
        });

        // Circuit should be unsatisfied.
        assert!(!Circuit::is_satisfied());
        Circuit::reset();
    }
}
