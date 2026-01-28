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

    /// Creates an identifier literal from a field element.
    fn from_field(field: &Self::Field) -> Result<Self> {
        Self::from_bits_le(&field.to_bits_le())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network_environment::Console;

    type CurrentEnvironment = Console;

    const ITERATIONS: u64 = 100;

    #[test]
    fn test_from_field_round_trip() -> Result<()> {
        let mut rng = TestRng::default();

        for _ in 0..ITERATIONS {
            // Sample a random identifier literal.
            let expected = IdentifierLiteral::<CurrentEnvironment>::rand(&mut rng);

            // Convert to field and back.
            let field = expected.to_field()?;
            let candidate = IdentifierLiteral::<CurrentEnvironment>::from_field(&field)?;

            // Verify round-trip.
            assert_eq!(expected, candidate);
        }
        Ok(())
    }

    #[test]
    fn test_from_field_valid() {
        // Create a known valid identifier.
        let original = IdentifierLiteral::<CurrentEnvironment>::new("hello_world").unwrap();
        let field = original.to_field().unwrap();
        let recovered = IdentifierLiteral::<CurrentEnvironment>::from_field(&field).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_from_field_invalid_ascii() {
        // Create a field with non-ASCII high bit set in the first byte.
        let field = Field::<CurrentEnvironment>::from_bits_le(&[true; 8]).unwrap();
        assert!(IdentifierLiteral::<CurrentEnvironment>::from_field(&field).is_err());
    }

    #[test]
    fn test_from_field_rejects_non_trailing_null() {
        // Construct a field with bytes [0x61, 0x00, 0x62, 0x00, ...] — non-null after null.
        let mut bad_bytes = vec![0u8; 32];
        bad_bytes[0] = b'a';
        bad_bytes[2] = b'b';
        let field = Field::<CurrentEnvironment>::from_bits_le(
            &bad_bytes.to_bits_le()[..Field::<CurrentEnvironment>::size_in_bits()],
        )
        .expect("Failed to construct field from bits");
        assert!(IdentifierLiteral::<CurrentEnvironment>::from_field(&field).is_err());
    }

    #[test]
    fn test_from_field_rejects_digit_start() {
        // Create a field with first byte = '1' (digit).
        let mut bytes = [0u8; 31];
        bytes[0] = b'1';
        // Pack as field.
        let field = Field::<CurrentEnvironment>::from_bits_le(&bytes.to_bits_le()).unwrap();
        assert!(IdentifierLiteral::<CurrentEnvironment>::from_field(&field).is_err());
    }

    #[test]
    fn test_from_field_rejects_underscore_start() {
        // Create a field with first byte = '_' (underscore).
        let mut bytes = [0u8; 31];
        bytes[0] = b'_';
        // Pack as field.
        let field = Field::<CurrentEnvironment>::from_bits_le(&bytes.to_bits_le()).unwrap();
        assert!(IdentifierLiteral::<CurrentEnvironment>::from_field(&field).is_err());
    }
}
