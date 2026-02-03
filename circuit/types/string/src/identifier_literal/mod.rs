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

use snarkvm_circuit_environment::prelude::*;
use snarkvm_circuit_types_boolean::Boolean;
use snarkvm_circuit_types_field::Field;
use snarkvm_circuit_types_integers::U8;

#[cfg(test)]
use snarkvm_circuit_environment::assert_scope;

/// A circuit identifier literal storing an ASCII string (up to 31 bytes) as a byte array.
///
/// The circuit validates that every byte is a valid identifier character
/// (`[a-zA-Z0-9_\0]`), that the first byte is a letter, and that null bytes
/// appear only as trailing padding.
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

    /// Constructs an identifier literal from circuit bytes, validating the contents.
    fn from_bytes(bytes: [U8<E>; 31]) -> Self {
        // Validate the bytes.
        validate_identifier_bytes::<E>(&bytes);

        Self { bytes }
    }
}

impl<E: Environment> Inject for IdentifierLiteral<E> {
    type Primitive = console::IdentifierLiteral<E::Network>;

    /// Initializes a new circuit identifier literal from a primitive.
    fn new(mode: Mode, value: Self::Primitive) -> Self {
        // Access the raw bytes from the console identifier literal.
        let raw_bytes = value.bytes();
        // Inject each byte into the circuit.
        let bytes: [U8<E>; 31] = std::array::from_fn(|i| U8::new(mode, console::Integer::new(raw_bytes[i])));
        // Validate and construct.
        Self::from_bytes(bytes)
    }
}

impl<E: Environment> Eject for IdentifierLiteral<E> {
    type Primitive = console::IdentifierLiteral<E::Network>;

    /// Ejects the mode of the identifier literal.
    fn eject_mode(&self) -> Mode {
        self.bytes.eject_mode()
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
/// This function converts bytes to bits and delegates to `validate_identifier_bits`.
fn validate_identifier_bytes<E: Environment>(bytes: &[U8<E>; 31]) {
    // Collect all 248 bits from the 31 bytes.
    let mut bits = Vec::with_capacity(248);
    for byte in bytes.iter() {
        byte.write_bits_le(&mut bits);
    }
    // Validate the bits.
    validate_identifier_bits::<E>(&bits);
}

/// Validates that the bits represent a valid identifier literal.
///
/// Expects exactly 248 bits (31 bytes). Each byte must be in `[a-zA-Z0-9_\0]`.
/// The first byte must be a letter. Null bytes must be trailing-only.
fn validate_identifier_bits<E: Environment>(bits: &[Boolean<E>]) {
    let size_in_bytes = console::IdentifierLiteral::<E::Network>::SIZE_IN_BYTES;
    let size_in_bits = size_in_bytes * 8;

    // Sanity check: requires exactly SIZE_IN_BITS bits.
    assert_eq!(
        bits.len(),
        size_in_bits,
        "validate_identifier_bits requires exactly {size_in_bits} bits, got {}",
        bits.len()
    );

    // Validate each byte and collect null flags.
    let mut null_flags: Vec<Boolean<E>> = Vec::with_capacity(size_in_bytes);
    for byte_idx in 0..size_in_bytes {
        let offset = byte_idx * 8;
        // Convert the slice to a fixed-size array reference.
        // Safety: always succeeds since we iterate in 8-bit chunks within bounds.
        let byte_bits = <&[Boolean<E>; 8]>::try_from(&bits[offset..offset + 8]).expect("slice length is exactly 8");
        let null_flag = validate_byte::<E>(byte_bits, byte_idx == 0);
        null_flags.push(null_flag);
    }

    // Enforce trailing nulls.
    validate_trailing_nulls::<E>(&null_flags);
}

/// Category selectors derived from bits b5 and b6.
/// Determines which ASCII range the byte falls into.
///
/// # Category Selection by (b6, b5)
///
/// | b6 | b5 | Hex Range   | Category                             |
/// |----|----|-------------|--------------------------------------|
/// | 0  | 0  | 0x00-0x1F   | Control (only 0x00 valid)            |
/// | 0  | 1  | 0x20-0x3F   | Symbols/Digits (only 0x30-0x39 valid)|
/// | 1  | 0  | 0x40-0x5F   | Uppercase/Symbols (A-Z, _ valid)     |
/// | 1  | 1  | 0x60-0x7F   | Lowercase/Symbols (a-z valid)        |
struct CategorySelectors<E: Environment> {
    /// (0,0): Null/control category - only 0x00 is valid.
    null: Boolean<E>,
    /// (0,1): Digit category - only 0x30-0x39 valid.
    digit: Boolean<E>,
    /// (1,0): Uppercase category - A-Z and _ valid.
    upper: Boolean<E>,
    /// (1,1): Lowercase category - a-z valid.
    lower: Boolean<E>,
}

impl<E: Environment> CategorySelectors<E> {
    /// Computes category selectors from bits b5 and b6.
    fn new(b5: &Boolean<E>, b6: &Boolean<E>) -> Self {
        let not_b5 = b5.not();
        let not_b6 = b6.not();
        Self { null: &not_b6 & &not_b5, digit: &not_b6 & b5, upper: b6 & &not_b5, lower: b6 & b5 }
    }
}

/// Precomputed intermediate values for byte validation.
/// Sharing intermediates saves ~300 constraints compared to recomputing per-predicate.
///
/// # Field Descriptions
///
/// - `u1`: b4 & b3 - True when offset >= 24 (high bits of 5-bit offset set).
/// - `b1b0`: b1 & b0 - True when both low bits set (used in upper/lower validation).
/// - `not_b2`: !b2 - Negation of b2 (for XOR computation).
/// - `all_zero_5`: !b4 & !b3 & !b2 & !b1 & !b0 - True when offset is 0.
struct ByteValidationData<E: Environment> {
    /// b4 & b3: True when offset >= 24.
    u1: Boolean<E>,
    /// b1 & b0: Both low bits set.
    b1b0: Boolean<E>,
    /// !b2: Negation of b2.
    not_b2: Boolean<E>,
    /// Low 5 bits all zero (offset = 0).
    all_zero_5: Boolean<E>,
}

impl<E: Environment> ByteValidationData<E> {
    /// Computes intermediate values from raw bits.
    fn new(b0: &Boolean<E>, b1: &Boolean<E>, b2: &Boolean<E>, b3: &Boolean<E>, b4: &Boolean<E>) -> Self {
        // u1 = b4 & b3 (offset >= 24).
        let u1 = b4 & b3;

        // b1b0 = b1 & b0 (both low bits set).
        let b1b0 = b1 & b0;

        // not_b2 = !b2 (for XOR computation).
        let not_b2 = b2.not();

        // Compute all_zero_5 = !b4 & !b3 & !b2 & !b1 & !b0 (offset is 0).
        let not_b4 = b4.not();
        let not_b3 = b3.not();
        let not_b1 = b1.not();
        let not_b0 = b0.not();
        let z1 = &not_b4 & &not_b3;
        let z2 = &z1 & &not_b2;
        let z3 = &z2 & &not_b1;
        let all_zero_5 = &z3 & &not_b0;

        Self { u1, b1b0, not_b2, all_zero_5 }
    }
}

/// Returns true if the digit range check fails (low nibble > 9).
///
/// # Truth Table (b0 not needed since it doesn't affect whether nibble > 9)
///
/// | Nibble | b3 | b2 | b1 | b3&(b2|b1) | Valid (≤9)? |
/// |--------|----|----|----| -----------|-------------|
/// |  0-7   | 0  | *  | *  |     0      |    Yes      |
/// |   8    | 1  | 0  | 0  |     0      |    Yes      |
/// |   9    | 1  | 0  | 0  |     0      |    Yes      |
/// | 10-15  | 1  | 1+ | *  |     1      |    No       |
///
/// Note: 8=0b1000 and 9=0b1001 both have b3=1,b2=0,b1=0; they differ only in b0.
fn is_invalid_digit_nibble<E: Environment>(b1: &Boolean<E>, b2: &Boolean<E>, b3: &Boolean<E>) -> Boolean<E> {
    // bad_digit = b3 & (b2 | b1) = (b3 & b2) | (b3 & b1).
    let d1 = b3 & b2;
    let d2 = b3 & b1;
    &d1 | &d2
}

/// Returns true if the 5-bit offset represents an invalid uppercase character.
/// Valid offsets: 1-26 (A-Z) and 31 (_). Invalid: 0, 27-30.
///
/// # ASCII Mapping
/// Offset 0 = 0x40 '@' (invalid)
/// Offset 1-26 = 0x41-0x5A 'A'-'Z' (valid)
/// Offset 27-30 = 0x5B-0x5E '[', '\', ']', '^' (invalid)
/// Offset 31 = 0x5F '_' (valid)
///
/// # Truth Table for Offsets 24-31 (where b4=1, b3=1)
///
/// | Offset | Binary  | b2 | b1&b0 | XOR(b2,b1&b0) | Char | Valid? |
/// |--------|---------|----| ------|---------------|------|--------|
/// |   24   | 11000   | 0  |   0   |      0        |  X   |  Yes   |
/// |   25   | 11001   | 0  |   0   |      0        |  Y   |  Yes   |
/// |   26   | 11010   | 0  |   0   |      0        |  Z   |  Yes   |
/// |   27   | 11011   | 0  |   1   |      1        |  [   |  No    |
/// |   28   | 11100   | 1  |   0   |      1        |  \   |  No    |
/// |   29   | 11101   | 1  |   0   |      1        |  ]   |  No    |
/// |   30   | 11110   | 1  |   0   |      1        |  ^   |  No    |
/// |   31   | 11111   | 1  |   1   |      0        |  _   |  Yes   |
///
/// Key insight: `(b4 & b3) & XOR(b2, b1&b0)` is true exactly for offsets 27-30.
fn is_invalid_upper_offset<E: Environment>(data: &ByteValidationData<E>, b2: &Boolean<E>) -> Boolean<E> {
    // XOR(b2, b1b0) = (b2 & !b1b0) | (!b2 & b1b0).
    let not_b1b0 = data.b1b0.clone().not();
    let xor_case_a = b2 & &not_b1b0;
    let xor_case_b = &data.not_b2 & &data.b1b0;
    let xor_b2_b1b0 = &xor_case_a | &xor_case_b;
    // bad_upper catches exactly offsets 27-30.
    let bad_upper = &data.u1 & &xor_b2_b1b0;
    // invalid = offset is 0 or 27-30.
    &data.all_zero_5 | &bad_upper
}

/// Returns true if the 5-bit offset represents an invalid lowercase character.
/// Valid offsets: 1-26 (a-z). Invalid: 0, 27-31.
///
/// # ASCII Mapping
/// Offset 0 = 0x60 '`' (invalid)
/// Offset 1-26 = 0x61-0x7A 'a'-'z' (valid)
/// Offset 27-31 = 0x7B-0x7F '{', '|', '}', '~', DEL (invalid)
///
/// # Truth Table for Offsets 24-31 (where b4=1, b3=1)
///
/// | Offset | Binary  | b2 | b1&b0 | b2|(b1&b0) | Char | Valid? |
/// |--------|---------|----| ------|------------|------|--------|
/// |   24   | 11000   | 0  |   0   |     0      |  x   |  Yes   |
/// |   25   | 11001   | 0  |   0   |     0      |  y   |  Yes   |
/// |   26   | 11010   | 0  |   0   |     0      |  z   |  Yes   |
/// |   27   | 11011   | 0  |   1   |     1      |  {   |  No    |
/// |   28   | 11100   | 1  |   0   |     1      |  |   |  No    |
/// |   29   | 11101   | 1  |   0   |     1      |  }   |  No    |
/// |   30   | 11110   | 1  |   0   |     1      |  ~   |  No    |
/// |   31   | 11111   | 1  |   1   |     1      | DEL  |  No    |
///
/// Key insight: `(b4 & b3) & (b2 | (b1&b0))` is true exactly for offsets 27-31.
fn is_invalid_lower_offset<E: Environment>(data: &ByteValidationData<E>, b2: &Boolean<E>) -> Boolean<E> {
    // bad_lower = u1 & (b2 | b1b0) catches offsets 27-31.
    let b2_or_b1b0 = b2 | &data.b1b0;
    let bad_lower = &data.u1 & &b2_or_b1b0;
    // invalid = offset is 0 or 27-31.
    &data.all_zero_5 | &bad_lower
}

/// Validates a single byte of an identifier literal.
/// Returns the null flag (true if this byte is 0x00) for trailing-null enforcement.
///
/// # Validation Rules
/// - b7 must be 0 (ASCII).
/// - Byte must be in [a-zA-Z0-9_\0].
/// - If `is_first_byte`, must be a letter (not digit, underscore, or null).
///
/// # Valid Character Ranges
///
/// | Category       | ASCII Range | Hex Range   | Bits (b7..b0)          |
/// |----------------|-------------|-------------|------------------------|
/// | Null           | NUL         | 0x00        | 0000_0000              |
/// | Digits         | '0'-'9'     | 0x30-0x39   | 0011_0000 - 0011_1001  |
/// | Uppercase      | 'A'-'Z'     | 0x41-0x5A   | 0100_0001 - 0101_1010  |
/// | Underscore     | '_'         | 0x5F        | 0101_1111              |
/// | Lowercase      | 'a'-'z'     | 0x61-0x7A   | 0110_0001 - 0111_1010  |
///
/// # Category Selection by (b6, b5)
///
/// | b6 | b5 | Hex Range   | Category                             |
/// |----|----|-------------|--------------------------------------|
/// | 0  | 0  | 0x00-0x1F   | Control (only 0x00 valid)            |
/// | 0  | 1  | 0x20-0x3F   | Symbols/Digits (only 0x30-0x39 valid)|
/// | 1  | 0  | 0x40-0x5F   | Uppercase/Symbols (only A-Z, _ valid)|
/// | 1  | 1  | 0x60-0x7F   | Lowercase/Symbols (only a-z valid)   |
fn validate_byte<E: Environment>(bits: &[Boolean<E>; 8], is_first_byte: bool) -> Boolean<E> {
    let [b0, b1, b2, b3, b4, b5, b6, b7] = bits;

    // Assert b7 = 0 (ASCII high bit must be zero).
    E::assert_eq(b7, Boolean::<E>::constant(false)).expect("Identifier literal high bit must be zero");

    // Compute category selectors from (b6, b5).
    let cat = CategorySelectors::new(b5, b6);

    // Compute shared intermediates for offset validation.
    let data = ByteValidationData::new(b0, b1, b2, b3, b4);

    // Validate null category: byte must be exactly 0x00.
    let any_low5 = data.all_zero_5.clone().not();
    let null_violation = &cat.null & &any_low5;
    E::assert_eq(&null_violation, Boolean::<E>::constant(false)).expect("Identifier literal null byte violation");

    // Validate digit category: must have b4=1 and low nibble <= 9.
    let not_b4 = b4.not();
    let digit_b4_violation = &cat.digit & &not_b4;
    E::assert_eq(&digit_b4_violation, Boolean::<E>::constant(false)).expect("Identifier literal digit b4 violation");
    let bad_digit_nibble = is_invalid_digit_nibble::<E>(b1, b2, b3);
    let digit_range_violation = &cat.digit & &bad_digit_nibble;
    E::assert_eq(&digit_range_violation, Boolean::<E>::constant(false))
        .expect("Identifier literal digit range violation");

    // Validate uppercase category: offsets 1-26 (A-Z) and 31 (_) valid.
    let invalid_upper = is_invalid_upper_offset::<E>(&data, b2);
    let upper_violation = &cat.upper & &invalid_upper;
    E::assert_eq(&upper_violation, Boolean::<E>::constant(false)).expect("Identifier literal uppercase violation");

    // Validate lowercase category: offsets 1-26 (a-z) valid.
    let invalid_lower = is_invalid_lower_offset::<E>(&data, b2);
    let lower_violation = &cat.lower & &invalid_lower;
    E::assert_eq(&lower_violation, Boolean::<E>::constant(false)).expect("Identifier literal lowercase violation");

    // First byte must be a letter (not digit, underscore, or null).
    if is_first_byte {
        let is_underscore_offset = &data.u1 & &data.b1b0 & b2;
        let is_uppercase_letter = &cat.upper & &is_underscore_offset.not();
        let is_letter = &is_uppercase_letter | &cat.lower;
        E::assert(is_letter).expect("Identifier literal must start with a letter");
    }

    // Return the null flag for trailing-null enforcement.
    cat.null
}

/// Enforces that once a null byte appears, all subsequent bytes must be null.
fn validate_trailing_nulls<E: Environment>(null_flags: &[Boolean<E>]) {
    // For each consecutive pair: null_flags[i-1] * (1 - null_flags[i]) = 0.
    for i in 1..null_flags.len() {
        let not_null = null_flags[i].clone().not();
        E::enforce(|| (&null_flags[i - 1], &not_null, E::zero())).expect("Identifier literal trailing null violation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;
    use snarkvm_utilities::{TestRng, Uniform};

    type CurrentEnvironment = Circuit;

    const ITERATIONS: usize = 10;

    fn check_new(
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
        // 248 public bits + 810 private intermediates, 1275 constraints.
        check_new(Mode::Public, 0, 248, 810, 1275)
    }

    #[test]
    fn test_new_private() -> Result<()> {
        // 248 private bits + 810 private intermediates = 1058 private, 1275 constraints.
        check_new(Mode::Private, 0, 0, 1058, 1275)
    }

    #[test]
    fn test_new_max_length_identifier() -> Result<()> {
        // Test the maximally large identifier (31 characters, no null padding).
        let max_str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcde";
        assert_eq!(
            max_str.len(),
            console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::SIZE_IN_BYTES
        );

        let expected =
            console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::new(max_str).unwrap();

        Circuit::scope("new max length", || {
            let candidate = IdentifierLiteral::<CurrentEnvironment>::new(Mode::Private, expected);
            assert_eq!(expected, candidate.eject_value());
            assert!(Circuit::is_satisfied());
        });
        Circuit::reset();
        Ok(())
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
            // Check positive case: valid bytes should satisfy the circuit.
            if expected_valid {
                assert!(Circuit::is_satisfied(), "First byte {byte}: expected circuit to be satisfied");
            } else {
                // Check negative case: invalid bytes should NOT satisfy the circuit.
                assert!(!Circuit::is_satisfied(), "First byte {byte}: expected circuit to be unsatisfied");
            }
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
            // Check positive case: valid bytes should satisfy the circuit.
            if expected_valid {
                assert!(Circuit::is_satisfied(), "Second byte {byte}: expected circuit to be satisfied");
            } else {
                // Check negative case: invalid bytes should NOT satisfy the circuit.
                assert!(!Circuit::is_satisfied(), "Second byte {byte}: expected circuit to be unsatisfied");
            }
            Circuit::reset();
        }
    }

    /// Helper to convert a byte to 8 Boolean circuit values (LSB first).
    fn byte_to_bits(byte: u8, mode: Mode) -> [Boolean<CurrentEnvironment>; 8] {
        std::array::from_fn(|i| Boolean::new(mode, (byte >> i) & 1 == 1))
    }

    /// Asserts that padding bits beyond the identifier data are zero.
    fn validate_padding_bits(bits: &[Boolean<CurrentEnvironment>], data_bits: usize) {
        for bit in bits.iter().skip(data_bits) {
            CurrentEnvironment::assert_eq(bit, Boolean::<CurrentEnvironment>::constant(false))
                .expect("Identifier literal padding bit must be zero");
        }
    }

    /// Validates a single byte using validate_byte and returns whether the circuit is satisfied.
    fn check_validate_byte(byte: u8, is_first_byte: bool) -> bool {
        Circuit::scope(format!("validate_byte_{byte}"), || {
            let bits = byte_to_bits(byte, Mode::Private);
            let _null_flag = validate_byte::<CurrentEnvironment>(&bits, is_first_byte);
        });
        let satisfied = Circuit::is_satisfied();
        Circuit::reset();
        satisfied
    }

    #[test]
    fn test_validate_byte_constraint_counts() {
        // Verify constraint counts for validate_byte.
        // First byte (with is_letter check): 8 private inputs + 30 intermediates = 38 private, 45 constraints.
        Circuit::scope("validate_byte_first", || {
            let bits = byte_to_bits(b'a', Mode::Private);
            let _null_flag = validate_byte::<CurrentEnvironment>(&bits, true);
            assert_scope!(0, 0, 38, 45);
        });
        Circuit::reset();

        // Non-first byte (without is_letter check): 8 private inputs + 26 intermediates = 34 private, 40 constraints.
        Circuit::scope("validate_byte_non_first", || {
            let bits = byte_to_bits(b'a', Mode::Private);
            let _null_flag = validate_byte::<CurrentEnvironment>(&bits, false);
            assert_scope!(0, 0, 34, 40);
        });
        Circuit::reset();
    }

    #[test]
    fn test_validate_trailing_nulls_constraint_counts() {
        // Verify constraint counts for validate_trailing_nulls.
        // For n flags, we have n-1 constraints (one per consecutive pair), plus n-1 intermediates for not().
        // However, E::enforce adds additional constraints.
        Circuit::scope("trailing_nulls_3", || {
            let null_flags: Vec<Boolean<CurrentEnvironment>> =
                [false, false, true].iter().map(|&b| Boolean::new(Mode::Private, b)).collect();
            validate_trailing_nulls::<CurrentEnvironment>(&null_flags);
            // 3 private inputs, 5 constraints (E::enforce adds more than just 1 per pair).
            assert_scope!(0, 0, 3, 5);
        });
        Circuit::reset();

        Circuit::scope("trailing_nulls_31", || {
            let null_flags: Vec<Boolean<CurrentEnvironment>> =
                (0..31).map(|_| Boolean::new(Mode::Private, false)).collect();
            validate_trailing_nulls::<CurrentEnvironment>(&null_flags);
            // 31 private inputs, 61 constraints.
            assert_scope!(0, 0, 31, 61);
        });
        Circuit::reset();
    }

    #[test]
    fn test_validate_padding_bits_constraint_counts() {
        // Verify constraint counts for validate_padding_bits.
        // Each padding bit beyond data_bits adds constraints via assert_eq.
        Circuit::scope("padding_2_bits", || {
            let bits: Vec<Boolean<CurrentEnvironment>> = (0..10).map(|_| Boolean::new(Mode::Private, false)).collect();
            validate_padding_bits(&bits, 8);
            // 10 private inputs, 12 constraints.
            assert_scope!(0, 0, 10, 12);
        });
        Circuit::reset();
    }

    #[test]
    fn test_validate_identifier_bits_constraint_counts() {
        // Verify total constraint counts.
        // Using ByteValidationData struct to share intermediates saves ~300 constraints:
        // - First byte: 38 private, 45 constraints (includes is_letter check).
        // - Non-first bytes (30): 34 private each = 1020 private, 40 constraints each = 1200 constraints.
        // - Trailing nulls (30 pairs): ~30 intermediates, 61 constraints.
        // Total: 248 bits + 810 intermediates = 1058 private, 1275 constraints.
        Circuit::scope("validate_identifier_bits", || {
            // Create 248 bits (31 bytes) representing a valid identifier "a" followed by nulls.
            let mut bits: Vec<Boolean<CurrentEnvironment>> = Vec::with_capacity(248);
            // First byte: 'a' = 0x61 = 0b01100001.
            bits.extend(byte_to_bits(b'a', Mode::Private));
            // Remaining 30 bytes: null (0x00).
            for _ in 1..31 {
                bits.extend(byte_to_bits(0x00, Mode::Private));
            }
            validate_identifier_bits::<CurrentEnvironment>(&bits);
            assert_scope!(0, 0, 1058, 1275);
        });
        assert!(Circuit::is_satisfied());
        Circuit::reset();
    }

    #[test]
    fn test_validate_byte_lowercase_a_to_z() {
        // All lowercase letters should be valid as first or non-first byte.
        for ch in b'a'..=b'z' {
            assert!(check_validate_byte(ch, true), "Lowercase {ch} should be valid as first byte");
            assert!(check_validate_byte(ch, false), "Lowercase {ch} should be valid as non-first byte");
        }
    }

    #[test]
    fn test_validate_byte_uppercase_a_to_z() {
        // All uppercase letters should be valid as first or non-first byte.
        for ch in b'A'..=b'Z' {
            assert!(check_validate_byte(ch, true), "Uppercase {ch} should be valid as first byte");
            assert!(check_validate_byte(ch, false), "Uppercase {ch} should be valid as non-first byte");
        }
    }

    #[test]
    fn test_validate_byte_digits_0_to_9() {
        // Digits should be invalid as first byte, valid as non-first byte.
        for ch in b'0'..=b'9' {
            assert!(!check_validate_byte(ch, true), "Digit {ch} should be invalid as first byte");
            assert!(check_validate_byte(ch, false), "Digit {ch} should be valid as non-first byte");
        }
    }

    #[test]
    fn test_validate_byte_underscore() {
        // Underscore should be invalid as first byte, valid as non-first byte.
        assert!(!check_validate_byte(b'_', true), "Underscore should be invalid as first byte");
        assert!(check_validate_byte(b'_', false), "Underscore should be valid as non-first byte");
    }

    #[test]
    fn test_validate_byte_null() {
        // Null should be invalid as first byte, valid as non-first byte (as trailing padding).
        assert!(!check_validate_byte(0x00, true), "Null should be invalid as first byte");
        assert!(check_validate_byte(0x00, false), "Null should be valid as non-first byte");
    }

    #[test]
    fn test_validate_byte_invalid_chars() {
        // Test a selection of invalid characters.
        let invalid_chars = [b' ', b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'-', b'+', b'='];
        for ch in invalid_chars {
            assert!(!check_validate_byte(ch, true), "Char {ch} should be invalid as first byte");
            assert!(!check_validate_byte(ch, false), "Char {ch} should be invalid as non-first byte");
        }
    }

    #[test]
    fn test_validate_byte_non_ascii() {
        // Non-ASCII bytes (high bit set) should be invalid.
        for ch in 128u8..=255 {
            assert!(!check_validate_byte(ch, true), "Non-ASCII {ch} should be invalid as first byte");
            assert!(!check_validate_byte(ch, false), "Non-ASCII {ch} should be invalid as non-first byte");
        }
    }

    #[test]
    fn test_validate_trailing_nulls_valid() {
        // Valid: no nulls, or nulls only at the end.
        let patterns: &[&[bool]] = &[
            &[false, false, false], // No nulls.
            &[false, false, true],  // One trailing null.
            &[false, true, true],   // Two trailing nulls.
            &[true, true, true],    // All nulls (valid for trailing check, first-byte check is separate).
        ];

        for pattern in patterns {
            Circuit::scope("trailing_nulls_valid", || {
                let null_flags: Vec<Boolean<CurrentEnvironment>> =
                    pattern.iter().map(|&b| Boolean::new(Mode::Private, b)).collect();
                validate_trailing_nulls::<CurrentEnvironment>(&null_flags);
            });
            assert!(Circuit::is_satisfied(), "Pattern {pattern:?} should be valid");
            Circuit::reset();
        }
    }

    #[test]
    fn test_validate_trailing_nulls_invalid() {
        // Invalid: non-null after null.
        let patterns: &[&[bool]] = &[
            &[true, false, false], // Non-null after null.
            &[false, true, false], // Non-null after null.
            &[true, false, true],  // Non-null sandwiched.
            &[true, true, false],  // Non-null at end after nulls.
        ];

        for pattern in patterns {
            Circuit::scope("trailing_nulls_invalid", || {
                let null_flags: Vec<Boolean<CurrentEnvironment>> =
                    pattern.iter().map(|&b| Boolean::new(Mode::Private, b)).collect();
                validate_trailing_nulls::<CurrentEnvironment>(&null_flags);
            });
            assert!(!Circuit::is_satisfied(), "Pattern {pattern:?} should be invalid");
            Circuit::reset();
        }
    }

    #[test]
    fn test_validate_padding_bits_valid() {
        // Valid: padding bits are all zero.
        Circuit::scope("padding_valid", || {
            // Create bits where first 8 are data (can be any value), rest are padding (must be zero).
            let mut bits: Vec<Boolean<CurrentEnvironment>> =
                (0..10).map(|_| Boolean::new(Mode::Private, false)).collect();
            bits[0] = Boolean::new(Mode::Private, true); // Data bits can be anything.
            validate_padding_bits(&bits, 8);
        });
        assert!(Circuit::is_satisfied());
        Circuit::reset();
    }

    #[test]
    fn test_validate_padding_bits_invalid() {
        // Invalid: padding bit is one.
        Circuit::scope("padding_invalid", || {
            let mut bits: Vec<Boolean<CurrentEnvironment>> =
                (0..10).map(|_| Boolean::new(Mode::Private, false)).collect();
            bits[9] = Boolean::new(Mode::Private, true); // Padding bit is 1.
            validate_padding_bits(&bits, 8);
        });
        assert!(!Circuit::is_satisfied());
        Circuit::reset();
    }

    #[test]
    fn test_validate_byte_exhaustive() {
        // Test ALL 256 byte values × 2 is_first_byte values = 512 combinations.
        for byte_value in 0u8..=255 {
            for is_first in [true, false] {
                let expected_valid = match (is_first, byte_value) {
                    // First byte: only letters valid.
                    (true, b'A'..=b'Z') | (true, b'a'..=b'z') => true,
                    (true, _) => false,
                    // Non-first byte: letters, digits, underscore, null valid.
                    (false, b'A'..=b'Z') | (false, b'a'..=b'z') => true,
                    (false, b'0'..=b'9') | (false, b'_') | (false, 0x00) => true,
                    (false, _) => false,
                };

                Circuit::scope(format!("byte_{byte_value}_first_{is_first}"), || {
                    let bits = byte_to_bits(byte_value, Mode::Private);
                    let _null_flag = validate_byte::<CurrentEnvironment>(&bits, is_first);
                });

                assert_eq!(
                    Circuit::is_satisfied(),
                    expected_valid,
                    "byte={byte_value} (0x{byte_value:02x}, '{}'), is_first={is_first}: expected {}",
                    if byte_value.is_ascii_graphic() { byte_value as char } else { '?' },
                    if expected_valid { "satisfied" } else { "unsatisfied" }
                );
                Circuit::reset();
            }
        }
    }

    #[test]
    fn test_validate_trailing_nulls_exhaustive() {
        // Test all 2^n combinations for n = 1 to 6 (64 combinations for n=6).
        for n in 1..=6usize {
            for pattern in 0..(1 << n) {
                // Convert pattern to bool array (bit i = whether byte i is null).
                let null_flags: Vec<bool> = (0..n).map(|i| (pattern >> i) & 1 == 1).collect();

                // Valid iff nulls are trailing-only: once a null appears, all subsequent must be null.
                // Equivalently: no 0 appears after a 1 in the null_flags array.
                let expected_valid = {
                    let mut seen_null = false;
                    let mut valid = true;
                    for &is_null in &null_flags {
                        if seen_null && !is_null {
                            valid = false;
                            break;
                        }
                        seen_null |= is_null;
                    }
                    valid
                };

                Circuit::scope(format!("trailing_n{n}_p{pattern}"), || {
                    let flags: Vec<Boolean<CurrentEnvironment>> =
                        null_flags.iter().map(|&b| Boolean::new(Mode::Private, b)).collect();
                    validate_trailing_nulls::<CurrentEnvironment>(&flags);
                });

                assert_eq!(
                    Circuit::is_satisfied(),
                    expected_valid,
                    "n={n}, pattern={pattern:0width$b}: expected {}",
                    if expected_valid { "satisfied" } else { "unsatisfied" },
                    width = n
                );
                Circuit::reset();
            }
        }
    }

    #[test]
    fn test_is_invalid_digit_nibble_exhaustive() {
        // Test all 16 nibble values (0-15).
        // The function uses only b1, b2, b3 (not b0), but we test all nibbles for clarity.
        for nibble in 0u8..16 {
            let b1 = (nibble >> 1) & 1 == 1;
            let b2 = (nibble >> 2) & 1 == 1;
            let b3 = (nibble >> 3) & 1 == 1;

            // Invalid if nibble > 9 (i.e., 10-15).
            let expected_invalid = nibble > 9;

            Circuit::scope(format!("digit_nibble_{nibble}"), || {
                let b1_circuit = Boolean::new(Mode::Private, b1);
                let b2_circuit = Boolean::new(Mode::Private, b2);
                let b3_circuit = Boolean::new(Mode::Private, b3);
                let result = is_invalid_digit_nibble::<CurrentEnvironment>(&b1_circuit, &b2_circuit, &b3_circuit);
                assert_eq!(
                    result.eject_value(),
                    expected_invalid,
                    "nibble={nibble}: expected is_invalid={expected_invalid}"
                );
            });
            assert!(Circuit::is_satisfied(), "Circuit should be satisfied for nibble={nibble}");
            Circuit::reset();
        }
    }

    #[test]
    fn test_is_invalid_upper_offset_exhaustive() {
        // Test all 32 offset values.
        // Valid offsets: 1-26 (A-Z), 31 (_). Invalid: 0, 27-30.
        for offset in 0u8..32 {
            let b0 = offset & 1 == 1;
            let b1 = (offset >> 1) & 1 == 1;
            let b2 = (offset >> 2) & 1 == 1;
            let b3 = (offset >> 3) & 1 == 1;
            let b4 = (offset >> 4) & 1 == 1;

            // Invalid offsets: 0, 27, 28, 29, 30.
            let expected_invalid = offset == 0 || (27..=30).contains(&offset);

            Circuit::scope(format!("upper_offset_{offset}"), || {
                let b0_c = Boolean::new(Mode::Private, b0);
                let b1_c = Boolean::new(Mode::Private, b1);
                let b2_c = Boolean::new(Mode::Private, b2);
                let b3_c = Boolean::new(Mode::Private, b3);
                let b4_c = Boolean::new(Mode::Private, b4);
                let data = ByteValidationData::new(&b0_c, &b1_c, &b2_c, &b3_c, &b4_c);
                let result = is_invalid_upper_offset::<CurrentEnvironment>(&data, &b2_c);
                assert_eq!(
                    result.eject_value(),
                    expected_invalid,
                    "offset={offset}: expected is_invalid={expected_invalid}"
                );
            });
            assert!(Circuit::is_satisfied(), "Circuit should be satisfied for offset={offset}");
            Circuit::reset();
        }
    }

    #[test]
    fn test_is_invalid_lower_offset_exhaustive() {
        // Test all 32 offset values.
        // Valid offsets: 1-26 (a-z). Invalid: 0, 27-31.
        for offset in 0u8..32 {
            let b0 = offset & 1 == 1;
            let b1 = (offset >> 1) & 1 == 1;
            let b2 = (offset >> 2) & 1 == 1;
            let b3 = (offset >> 3) & 1 == 1;
            let b4 = (offset >> 4) & 1 == 1;

            // Invalid offsets: 0, 27, 28, 29, 30, 31.
            let expected_invalid = offset == 0 || offset >= 27;

            Circuit::scope(format!("lower_offset_{offset}"), || {
                let b0_c = Boolean::new(Mode::Private, b0);
                let b1_c = Boolean::new(Mode::Private, b1);
                let b2_c = Boolean::new(Mode::Private, b2);
                let b3_c = Boolean::new(Mode::Private, b3);
                let b4_c = Boolean::new(Mode::Private, b4);
                let data = ByteValidationData::new(&b0_c, &b1_c, &b2_c, &b3_c, &b4_c);
                let result = is_invalid_lower_offset::<CurrentEnvironment>(&data, &b2_c);
                assert_eq!(
                    result.eject_value(),
                    expected_invalid,
                    "offset={offset}: expected is_invalid={expected_invalid}"
                );
            });
            assert!(Circuit::is_satisfied(), "Circuit should be satisfied for offset={offset}");
            Circuit::reset();
        }
    }
}
