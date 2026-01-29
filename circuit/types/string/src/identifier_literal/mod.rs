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

mod equal;
mod helpers;

#[cfg(test)]
use snarkvm_circuit_environment::assert_scope;

use snarkvm_circuit_environment::prelude::*;
use snarkvm_circuit_types_boolean::Boolean;
use snarkvm_circuit_types_field::Field;
use snarkvm_circuit_types_integers::U8;

/// A circuit identifier literal storing an ASCII string (up to 31 bytes) as a byte array.
///
/// When injected in non-constant mode, the circuit validates that every byte
/// is a valid identifier character (`[a-zA-Z0-9_\0]`), that the first byte is
/// a letter, and that null bytes appear only as trailing padding.
#[derive(Clone)]
pub struct IdentifierLiteral<E: Environment> {
    /// The bytes of the identifier literal.
    bytes: [U8<E>; 31],
}

impl<E: Environment> IdentifierLiteral<E> {
    /// Returns the number of bits in an identifier literal (248 = 31 bytes * 8 bits).
    pub const fn size_in_bits() -> usize {
        console::IdentifierLiteral::<E::Network>::SIZE_IN_BITS
    }
}

impl<E: Environment> Inject for IdentifierLiteral<E> {
    type Primitive = console::IdentifierLiteral<E::Network>;

    /// Initializes a new circuit identifier literal from a primitive.
    fn new(mode: Mode, value: Self::Primitive) -> Self {
        // Access the raw bytes from the console identifier literal.
        let raw_bytes = value.bytes();
        // Inject each byte into the circuit.
        let mut bytes_vec = Vec::with_capacity(raw_bytes.len());
        for &byte in raw_bytes.iter() {
            bytes_vec.push(U8::new(mode, console::Integer::new(byte)));
        }
        // Convert the Vec to a fixed-size array.
        // Safety: max_bytes is always 31, matching the array size.
        let bytes: [U8<E>; 31] = bytes_vec.try_into().unwrap_or_else(|_| E::halt("Failed to convert to byte array"));

        // Validate the character set in the circuit.
        validate_identifier_bytes::<E>(&bytes);

        Self { bytes }
    }
}

impl<E: Environment> Eject for IdentifierLiteral<E> {
    type Primitive = console::IdentifierLiteral<E::Network>;

    /// Ejects the mode of the identifier literal.
    fn eject_mode(&self) -> Mode {
        self.bytes[0].eject_mode()
    }

    /// Ejects the identifier literal as a primitive.
    fn eject_value(&self) -> Self::Primitive {
        // Eject each byte and collect into a 31-byte array.
        let mut raw_bytes = [0u8; 31];
        for (i, byte) in self.bytes.iter().enumerate() {
            raw_bytes[i] = *byte.eject_value();
        }
        // Recover the identifier literal from the byte array.
        console::IdentifierLiteral::from_bytes_array(raw_bytes).expect("Failed to eject identifier literal")
    }
}

impl<E: Environment> Parser for IdentifierLiteral<E> {
    /// Parses a string into an identifier literal circuit.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        // Parse the content from the string.
        let (string, content) = console::IdentifierLiteral::parse(string)?;
        // Parse the mode from the string.
        let (string, mode) = opt(pair(tag("."), Mode::parse))(string)?;

        match mode {
            Some((_, mode)) => Ok((string, IdentifierLiteral::new(mode, content))),
            None => Ok((string, IdentifierLiteral::new(Mode::Constant, content))),
        }
    }
}

impl<E: Environment> FromStr for IdentifierLiteral<E> {
    type Err = Error;

    /// Parses a string into an identifier literal circuit.
    #[inline]
    fn from_str(string: &str) -> Result<Self> {
        match Self::parse(string) {
            Ok((remainder, object)) => {
                ensure!(remainder.is_empty(), "Failed to parse string. Found invalid character in: \"{remainder}\"");
                Ok(object)
            }
            Err(error) => bail!("Failed to parse string. {error}"),
        }
    }
}

impl<E: Environment> TypeName for IdentifierLiteral<E> {
    /// Returns the type name of the circuit as a string.
    #[inline]
    fn type_name() -> &'static str {
        console::IdentifierLiteral::<E::Network>::type_name()
    }
}

impl<E: Environment> Debug for IdentifierLiteral<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<E: Environment> Display for IdentifierLiteral<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.eject_value(), self.eject_mode())
    }
}

/// Validates that the bytes of a field element represent a valid identifier literal.
///
/// Each of the 31 bytes must be in `[a-zA-Z0-9_\0]` (null bytes must be trailing-only).
/// The first byte must be a letter (not digit, underscore, or null).
/// This function converts bytes to bits and delegates to `validate_identifier_bits`,
/// which also checks that padding bits (248..252) are zero.
fn validate_identifier_bytes<E: Environment>(bytes: &[U8<E>; 31]) {
    // Collect all 248 bits from the 31 bytes.
    let mut bits = Vec::with_capacity(248);
    for byte in bytes.iter() {
        byte.write_bits_le(&mut bits);
    }
    // Pad to field size for validate_identifier_bits (it checks padding bits too).
    let field_bits = console::Field::<E::Network>::size_in_bits();
    while bits.len() < field_bits {
        bits.push(Boolean::constant(false));
    }
    // Validate the bits.
    validate_identifier_bits::<E>(&bits);
}

/// Validates that the bits of a field element represent a valid identifier literal.
///
/// Each of the 31 bytes must be in `[a-zA-Z0-9_\0]` (null bytes must be trailing-only).
/// The first byte must be a letter (uppercase or lowercase).
/// The remaining high bits (248..252) must be zero.
///
/// # Circuit Approach
///
/// ASCII characters are validated by examining bit patterns. For each byte (b7..b0):
/// - b7 must be 0 (ASCII requirement)
/// - (b6, b5) determines the character category:
///   - (0,0): null byte (0x00) - valid only as trailing padding
///   - (0,1): digits 0x30-0x39 - requires b4=1 and low nibble ≤ 9
///   - (1,0): uppercase/underscore 0x40-0x5F - valid: A-Z (1-26), _ (31)
///   - (1,1): lowercase 0x60-0x7F - valid: a-z (1-26)
///
/// The validation proceeds in steps:
///     1. Check b7=0 (ASCII high bit)
///     2. Compute category selectors from (b6, b5)
///     3. Compute shared intermediate values for range checks
///     4-7. Validate each category's allowed character range
///     8. Enforce first byte is a letter (not digit, underscore, or null)
///     9. Enforce null bytes appear only as trailing padding
///     10. Check padding bits (248..252) are zero
fn validate_identifier_bits<E: Environment>(bits: &[Boolean<E>]) {
    // The maximum number of bytes in an identifier literal.
    let max_bytes = console::IdentifierLiteral::<E::Network>::SIZE_IN_BYTES;

    // Null flags per byte, collected for trailing-null enforcement in Step 9.
    let mut null_flags: Vec<Boolean<E>> = Vec::with_capacity(max_bytes);

    // Validate each of the 31 bytes.
    for byte_idx in 0..max_bytes {
        let offset = byte_idx * 8;
        // Extract the 8 bits for this byte: b0 (LSB) through b7 (MSB).
        let b0 = &bits[offset];
        let b1 = &bits[offset + 1];
        let b2 = &bits[offset + 2];
        let b3 = &bits[offset + 3];
        let b4 = &bits[offset + 4];
        let b5 = &bits[offset + 5];
        let b6 = &bits[offset + 6];
        let b7 = &bits[offset + 7];

        // Step 1: Assert b7 = 0 (ASCII high bit must be zero).
        E::assert_eq(b7, Boolean::<E>::constant(false)).expect("Identifier literal high bit must be zero");

        // Step 2: Compute category selectors from (b6, b5).
        // sel_00 = (1-b6)*(1-b5) -> null category (0x00).
        let not_b6 = b6.not();
        let not_b5 = b5.not();
        let sel_00 = &not_b6 & &not_b5;
        // sel_01 = (1-b6)*b5 -> digit category (0x30-0x39).
        let sel_01 = &not_b6 & b5;
        // sel_10 = b6*(1-b5) -> uppercase/underscore category (0x40-0x5F).
        let sel_10 = b6 & &not_b5;
        // sel_11 = b6*b5 -> lowercase category (0x60-0x7F).
        let sel_11 = b6 & b5;

        // Step 3: Compute shared intermediates.
        // u1 = b4*b3.
        let u1 = b4 & b3;
        // b1b0 = b1*b0.
        let b1b0 = b1 & b0;
        // z1 = (1-b4)*(1-b3).
        let not_b4 = b4.not();
        let not_b3 = b3.not();
        let z1 = &not_b4 & &not_b3;
        // z2 = z1*(1-b2).
        let not_b2 = b2.not();
        let z2 = &z1 & &not_b2;
        // z3 = z2*(1-b1).
        let not_b1 = b1.not();
        let z3 = &z2 & &not_b1;
        // all_zero_5 = !b4&!b3&!b2&!b1&!b0.
        let not_b0 = b0.not();
        let all_zero_5 = &z3 & &not_b0;

        // Step 4: Null case (b6=0, b5=0 -> byte must be 0x00).
        // sel_00 AND NOT(all bits zero) = false.
        let any_low5 = all_zero_5.clone().not();
        let null_violation = &sel_00 & &any_low5;
        E::assert_eq(&null_violation, Boolean::<E>::constant(false)).expect("Identifier literal null byte violation");

        // Step 5: Digit case (b6=0, b5=1 -> 0x30-0x39, i.e. b4=1 and low4 <= 9).
        // sel_01 * (1-b4) = 0 -> b4 must be 1.
        let digit_b4_violation = &sel_01 & &not_b4;
        E::assert_eq(&digit_b4_violation, Boolean::<E>::constant(false))
            .expect("Identifier literal digit b4 violation");
        // For digits, b3&(b2|b1) means value > 9 -> bad.
        // d1 = b3*b2.
        let d1 = b3 & b2;
        // d2 = b3*b1.
        let d2 = b3 & b1;
        // bad_digit = d1 | d2 = b3&(b2|b1).
        let bad_digit = &d1 | &d2;
        let digit_range_violation = &sel_01 & &bad_digit;
        E::assert_eq(&digit_range_violation, Boolean::<E>::constant(false))
            .expect("Identifier literal digit range violation");

        // Step 6: Uppercase/underscore case (b6=1, b5=0 -> 0x41-0x5A or 0x5F).
        // Valid offsets: 1-26 (A-Z) and 31 (_). Invalid: 0 and 27-30.
        // Offsets 27-30 have b4&b3=1 and b2 XOR (b1&b0) = 1.
        // XOR(b2, b1&b0): true for offsets {27=011,28=100,29=101,30=110},
        //                  false for offsets {24=000,25=001,26=010,31=111}.
        let not_b1b0 = b1b0.clone().not();
        let xor_case_a = b2 & &not_b1b0;
        let xor_case_b = &not_b2 & &b1b0;
        let xor_b2_b1b0 = &xor_case_a | &xor_case_b;
        // bad_upper = u1 & XOR(b2, b1&b0) -> catches only offsets 27-30.
        let bad_upper = &u1 & &xor_b2_b1b0;
        // invalid_upper = offset is 0 or 27-30.
        let invalid_upper = &all_zero_5 | &bad_upper;
        let upper_violation = &sel_10 & &invalid_upper;
        E::assert_eq(&upper_violation, Boolean::<E>::constant(false)).expect("Identifier literal uppercase violation");

        // Step 7: Lowercase case (b6=1, b5=1 -> 0x61-0x7A).
        // Valid offsets: 1-26 (a-z). Invalid: 0 and 27-31.
        // Offsets 27-31 have u1=1 and (b2 OR b1&b0) = 1.
        // Offsets 24-26 have u1=1 but (b2 OR b1&b0) = 0, so they pass.
        let b2_or_b1b0 = b2 | &b1b0;
        let bad_lower = &u1 & &b2_or_b1b0;
        let invalid_lower = &all_zero_5 | &bad_lower;
        let lower_violation = &sel_11 & &invalid_lower;
        E::assert_eq(&lower_violation, Boolean::<E>::constant(false)).expect("Identifier literal lowercase violation");

        // Step 8: First byte must be a letter (not digit, underscore, or null).
        // sel_10 includes both uppercase letters (A-Z, offsets 1-26) and underscore (offset 31).
        // We need to exclude underscore from the "letter" classification.
        // Underscore has offset 31 = 0b11111: u1=1, b1b0=1, b2=1.
        if byte_idx == 0 {
            let is_underscore_offset = &u1 & &b1b0 & b2;
            let is_uppercase_letter = &sel_10 & &is_underscore_offset.not();
            let is_letter = &is_uppercase_letter | &sel_11;
            E::assert(is_letter).expect("Identifier literal must start with a letter");
        }

        // Record the null flag for trailing-null enforcement (Step 9).
        null_flags.push(sel_00);
    }

    // Step 9: Enforce trailing nulls.
    // Once a null byte appears, all subsequent bytes must also be null.
    // For each consecutive pair: null_flags[i-1] * (1 - null_flags[i]) = 0.
    for i in 1..max_bytes {
        let not_null = null_flags[i].clone().not();
        E::enforce(|| (&null_flags[i - 1], &not_null, E::zero())).expect("Identifier literal trailing null violation");
    }

    // Validate that the padding bits (248..252) are zero.
    let data_bits = max_bytes * 8;
    let field_bits = bits.len();
    for bit in bits.iter().take(field_bits).skip(data_bits) {
        E::assert_eq(bit, Boolean::<E>::constant(false)).expect("Identifier literal padding bit must be zero");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;

    type CurrentEnvironment = Circuit;

    /// Test strings covering various identifier patterns.
    const TEST_STRINGS: &[&str] = &["a", "hello", "hello_world", "Test123", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcde"];

    fn check_new(
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

            Circuit::scope(format!("new {mode}"), || {
                // Inject the identifier literal into the circuit.
                let candidate = IdentifierLiteral::<CurrentEnvironment>::new(mode, expected);
                // Check that the ejected value matches the expected value.
                assert_eq!(expected, candidate.eject_value());
                // Check that the ejected mode matches.
                assert_eq!(mode, candidate.eject_mode());
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });
            Circuit::reset();
        }
        Ok(())
    }

    #[test]
    fn test_new_constant() -> Result<()> {
        check_new(Mode::Constant, 248, 0, 0, 0)
    }

    #[test]
    fn test_new_public() -> Result<()> {
        check_new(Mode::Public, 0, 248, 810, 1275)
    }

    #[test]
    fn test_new_private() -> Result<()> {
        check_new(Mode::Private, 0, 0, 1058, 1275)
    }

    #[test]
    fn test_non_trailing_null_unsatisfied() {
        use console::FromBytes;

        // Construct a field with bytes [0x61, 0x00, 0x62, 0x00, ...] — non-null after null.
        let mut bad_bytes = vec![0u8; 32];
        bad_bytes[0] = b'a';
        bad_bytes[2] = b'b';
        let field_value = console::Field::<<CurrentEnvironment as Environment>::Network>::from_bytes_le(&bad_bytes)
            .expect("Failed to construct field from bytes");

        // Inject the field and validate in a scope.
        Circuit::scope("test_non_trailing_null", || {
            let field = Field::<CurrentEnvironment>::new(Mode::Private, field_value);
            let _candidate = IdentifierLiteral::<CurrentEnvironment>::from_field(field);
        });
        // The circuit must be unsatisfied due to the trailing-null violation.
        assert!(!Circuit::is_satisfied());
        Circuit::reset();
    }

    #[test]
    fn test_first_char_must_be_letter() {
        use console::FromBytes;

        // Construct a field with bytes [0x31, ...] — starts with '1' (digit).
        let mut digit_start_bytes = vec![0u8; 32];
        digit_start_bytes[0] = b'1';
        let field_value =
            console::Field::<<CurrentEnvironment as Environment>::Network>::from_bytes_le(&digit_start_bytes)
                .expect("Failed to construct field from bytes");

        // Inject the field and validate in a scope.
        Circuit::scope("test_first_char_digit", || {
            let field = Field::<CurrentEnvironment>::new(Mode::Private, field_value);
            let _candidate = IdentifierLiteral::<CurrentEnvironment>::from_field(field);
        });
        // The circuit must be unsatisfied due to first character not being a letter.
        assert!(!Circuit::is_satisfied());
        Circuit::reset();

        // Construct a field with bytes [0x5F, ...] — starts with '_' (underscore).
        let mut underscore_start_bytes = vec![0u8; 32];
        underscore_start_bytes[0] = b'_';
        let field_value =
            console::Field::<<CurrentEnvironment as Environment>::Network>::from_bytes_le(&underscore_start_bytes)
                .expect("Failed to construct field from bytes");

        // Inject the field and validate in a scope.
        Circuit::scope("test_first_char_underscore", || {
            let field = Field::<CurrentEnvironment>::new(Mode::Private, field_value);
            let _candidate = IdentifierLiteral::<CurrentEnvironment>::from_field(field);
        });
        // The circuit must be unsatisfied due to first character not being a letter.
        assert!(!Circuit::is_satisfied());
        Circuit::reset();
    }

    #[test]
    fn test_size_in_bits() {
        assert_eq!(IdentifierLiteral::<CurrentEnvironment>::size_in_bits(), 248);
    }

    #[test]
    fn test_ascii_character_validation() {
        use console::FromBytes;

        // Test all 256 possible first byte values.
        for byte in 0u8..=255 {
            let mut raw_bytes = vec![0u8; 32];
            raw_bytes[0] = byte;
            let field_value = console::Field::<<CurrentEnvironment as Environment>::Network>::from_bytes_le(&raw_bytes)
                .expect("Failed to construct field");

            Circuit::scope(format!("first_byte_{byte}"), || {
                let field = Field::<CurrentEnvironment>::new(Mode::Private, field_value);
                let _candidate = IdentifierLiteral::<CurrentEnvironment>::from_field(field);
            });

            // Only a-z and A-Z should satisfy the circuit.
            let expected_valid = byte.is_ascii_alphabetic();
            assert_eq!(
                Circuit::is_satisfied(),
                expected_valid,
                "First byte {byte}: expected satisfied={expected_valid}"
            );
            Circuit::reset();
        }

        // Test all 256 possible second byte values (with valid first byte).
        for byte in 0u8..=255 {
            let mut raw_bytes = vec![0u8; 32];
            raw_bytes[0] = b'a'; // Valid first byte.
            raw_bytes[1] = byte;
            let field_value = console::Field::<<CurrentEnvironment as Environment>::Network>::from_bytes_le(&raw_bytes)
                .expect("Failed to construct field");

            Circuit::scope(format!("second_byte_{byte}"), || {
                let field = Field::<CurrentEnvironment>::new(Mode::Private, field_value);
                let _candidate = IdentifierLiteral::<CurrentEnvironment>::from_field(field);
            });

            // a-z, A-Z, 0-9, _, and null should satisfy the circuit.
            let expected_valid = byte.is_ascii_alphanumeric() || byte == b'_' || byte == 0;
            assert_eq!(
                Circuit::is_satisfied(),
                expected_valid,
                "Second byte {byte}: expected satisfied={expected_valid}"
            );
            Circuit::reset();
        }
    }
}
