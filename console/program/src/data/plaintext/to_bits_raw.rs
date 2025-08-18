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

impl<N: Network> ToBitsRaw for Plaintext<N> {
    /// Returns this plaintext as a list of **little-endian** bits without variant bits.
    fn write_bits_raw_le(&self, vec: &mut Vec<bool>) {
        // Fetch the standard bit serialization with variant bits.
        let bits = self.to_bits_le();

        // Truncate the first two bits (variant bits).
        let bits = &bits[2..];

        // Extend the vector with the bits.
        vec.extend_from_slice(bits);
    }

    /// Returns this plaintext as a list of **big-endian** bits without variant bits.
    fn write_bits_raw_be(&self, vec: &mut Vec<bool>) {
        // Fetch the standard bit serialization with variant bits.
        let bits = self.to_bits_be();

        // Truncate the first two bits (variant bits).
        let bits = &bits[2..];

        // Extend the vector with the bits.
        vec.extend_from_slice(bits);
    }
}
