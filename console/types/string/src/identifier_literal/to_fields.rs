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
    fn to_fields(&self) -> Result<Vec<Self::Field>> {
        Ok(vec![self.to_field()?])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network_environment::Console;

    type CurrentEnvironment = Console;

    const ITERATIONS: u64 = 100;

    #[test]
    fn test_to_fields() -> Result<()> {
        let mut rng = TestRng::default();

        for _ in 0..ITERATIONS {
            // Sample a random identifier literal.
            let identifier = IdentifierLiteral::<CurrentEnvironment>::rand(&mut rng);

            // Convert to fields.
            let fields = identifier.to_fields()?;

            // Verify exactly one field element.
            assert_eq!(1, fields.len());

            // Verify the field value matches to_field.
            assert_eq!(identifier.to_field()?, fields[0]);
        }
        Ok(())
    }
}
