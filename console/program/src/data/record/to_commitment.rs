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
    /// Returns the record commitment.
    pub fn to_commitment(
        &self,
        program_id: &ProgramID<N>,
        record_name: &Identifier<N>,
        record_view_key: &Field<N>,
    ) -> Result<Field<N>> {
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
        let digest = N::hash_bhp1024(&record_bits);

        // If the record is non-hiding, then return the digest. Otherwise, return the commitment.
        match !self.is_hiding() {
            // Version 0 - Compute the BHP hash of the program record.
            true => digest,
            // Version 1 - Compute the BHP commitment of the program record.
            false => {
                // Construct the commitment nonce.
                let cm_nonce = N::hash_to_scalar_psd2(&[N::commitment_domain(), *record_view_key])?;
                // Compute the BHP commitment of the program record using the commitment nonce.
                N::commit_bhp256(&digest, &cm_nonce)
            }
        }
    }
}

impl<N: Network> Record<N, Ciphertext<N>> {
    /// Returns the record commitment.
    pub fn to_commitment(
        &self,
        _program_id: &ProgramID<N>,
        _record_name: &Identifier<N>,
        _record_view_key: &Field<N>,
    ) -> Result<Field<N>> {
        bail!("Illegal operation: Record::to_commitment() cannot be invoked on the `Ciphertext` variant.")
    }
}
