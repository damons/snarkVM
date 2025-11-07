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

use crate::{Opcode, Operand, RegistersCircuit, RegistersTrait, StackTrait};
use console::{
    network::prelude::*,
    program::{Literal, LiteralType, PlaintextType, Register, RegisterType, Value},
    types::Boolean,
};
use snarkvm_algorithms::snark::varuna::VarunaVersion;
use snarkvm_synthesizer_snark::{Proof, VerifyingKey};

/// Computes whether `proof` is valid for the given `verifying_key` and `public inputs`.
pub type SnarkVerify<N> = SnarkVerification<N>;

/// Computes whether `proof` is valid for the given `verifying_key` and `public inputs`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SnarkVerification<N: Network> {
    /// The operands.
    operands: Vec<Operand<N>>,
    /// The destination register.
    destination: Register<N>,
}

impl<N: Network> SnarkVerification<N> {
    /// Initializes a new `snark.verify` instruction.
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
        Opcode::Snark("snark.verify")
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

/// Evaluate a snark verification operation.
///
/// This allows running the verification without the machinery of stacks and registers.
/// This is necessary for the Leo interpreter.
pub fn evaluate_varuna_proof<N: Network>(
    verifying_key: &VerifyingKey<N>,
    _function_name: &str,
    varuna_version: VarunaVersion,
    inputs: &[N::Field],
    proof: &Proof<N>,
) -> Result<bool> {
    // Verify the proof.
    Ok(verifying_key.verify(_function_name, varuna_version, inputs, proof))
}

impl<N: Network> SnarkVerification<N> {
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
        // Ensure the number of operands is correct.
        if self.operands.len() != 3 {
            bail!("Instruction '{}' expects 3 operands, found {} operands", Self::opcode(), self.operands.len())
        }

        // Retrieve the inputs.
        let verifying_key = match registers.load(stack, &self.operands[0])? {
            Value::Plaintext(plaintext) => {
                // Get the plaintext as a byte array.
                let bytes = plaintext.as_byte_array()?;
                // Deserialize the verifying key.
                VerifyingKey::<N>::from_bytes_le(&bytes)?
            }
            _ => bail!("Expected the first operand to be a byte array."),
        };
        let inputs = match registers.load(stack, &self.operands[1])? {
            Value::Plaintext(plaintext) => plaintext.as_field_array()?.into_iter().map(|f| *f).collect::<Vec<_>>(),
            _ => bail!("Expected the second operand to be an array of fields."),
        };
        let proof = match registers.load(stack, &self.operands[2])? {
            Value::Plaintext(plaintext) => {
                // Get the plaintext as a byte array.
                let bytes = plaintext.as_byte_array()?;
                // Deserialize the proof.
                Proof::<N>::from_bytes_le(&bytes)?
            }
            _ => bail!("Expected the third operand to be a byte array."),
        };

        // Verify the signature.
        let _function_name = "snark.verify";
        let varuna_version = VarunaVersion::V2;
        let output = evaluate_varuna_proof(&verifying_key, _function_name, varuna_version, &inputs, &proof)?;
        let output = Literal::Boolean(Boolean::new(output));

        // Store the output.
        registers.store_literal(stack, &self.destination, output)
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

        // Enforce that the verifying key is an array of bytes.
        match &input_types[0] {
            RegisterType::Plaintext(PlaintextType::Array(array_type))
                if array_type.base_element_type() == &PlaintextType::Literal(LiteralType::U8) =>
            {
                // valid byte array
            }
            _ => bail!(
                "Instruction '{}' expects the first input to be a byte array. Found input of type '{}'",
                Self::opcode(),
                input_types[0]
            ),
        }

        // Ensure the second operand is an array of fields.
        match &input_types[1] {
            RegisterType::Plaintext(PlaintextType::Array(array_type))
                if array_type.base_element_type() == &PlaintextType::Literal(LiteralType::Field) =>
            {
                // valid byte array
            }
            _ => bail!(
                "Instruction '{}' expects the second input to be an array of fields. Found input of type '{}'",
                Self::opcode(),
                input_types[1]
            ),
        }

        // Ensure the third operand is an array of bytes.
        match &input_types[2] {
            RegisterType::Plaintext(PlaintextType::Array(array_type))
                if array_type.base_element_type() == &PlaintextType::Literal(LiteralType::U8) =>
            {
                // valid byte array
            }
            _ => bail!(
                "Instruction '{}' expects the third input to be a byte array. Found input of type '{}'",
                Self::opcode(),
                input_types[2]
            ),
        }

        Ok(vec![RegisterType::Plaintext(PlaintextType::Literal(LiteralType::Boolean))])
    }
}

impl<N: Network> Parser for SnarkVerification<N> {
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

impl<N: Network> FromStr for SnarkVerification<N> {
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

impl<N: Network> Debug for SnarkVerification<N> {
    /// Prints the operation as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for SnarkVerification<N> {
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

impl<N: Network> FromBytes for SnarkVerification<N> {
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

impl<N: Network> ToBytes for SnarkVerification<N> {
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
        let (string, is) = SnarkVerify::<CurrentNetwork>::parse("snark.verify r0 r1 r2 into r3").unwrap();
        assert!(string.is_empty(), "Parser did not consume all of the string: '{string}'");
        assert_eq!(is.operands.len(), 3, "The number of operands is incorrect");
        assert_eq!(is.operands[0], Operand::Register(Register::Locator(0)), "The first operand is incorrect");
        assert_eq!(is.operands[1], Operand::Register(Register::Locator(1)), "The second operand is incorrect");
        assert_eq!(is.operands[2], Operand::Register(Register::Locator(2)), "The third operand is incorrect");
        assert_eq!(is.destination, Register::Locator(3), "The destination register is incorrect");
    }
}
