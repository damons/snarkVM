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

impl<N: Network> ToBits for Record<N, Plaintext<N>> {
    /// Returns this data as a list of **little-endian** bits.
    fn write_bits_le(&self, vec: &mut Vec<bool>) {
        // Construct the hiding bit.
        // Note: While this bitflag is redundant, it is necessary for backwards compatibility.
        vec.push(self.is_hiding());

        // Construct the owner bits.
        match &self.owner {
            Owner::Public(public) => public.write_bits_le(vec),
            Owner::Private(Plaintext::Literal(Literal::Address(address), ..)) => address.write_bits_le(vec),
            _ => N::halt("Internal error: plaintext to_bits_le corrupted in record owner"),
        };

        // Compute the data bits.
        let mut data_bits_le = vec![];
        for (identifier, entry) in &self.data {
            identifier.write_bits_le(&mut data_bits_le);
            entry.write_bits_le(&mut data_bits_le);
        }
        // Construct the data bits.
        u32::try_from(data_bits_le.len()).or_halt_with::<N>("Record data exceeds u32::MAX bits").write_bits_le(vec);
        vec.extend_from_slice(&data_bits_le);

        // Construct the nonce bits.
        self.nonce.write_bits_le(vec);

        // Construct the version bits.
        self.version.write_bits_le(vec);

        // Construct the owner visibility bit.
        match &self.owner {
            Owner::Public(_) => vec.push(true),   // Public owner
            Owner::Private(_) => vec.push(false), // Private owner
        }
    }

    /// Returns this data as a list of **big-endian** bits.
    fn write_bits_be(&self, vec: &mut Vec<bool>) {
        // Construct the hiding bit.
        // Note: While this bitflag is redundant, it is necessary for backwards compatibility.
        vec.push(self.is_hiding());

        // Construct the owner bits.
        match &self.owner {
            Owner::Public(public) => public.write_bits_be(vec),
            Owner::Private(Plaintext::Literal(Literal::Address(address), ..)) => address.write_bits_be(vec),
            _ => N::halt("Internal error: plaintext to_bits_be corrupted in record owner"),
        };

        // Compute the data bits.
        let mut data_bits_be = vec![];
        for (identifier, entry) in &self.data {
            identifier.write_bits_be(&mut data_bits_be);
            entry.write_bits_be(&mut data_bits_be);
        }
        // Construct the data bits.
        u32::try_from(data_bits_be.len()).or_halt_with::<N>("Record data exceeds u32::MAX bits").write_bits_be(vec);
        vec.extend_from_slice(&data_bits_be);

        // Construct the nonce bits.
        self.nonce.write_bits_be(vec);

        // Construct the version bits.
        self.version.write_bits_be(vec);

        // Construct the owner visibility bit.
        match &self.owner {
            Owner::Public(_) => vec.push(true),   // Public owner
            Owner::Private(_) => vec.push(false), // Private owner
        }
    }
}

impl<N: Network> ToBits for Record<N, Ciphertext<N>> {
    /// Returns this data as a list of **little-endian** bits.
    fn write_bits_le(&self, vec: &mut Vec<bool>) {
        // Construct the hiding bit.
        // Note: While this bitflag is redundant, it is necessary for backwards compatibility.
        vec.push(self.is_hiding());

        // Construct the owner bits.
        match &self.owner {
            Owner::Public(public) => public.write_bits_le(vec),
            Owner::Private(ciphertext) => {
                // Ensure there is exactly one field element in the ciphertext.
                match ciphertext.len() == 1 {
                    true => ciphertext[0].write_bits_le(vec),
                    false => N::halt("Internal error: ciphertext to_bits_le corrupted in record owner"),
                }
            }
        };

        // Compute the data bits.
        let mut data_bits_le = vec![];
        for (identifier, entry) in &self.data {
            identifier.write_bits_le(&mut data_bits_le);
            entry.write_bits_le(&mut data_bits_le);
        }
        // Construct the data bits.
        u32::try_from(data_bits_le.len()).or_halt_with::<N>("Record data exceeds u32::MAX bits").write_bits_le(vec);
        vec.extend_from_slice(&data_bits_le);

        // Construct the nonce bits.
        self.nonce.write_bits_le(vec);

        // Construct the version bits.
        self.version.write_bits_le(vec);

        // Construct the owner visibility bit.
        match &self.owner {
            Owner::Public(_) => vec.push(true),   // Public owner
            Owner::Private(_) => vec.push(false), // Private owner
        }
    }

    /// Returns this data as a list of **big-endian** bits.
    fn write_bits_be(&self, vec: &mut Vec<bool>) {
        // Construct the hiding bit.
        // Note: While this bitflag is redundant, it is necessary for backwards compatibility.
        vec.push(self.is_hiding());

        // Construct the owner bits.
        match &self.owner {
            Owner::Public(public) => public.write_bits_be(vec),
            Owner::Private(ciphertext) => {
                // Ensure there is exactly one field element in the ciphertext.
                match ciphertext.len() == 1 {
                    true => ciphertext[0].write_bits_be(vec),
                    false => N::halt("Internal error: ciphertext to_bits_be corrupted in record owner"),
                }
            }
        };

        // Compute the data bits.
        let mut data_bits_be = vec![];
        for (identifier, entry) in &self.data {
            identifier.write_bits_be(&mut data_bits_be);
            entry.write_bits_be(&mut data_bits_be);
        }
        // Construct the data bits.
        u32::try_from(data_bits_be.len()).or_halt_with::<N>("Record data exceeds u32::MAX bits").write_bits_be(vec);
        vec.extend_from_slice(&data_bits_be);

        // Construct the nonce bits.
        self.nonce.write_bits_be(vec);

        // Construct the version bits.
        self.version.write_bits_be(vec);

        // Construct the owner visibility bit.
        match &self.owner {
            Owner::Public(_) => vec.push(true),   // Public owner
            Owner::Private(_) => vec.push(false), // Private owner
        }
    }
}
