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

impl<E: Environment> FromField for IdentifierLiteral<E> {
    type Field = Field<E>;

    /// Creates an identifier literal from a circuit field element.
    ///
    /// Delegates to `FromBits`, which handles validation and upper-bit checking.
    fn from_field(field: Field<E>) -> Self {
        // Convert field to bits and delegate to FromBits.
        // FromBits will assert that bits beyond 248 are zero and validate the identifier.
        Self::from_bits_le(&field.to_bits_le())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::ToField as _;
    use snarkvm_circuit_environment::Circuit;
    use snarkvm_utilities::{TestRng, Uniform};

    type CurrentEnvironment = Circuit;

    const ITERATIONS: usize = 10;

    fn check_from_field(
        mode: Mode,
        num_constants: u64,
        num_public: u64,
        num_private: u64,
        num_constraints: u64,
    ) -> Result<()> {
        let mut rng = TestRng::default();

        for _ in 0..ITERATIONS {
            // Construct a random console identifier literal.
            let expected = console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::rand(&mut rng);

            // Get the field representation.
            let expected_field = expected.to_field().unwrap();
            let field = Field::<CurrentEnvironment>::new(mode, expected_field);

            Circuit::scope(format!("from_field {mode}"), || {
                // Reconstruct from field.
                let candidate = IdentifierLiteral::<CurrentEnvironment>::from_field(field);

                // Verify the value matches.
                assert_eq!(expected, candidate.eject_value());

                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });

            Circuit::reset();
        }
        Ok(())
    }

    #[test]
    fn test_from_field_constant() -> Result<()> {
        // Constant: field to_bits_le produces constant bits, FromBits validates via console.
        check_from_field(Mode::Constant, 253, 0, 0, 0)
    }

    #[test]
    fn test_from_field_public() -> Result<()> {
        // Non-constant: field to_bits_le + FromBits validation constraints.
        check_from_field(Mode::Public, 0, 0, 1315, 1535)
    }

    #[test]
    fn test_from_field_private() -> Result<()> {
        // Non-constant: field to_bits_le + FromBits validation constraints.
        check_from_field(Mode::Private, 0, 0, 1315, 1535)
    }

    #[test]
    fn test_from_field_round_trip() -> Result<()> {
        let mut rng = TestRng::default();

        for _ in 0..ITERATIONS {
            // Construct a random console identifier literal.
            let expected = console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::rand(&mut rng);

            // Inject, convert to field, then back.
            let injected = IdentifierLiteral::<CurrentEnvironment>::new(Mode::Private, expected);
            let field = injected.to_field();
            let recovered = IdentifierLiteral::<CurrentEnvironment>::from_field(field);

            // Verify round-trip.
            assert_eq!(expected, recovered.eject_value());

            Circuit::reset();
        }
        Ok(())
    }

    #[test]
    fn test_from_field_invalid_rejects() {
        use console::FromBytes;

        // Construct a field with bytes [0x61, 0x00, 0x62, 0x00, ...] — non-null after null.
        let mut bad_bytes = vec![0u8; 32];
        bad_bytes[0] = b'a';
        bad_bytes[2] = b'b';
        let field_value = console::Field::<<CurrentEnvironment as Environment>::Network>::from_bytes_le(&bad_bytes)
            .expect("Failed to construct field from bytes");

        // Inject the field and validate in a scope.
        Circuit::scope("test_from_field_invalid", || {
            let field = Field::<CurrentEnvironment>::new(Mode::Private, field_value);
            let _candidate = IdentifierLiteral::<CurrentEnvironment>::from_field(field);
        });

        // The circuit must be unsatisfied due to the trailing-null violation.
        assert!(!Circuit::is_satisfied());
        Circuit::reset();
    }
}
