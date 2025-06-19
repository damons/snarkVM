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

use super::*;

impl<A: Aleo> Record<A, Plaintext<A>> {
    /// Returns the record digest.
    pub fn to_digest(&self, program_id: &ProgramID<A>, record_name: &Identifier<A>) -> Field<A> {
        // Construct the input as `(program_id || record_name || record)`.
        let mut input = program_id.to_bits_le();
        record_name.write_bits_le(&mut input);
        self.write_bits_le(&mut input);

        // Version 0 - Construct the input without the version bits or owner visibility bit.
        let input_v0 = input[..input.len() - 9].to_vec();
        // Version 1 - Construct the input with the version bits & owner visibility bit.
        let input_v1 = input;

        // If the record is non-hiding, then remove the version bits & owner visibility bit (the last 9 bits)
        // to maintain backwards compatibility.
        let record_bits = Ternary::ternary(&!self.is_hiding(), &input_v0, &input_v1);

        // Compute the BHP hash of the program record.
        A::hash_bhp1024(&record_bits)
    }
}

impl<A: Aleo> Record<A, Ciphertext<A>> {
    /// Returns the record digest.
    pub fn to_digest(&self, _program_id: &ProgramID<A>, _record_name: &Identifier<A>) -> Field<A> {
        A::halt("Illegal operation: Record::to_digest() cannot be invoked on the `Ciphertext` variant.")
    }
}
