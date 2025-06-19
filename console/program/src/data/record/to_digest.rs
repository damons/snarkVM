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

impl<N: Network> Record<N, Plaintext<N>> {
    /// Returns the record digest.
    pub fn to_digest(&self, program_id: &ProgramID<N>, record_name: &Identifier<N>) -> Result<Field<N>> {
        // Construct the input as `(program_id || record_name || record)`.
        let input = to_bits_le![program_id, record_name, self];

        // If the record is non-hiding, then remove the version bits & owner visibility bit (the last 9 bits)
        // to maintain backwards compatibility.
        let record_bits = match !self.is_hiding() {
            // Version 0 - Construct the input without the version bits or owner visibility bit.
            true => input[..input.len() - 9].to_vec(),
            // Version 1 - Construct the input with the version bits & owner visibility bit.
            false => input,
        };

        // Compute the BHP hash of the program record.
        N::hash_bhp1024(&record_bits)
    }
}

impl<N: Network> Record<N, Ciphertext<N>> {
    /// Returns the record digest.
    pub fn to_digest(&self, _program_id: &ProgramID<N>, _record_name: &Identifier<N>) -> Result<Field<N>> {
        bail!("Illegal operation: Record::to_digest() cannot be invoked on the `Ciphertext` variant.")
    }
}
