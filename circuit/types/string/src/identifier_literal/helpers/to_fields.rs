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

impl<E: Environment> ToFields for IdentifierLiteral<E> {
    type Field = Field<E>;

    /// Converts the identifier literal to a list of field elements.
    fn to_fields(&self) -> Vec<Self::Field> {
        vec![self.to_field()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::ToField as _;
    use snarkvm_circuit_environment::Circuit;

    type CurrentEnvironment = Circuit;

    /// Test strings covering various identifier patterns.
    const TEST_STRINGS: &[&str] = &["a", "hello", "hello_world", "Test123", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcde"];

    fn check_to_fields(mode: Mode, num_constants: u64, num_public: u64, num_private: u64, num_constraints: u64) {
        for string in TEST_STRINGS {
            // Construct a console identifier literal.
            let console_value =
                console::IdentifierLiteral::<<CurrentEnvironment as Environment>::Network>::new(string).unwrap();

            // Inject the identifier literal.
            let candidate = IdentifierLiteral::<CurrentEnvironment>::new(mode, console_value);

            Circuit::scope(format!("to_fields {mode}"), || {
                // Perform the conversion.
                let fields = candidate.to_fields();

                // Verify exactly one field element.
                assert_eq!(1, fields.len());

                // Verify the field value matches.
                let expected_field = console_value.to_field().unwrap();
                assert_eq!(expected_field, fields[0].eject_value());

                // ToFields is pure bit packing, no additional constraints.
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });

            Circuit::reset();
        }
    }

    #[test]
    fn test_to_fields_constant() {
        // Pure bit packing, no constraints.
        check_to_fields(Mode::Constant, 0, 0, 0, 0);
    }

    #[test]
    fn test_to_fields_public() {
        // Pure bit packing, no constraints.
        check_to_fields(Mode::Public, 0, 0, 0, 0);
    }

    #[test]
    fn test_to_fields_private() {
        // Pure bit packing, no constraints.
        check_to_fields(Mode::Private, 0, 0, 0, 0);
    }
}
