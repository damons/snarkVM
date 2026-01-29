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

impl<E: Environment> Cast<Address<E>> for IdentifierLiteral<E> {
    /// Casts an `IdentifierLiteral` to an `Address`.
    #[inline]
    fn cast(&self) -> Address<E> {
        self.to_field().cast()
    }
}

impl<E: Environment> Cast<Boolean<E>> for IdentifierLiteral<E> {
    /// Casts an `IdentifierLiteral` to a `Boolean`.
    #[inline]
    fn cast(&self) -> Boolean<E> {
        self.to_field().cast()
    }
}

impl<E: Environment> Cast<Field<E>> for IdentifierLiteral<E> {
    /// Casts an `IdentifierLiteral` to a `Field`.
    #[inline]
    fn cast(&self) -> Field<E> {
        self.to_field()
    }
}

impl<E: Environment> Cast<Group<E>> for IdentifierLiteral<E> {
    /// Casts an `IdentifierLiteral` to a `Group`.
    #[inline]
    fn cast(&self) -> Group<E> {
        self.to_field().cast()
    }
}

impl<E: Environment, I: IntegerType> Cast<Integer<E, I>> for IdentifierLiteral<E> {
    /// Casts an `IdentifierLiteral` to an `Integer`.
    #[inline]
    fn cast(&self) -> Integer<E, I> {
        self.to_field().cast()
    }
}

impl<E: Environment> Cast<Scalar<E>> for IdentifierLiteral<E> {
    /// Casts an `IdentifierLiteral` to a `Scalar`.
    #[inline]
    fn cast(&self) -> Scalar<E> {
        self.to_field().cast()
    }
}

impl<E: Environment> Cast<IdentifierLiteral<E>> for IdentifierLiteral<E> {
    /// Casts an `IdentifierLiteral` to itself.
    #[inline]
    fn cast(&self) -> IdentifierLiteral<E> {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::ToField as _;
    use console_root::network::MainnetV0;
    use snarkvm_circuit_types::environment::{Circuit, Eject, Inject, Mode, UpdatableCount, count_is};

    type CurrentEnvironment = Circuit;

    /// Test strings covering various identifier patterns.
    const TEST_STRINGS: &[&str] = &["a", "hello", "hello_world", "Test123", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcde"];

    fn check_cast_to_field(mode: Mode, count: UpdatableCount) {
        for string in TEST_STRINGS {
            // Construct a console identifier literal.
            let console_value = console_root::types::IdentifierLiteral::<MainnetV0>::new(string).unwrap();
            let circuit_value = IdentifierLiteral::<CurrentEnvironment>::new(mode, console_value);

            Circuit::scope("test", || {
                // Perform the cast.
                let result: Field<CurrentEnvironment> = circuit_value.cast();

                // Verify the result matches.
                let expected = console_value.to_field().unwrap();
                assert_eq!(result.eject_value(), expected);
                assert!(Circuit::is_satisfied());

                count.assert_matches(
                    Circuit::num_constants_in_scope(),
                    Circuit::num_public_in_scope(),
                    Circuit::num_private_in_scope(),
                    Circuit::num_constraints_in_scope(),
                );
            });
            Circuit::reset();
        }
    }

    fn check_cast_to_self(mode: Mode, count: UpdatableCount) {
        for string in TEST_STRINGS {
            // Construct a console identifier literal.
            let console_value = console_root::types::IdentifierLiteral::<MainnetV0>::new(string).unwrap();
            let circuit_value = IdentifierLiteral::<CurrentEnvironment>::new(mode, console_value);

            Circuit::scope("test", || {
                // Perform the cast.
                let result: IdentifierLiteral<CurrentEnvironment> = circuit_value.cast();

                // Verify the result matches.
                assert_eq!(result.eject_value(), console_value);
                assert!(Circuit::is_satisfied());

                count.assert_matches(
                    Circuit::num_constants_in_scope(),
                    Circuit::num_public_in_scope(),
                    Circuit::num_private_in_scope(),
                    Circuit::num_constraints_in_scope(),
                );
            });
            Circuit::reset();
        }
    }

    #[test]
    fn test_identifier_to_field_constant() {
        check_cast_to_field(Mode::Constant, count_is!(0, 0, 0, 0));
    }

    #[test]
    fn test_identifier_to_field_public() {
        check_cast_to_field(Mode::Public, count_is!(0, 0, 0, 0));
    }

    #[test]
    fn test_identifier_to_field_private() {
        check_cast_to_field(Mode::Private, count_is!(0, 0, 0, 0));
    }

    #[test]
    fn test_identifier_to_self_constant() {
        check_cast_to_self(Mode::Constant, count_is!(0, 0, 0, 0));
    }

    #[test]
    fn test_identifier_to_self_public() {
        check_cast_to_self(Mode::Public, count_is!(0, 0, 0, 0));
    }

    #[test]
    fn test_identifier_to_self_private() {
        check_cast_to_self(Mode::Private, count_is!(0, 0, 0, 0));
    }
}
