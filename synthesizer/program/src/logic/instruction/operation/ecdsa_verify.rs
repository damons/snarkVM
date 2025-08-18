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

use crate::{HashVariant, Opcode, Operand, RegistersCircuit, RegistersTrait, StackTrait};
use console::{
    network::prelude::*,
    program::{Literal, LiteralType, PlaintextType, Register, RegisterType},
    types::Boolean,
};

/// The ECDSA signature verification instruction using Keccak256.
pub type ECDSAVerifyKeccak256<N> = ECDSAVerify<N, { HashVariant::HashKeccak256 as u8 }, false>;
/// The ECDSA signature verification instruction using Keccak256 with raw inputs.
pub type ECDSAVerifyKeccak256Raw<N> = ECDSAVerify<N, { HashVariant::HashKeccak256 as u8 }, true>;
/// The ECDSA signature verification instruction using Keccak384.
pub type ECDSAVerifyKeccak384<N> = ECDSAVerify<N, { HashVariant::HashKeccak384 as u8 }, false>;
/// The ECDSA signature verification instruction using Keccak384 with raw inputs.
pub type ECDSAVerifyKeccak384Raw<N> = ECDSAVerify<N, { HashVariant::HashKeccak384 as u8 }, true>;
/// The ECDSA signature verification instruction using Keccak512.
pub type ECDSAVerifyKeccak512<N> = ECDSAVerify<N, { HashVariant::HashKeccak512 as u8 }, false>;
/// The ECDSA signature verification instruction using Keccak512 with raw inputs.
pub type ECDSAVerifyKeccak512Raw<N> = ECDSAVerify<N, { HashVariant::HashKeccak512 as u8 }, true>;

/// The ECDSA signature verification instruction using SHA3-256.
pub type ECDSAVerifySha3_256<N> = ECDSAVerify<N, { HashVariant::HashSha3_256 as u8 }, false>;
/// The ECDSA signature verification instruction using SHA3-256 with raw inputs.
pub type ECDSAVerifySha3_256Raw<N> = ECDSAVerify<N, { HashVariant::HashSha3_256 as u8 }, true>;
/// The ECDSA signature verification instruction using SHA3-384.
pub type ECDSAVerifySha3_384<N> = ECDSAVerify<N, { HashVariant::HashSha3_384 as u8 }, false>;
/// The ECDSA signature verification instruction using SHA3-384 with raw inputs.
pub type ECDSAVerifySha3_384Raw<N> = ECDSAVerify<N, { HashVariant::HashSha3_384 as u8 }, true>;
/// The ECDSA signature verification instruction using SHA3-512.
pub type ECDSAVerifySha3_512<N> = ECDSAVerify<N, { HashVariant::HashSha3_512 as u8 }, false>;
/// The ECDSA signature verification instruction using SHA3-512 with raw inputs.
pub type ECDSAVerifySha3_512Raw<N> = ECDSAVerify<N, { HashVariant::HashSha3_512 as u8 }, true>;

/// Computes whether `signature` is valid for the given `address` and `message`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ECDSAVerify<N: Network, const VARIANT: u8, const RAW: bool> {
    /// The operands.
    operands: Vec<Operand<N>>,
    /// The destination register.
    destination: Register<N>,
}

impl<N: Network, const VARIANT: u8, const RAW: bool> ECDSAVerify<N, VARIANT, RAW> {
    /// Initializes a new `sign.verify` instruction.
    #[inline]
    pub fn new(operands: Vec<Operand<N>>, destination: Register<N>) -> Result<Self> {
        // Sanity check the number of operands.
        ensure!(operands.len() == 3, "Instruction '{}' must have three operands", Self::opcode());
        // Return the instruction.
        Ok(Self { operands, destination })
    }

    /// Returns the opcode.
    #[inline]
    pub const fn opcode() -> Opcode {
        // Helper macro to add the raw suffix to the opcode name if needed.
        macro_rules! op_name {
            ($base:literal, $raw:expr) => {
                if $raw { concat!($base, ".raw") } else { $base }
            };
        }

        // Determine the opcode name based on the variant and raw flag.
        let name = match VARIANT {
            4 => op_name!("ecdsa.verify.keccak256", RAW),
            5 => op_name!("ecdsa.verify.keccak384", RAW),
            6 => op_name!("ecdsa.verify.keccak512", RAW),
            12 => op_name!("ecdsa.verify.sha3_256", RAW),
            13 => op_name!("ecdsa.verify.sha3_384", RAW),
            14 => op_name!("ecdsa.verify.sha3_512", RAW),
            _ => panic!("Invalid 'ecdsa.verify' instruction opcode"),
        };
        Opcode::ECDSA(name)
    }

    /// Returns the operands in the operation.
    #[inline]
    pub fn operands(&self) -> &[Operand<N>] {
        // Sanity check that there are exactly three operands.
        debug_assert!(self.operands.len() == 3, "Instruction '{}' must have three operands", Self::opcode());
        // Return the operands.
        &self.operands
    }

    /// Returns the destination register.
    #[inline]
    pub fn destinations(&self) -> Vec<Register<N>> {
        vec![self.destination.clone()]
    }
}

impl<N: Network, const VARIANT: u8, const RAW: bool> ECDSAVerify<N, VARIANT, RAW> {
    /// Evaluates the instruction.
    #[inline]
    pub fn evaluate(&self, _stack: &impl StackTrait<N>, _registers: &mut impl RegistersTrait<N>) -> Result<()> {
        bail!("Instruction '{}' is currently only supported in finalize", Self::opcode());
    }

    /// Executes the instruction.
    #[inline]
    pub fn execute<A: circuit::Aleo<Network = N>>(
        &self,
        _stack: &impl StackTrait<N>,
        _registers: &mut impl RegistersCircuit<N, A>,
    ) -> Result<()> {
        bail!("Instruction '{}' is currently only supported in finalize", Self::opcode());
    }

    /// Finalizes the instruction.
    #[inline]
    pub fn finalize(&self, stack: &impl StackTrait<N>, registers: &mut impl RegistersTrait<N>) -> Result<()> {
        // // Ensure the number of operands is correct.
        // if self.operands.len() != 3 {
        //     bail!("Instruction '{}' expects 3 operands, found {} operands", Self::opcode(), self.operands.len())
        // }
        //
        // // Retrieve the inputs.
        // let signature = registers.load(stack, &self.operands[0])?;
        // let address = registers.load(stack, &self.operands[1])?;
        // let message = registers.load(stack, &self.operands[2])?;
        //
        // // Verify the signature.
        // let output = Literal::Boolean(Boolean::new(false));
        //
        // // Store the output.
        // registers.store_literal(stack, &self.destination, output)
        // TODO (raychu86): ECDSA - Implement the actual signature verification logic.
        unimplemented!()
    }

    /// Returns the output type from the given program and input types.
    #[inline]
    pub fn output_types(
        &self,
        _stack: &impl StackTrait<N>,
        input_types: &[RegisterType<N>],
    ) -> Result<Vec<RegisterType<N>>> {
        // Ensure the number of input types is correct.
        if input_types.len() != 3 {
            bail!("Instruction '{}' expects 3 inputs, found {} inputs", Self::opcode(), input_types.len())
        }

        // TODO (raychu86): Determine the proper operand types.
        // Ensure the first operand is a signature.
        // if input_types[0] != RegisterType::Plaintext(PlaintextType::Literal(LiteralType::Signature)) {
        //     bail!(
        //         "Instruction '{}' expects the first input to be a 'signature'. Found input of type '{}'",
        //         Self::opcode(),
        //         input_types[0]
        //     )
        // }

        // Ensure the second operand is an address.
        // if input_types[1] != RegisterType::Plaintext(PlaintextType::Literal(LiteralType::Address)) {
        //     bail!(
        //         "Instruction '{}' expects the second input to be an 'address'. Found input of type '{}'",
        //         Self::opcode(),
        //         input_types[1]
        //     )
        // }

        Ok(vec![RegisterType::Plaintext(PlaintextType::Literal(LiteralType::Boolean))])
    }
}

impl<N: Network, const VARIANT: u8, const RAW: bool> Parser for ECDSAVerify<N, VARIANT, RAW> {
    /// Parses a string into an operation.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        // Parse the opcode from the string.
        let (string, _) = tag(*Self::opcode())(string)?;
        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;
        // Parse the first operand from the string.
        let (string, first) = Operand::parse(string)?;
        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;
        // Parse the second operand from the string.
        let (string, second) = Operand::parse(string)?;
        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;
        // Parse the third operand from the string.
        let (string, third) = Operand::parse(string)?;
        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;
        // Parse the "into" from the string.
        let (string, _) = tag("into")(string)?;
        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;
        // Parse the destination register from the string.
        let (string, destination) = Register::parse(string)?;

        Ok((string, Self { operands: vec![first, second, third], destination }))
    }
}

impl<N: Network, const VARIANT: u8, const RAW: bool> FromStr for ECDSAVerify<N, VARIANT, RAW> {
    type Err = Error;

    /// Parses a string into an operation.
    #[inline]
    fn from_str(string: &str) -> Result<Self> {
        match Self::parse(string) {
            Ok((remainder, object)) => {
                // Ensure the remainder is empty.
                ensure!(remainder.is_empty(), "Failed to parse string. Found invalid character in: \"{remainder}\"");
                // Return the object.
                Ok(object)
            }
            Err(error) => bail!("Failed to parse string. {error}"),
        }
    }
}

impl<N: Network, const VARIANT: u8, const RAW: bool> Debug for ECDSAVerify<N, VARIANT, RAW> {
    /// Prints the operation as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network, const VARIANT: u8, const RAW: bool> Display for ECDSAVerify<N, VARIANT, RAW> {
    /// Prints the operation to a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Ensure the number of operands is 3.
        if self.operands.len() != 3 {
            return Err(fmt::Error);
        }
        // Print the operation.
        write!(f, "{} ", Self::opcode())?;
        self.operands.iter().try_for_each(|operand| write!(f, "{operand} "))?;
        write!(f, "into {}", self.destination)
    }
}

impl<N: Network, const VARIANT: u8, const RAW: bool> FromBytes for ECDSAVerify<N, VARIANT, RAW> {
    /// Reads the operation from a buffer.
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Initialize the vector for the operands.
        let mut operands = Vec::with_capacity(3);
        // Read the operands.
        for _ in 0..3 {
            operands.push(Operand::read_le(&mut reader)?);
        }
        // Read the destination register.
        let destination = Register::read_le(&mut reader)?;

        // Return the operation.
        Ok(Self { operands, destination })
    }
}

impl<N: Network, const VARIANT: u8, const RAW: bool> ToBytes for ECDSAVerify<N, VARIANT, RAW> {
    /// Writes the operation to a buffer.
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Ensure the number of operands is 3.
        if self.operands.len() != 3 {
            return Err(error(format!("The number of operands must be 3, found {}", self.operands.len())));
        }
        // Write the operands.
        self.operands.iter().try_for_each(|operand| operand.write_le(&mut writer))?;
        // Write the destination register.
        self.destination.write_le(&mut writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::network::MainnetV0;

    type CurrentNetwork = MainnetV0;

    #[test]
    fn test_parse() {
        let (string, is) =
            ECDSAVerifyKeccak256Raw::<CurrentNetwork>::parse("ecdsa.verify.keccak256.raw r0 r1 r2 into r3").unwrap();
        assert!(string.is_empty(), "Parser did not consume all of the string: '{string}'");
        assert_eq!(is.operands.len(), 3, "The number of operands is incorrect");
        assert_eq!(is.operands[0], Operand::Register(Register::Locator(0)), "The first operand is incorrect");
        assert_eq!(is.operands[1], Operand::Register(Register::Locator(1)), "The second operand is incorrect");
        assert_eq!(is.operands[2], Operand::Register(Register::Locator(2)), "The third operand is incorrect");
        assert_eq!(is.destination, Register::Locator(3), "The destination register is incorrect");
    }
}
