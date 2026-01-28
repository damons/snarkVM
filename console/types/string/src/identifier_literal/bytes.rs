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

impl<E: Environment> FromBytes for IdentifierLiteral<E> {
    /// Reads the identifier literal from a buffer.
    /// Format: length-prefixed (num_bytes as u8, then content bytes).
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the number of bytes.
        let num_bytes = u8::read_le(&mut reader)? as usize;
        // Validate the length.
        if num_bytes == 0 || num_bytes > Self::SIZE_IN_BYTES {
            return Err(error("Invalid identifier literal length"));
        }
        // Read the content bytes.
        let mut content = vec![0u8; num_bytes];
        reader.read_exact(&mut content)?;
        // Pad into a 31-byte array.
        let mut bytes = [0u8; 31];
        bytes[..num_bytes].copy_from_slice(&content);
        // Validate the identifier bytes.
        validate_identifier_bytes(&bytes).map_err(|e| error(e.to_string()))?;
        // Return the identifier literal.
        Ok(Self { bytes, _phantom: core::marker::PhantomData })
    }
}

impl<E: Environment> ToBytes for IdentifierLiteral<E> {
    /// Writes the identifier literal to a buffer.
    /// Format: length-prefixed (num_bytes as u8, then content bytes).
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Write the length.
        let length = self.length();
        length.write_le(&mut writer)?;
        // Write only the content bytes.
        writer.write_all(&self.bytes[..length as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network_environment::Console;

    type CurrentEnvironment = Console;

    #[test]
    fn test_bytes_roundtrip() -> Result<()> {
        let expected = IdentifierLiteral::<CurrentEnvironment>::new("hello_world")?;
        let expected_bytes = expected.to_bytes_le()?;
        let recovered = IdentifierLiteral::read_le(&expected_bytes[..])?;
        assert_eq!(expected, recovered);
        Ok(())
    }

    #[test]
    fn test_bytes_single_char() -> Result<()> {
        let expected = IdentifierLiteral::<CurrentEnvironment>::new("a")?;
        let expected_bytes = expected.to_bytes_le()?;
        let recovered = IdentifierLiteral::read_le(&expected_bytes[..])?;
        assert_eq!(expected, recovered);
        Ok(())
    }
}
