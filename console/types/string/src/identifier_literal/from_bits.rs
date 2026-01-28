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
    /// Initializes an identifier literal from a list of little-endian bits.
    fn from_bits_le(bits_le: &[bool]) -> Result<Self> {
        // Ensure there are enough bits for 31 bytes.
        ensure!(bits_le.len() >= 248, "Not enough bits for identifier literal");
        // Reconstruct the 31 bytes from the first 248 bits.
        let mut bytes = [0u8; 31];
        for i in 0..31 {
            let mut byte = 0u8;
            for j in 0..8 {
                if bits_le[i * 8 + j] {
                    byte |= 1 << j;
                }
            }
            bytes[i] = byte;
        }
        // Validate the identifier bytes.
        validate_identifier_bytes(&bytes)?;
        // Return the identifier literal.
        Ok(Self { bytes, _phantom: core::marker::PhantomData })
    }

    /// Initializes an identifier literal from a list of big-endian bits.
    fn from_bits_be(bits_be: &[bool]) -> Result<Self> {
        // Reverse the bits to get little-endian order.
        let bits_le: Vec<bool> = bits_be.iter().rev().copied().collect();
        Self::from_bits_le(&bits_le)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network_environment::Console;

    type CurrentEnvironment = Console;

    #[test]
    fn test_from_bits_le_roundtrip() -> Result<()> {
        let original = IdentifierLiteral::<CurrentEnvironment>::new("hello_world")?;
        let bits = original.to_bits_le();
        let recovered = IdentifierLiteral::<CurrentEnvironment>::from_bits_le(&bits)?;
        assert_eq!(original, recovered);
        Ok(())
    }

    #[test]
    fn test_from_bits_be_roundtrip() -> Result<()> {
        let original = IdentifierLiteral::<CurrentEnvironment>::new("hello_world")?;
        let bits = original.to_bits_be();
        let recovered = IdentifierLiteral::<CurrentEnvironment>::from_bits_be(&bits)?;
        assert_eq!(original, recovered);
        Ok(())
    }
}
