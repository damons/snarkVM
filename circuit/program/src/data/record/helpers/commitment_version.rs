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

use snarkvm_circuit_network::Aleo;
use snarkvm_circuit_types::{Boolean, U8, environment::prelude::*};

/// The commitment version circuit.
#[derive(Clone)]
pub struct CommitmentVersion<A: Aleo>(U8<A>);

#[cfg(feature = "console")]
impl<A: Aleo> Inject for CommitmentVersion<A> {
    type Primitive = console::CommitmentVersion;

    /// Initializes commitment version from a primitive.
    fn new(mode: Mode, commitment_version: Self::Primitive) -> Self {
        Self(U8::new(mode, console::U8::new(commitment_version as u8)))
    }
}

impl<A: Aleo> Deref for CommitmentVersion<A> {
    type Target = U8<A>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A: Aleo> ToBits for CommitmentVersion<A> {
    type Boolean = Boolean<A>;

    /// Returns `self` as a boolean vector in little-endian order.
    fn write_bits_le(&self, vec: &mut Vec<Boolean<A>>) {
        self.0.write_bits_le(vec);
    }

    /// Returns `self` as a boolean vector in big-endian order.
    fn write_bits_be(&self, vec: &mut Vec<Boolean<A>>) {
        self.0.write_bits_be(vec);
    }
}
