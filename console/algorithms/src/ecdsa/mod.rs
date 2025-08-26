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

#[cfg(test)]
pub mod tests;

mod serialize;

use super::*;
use snarkvm_utilities::bytes_from_bits_le;

use k256::{
    Secp256k1,
    ecdsa::{
        RecoveryId,
        Signature,
        SigningKey,
        VerifyingKey,
        signature::hazmat::{PrehashSigner, PrehashVerifier},
    },
    elliptic_curve::{Curve, generic_array::typenum::Unsigned},
};

/// An ECDSA/Secp256k1 signature (r,s) signature.
#[derive(Clone, PartialEq, Eq)]
pub struct ECDSASignature {
    signature: Signature,
    recovery_id: RecoveryId,
}

impl ECDSASignature {
    /// Returns a signature on a `message` using the given `signing_key` and hash function.
    pub fn sign<H: Hash<Output = Vec<bool>>>(
        signing_key: &SigningKey,
        hasher: &H,
        message: &[H::Input],
    ) -> Result<Self> {
        // Hash the message.
        let hash_bits = hasher.hash(message)?;
        // Convert the hash output to bytes.
        let hash_bytes = bytes_from_bits_le(&hash_bits);

        // Sign the prehashed message.
        signing_key
            .sign_prehash(&hash_bytes)
            .map(|(signature, recovery_id)| Self { signature, recovery_id })
            .map_err(|e| anyhow!("Failed to sign message: {e:?}"))
    }

    /// Recover the public key from `(r,s, recovery_id)` using *your* hasher on `message`.
    pub fn recover_public_key<H: Hash<Output = Vec<bool>>>(
        &self,
        hasher: &H,
        message: &[H::Input],
    ) -> Result<VerifyingKey> {
        // Hash the message.
        let hash_bits = hasher.hash(message)?;
        // Convert the hash output to bytes.
        let hash_bytes = bytes_from_bits_le(&hash_bits);

        // Recover the public key using the prehash.
        VerifyingKey::recover_from_prehash(&hash_bytes, &self.signature, self.recovery_id)
            .map_err(|e| anyhow!("Failed to recover public key: {e:?}"))
    }

    /// Verify `(r,s)` against `verifying_key` using *your* hasher on `message`.
    pub fn verify<H: Hash<Output = Vec<bool>>>(
        &self,
        verifying_key: &VerifyingKey,
        hasher: &H,
        message: &[H::Input],
    ) -> Result<()> {
        // Hash the message.
        let hash_bits = hasher.hash(message)?;
        // Convert the hash output to bytes.
        let hash_bytes = bytes_from_bits_le(&hash_bits);

        // Verify the signature using the prehash.
        verifying_key
            .verify_prehash(&hash_bytes, &self.signature)
            .map_err(|e| anyhow!("Failed to verify signature: {e:?}"))
    }

    /// Verify `(r,s)` against `verifying_key` using *your* hasher on `message`.
    pub fn verify_ethereum<H: Hash<Output = Vec<bool>>>(
        &self,
        ethereum_address: [u8; 20],
        hasher: &H,
        message: &[H::Input],
    ) -> Result<()> {
        // Derive the verifying key from the signature.
        let verifying_key = self.recover_public_key(hasher, message)?;

        // Ensure that the derived Ethereum address matches the provided one.
        let derived_ethereum_address = Self::ethereum_address_from_public_key(&verifying_key)?;
        ensure!(
            derived_ethereum_address == ethereum_address,
            "Derived Ethereum address does not match the provided address."
        );

        Ok(())
    }

    /// Converts a VerifyingKey to an Ethereum address (20 bytes).
    pub fn ethereum_address_from_public_key(verifying_key: &VerifyingKey) -> Result<[u8; 20]> {
        // Get the uncompressed public key bytes as [0x04, x_bytes..., y_bytes...]
        let public_key_point = verifying_key.to_encoded_point(false);
        let public_key_bytes = public_key_point.as_bytes();

        // Skip the 0x04 prefix, keep only the x and y coordinates (64 bytes)
        let coordinates_only = &public_key_bytes[1..]; // 32 bytes x + 32 bytes y

        // Step 3: Hash the coordinates with Keccak256
        let address_hash = Keccak256::default().hash(&coordinates_only.to_bits_le())?;
        let address_bytes = bytes_from_bits_le(&address_hash);

        // Step 4: Take the last 20 bytes as the Ethereum address
        let mut ethereum_address = [0u8; 20];
        ethereum_address.copy_from_slice(&address_bytes[12..32]);

        Ok(ethereum_address)
    }
}

impl ToBytes for ECDSASignature {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        let bytes = self.signature.to_bytes().to_vec();
        bytes.write_le(&mut writer)?;
        self.recovery_id.to_byte().write_le(&mut writer)
    }
}

impl FromBytes for ECDSASignature {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Declare the signature size in bytes for secp256k1.
        const SIGNATURE_SIZE_IN_BYTES: usize = <Secp256k1 as Curve>::FieldBytesSize::USIZE * 2;

        // Read the signature bytes.
        let mut bytes = vec![0u8; SIGNATURE_SIZE_IN_BYTES];
        reader.read_exact(&mut bytes)?;

        // Read the recovery ID byte.
        let recovery_id_byte = u8::read_le(&mut reader)?;

        // Construct the signature and recovery ID from the bytes.
        let signature = Signature::from_slice(&bytes).map_err(error)?;
        let recovery_id = RecoveryId::from_byte(recovery_id_byte)
            .ok_or_else(|| error(format!("Invalid recovery ID byte {recovery_id_byte}")))?;

        Ok(Self { signature, recovery_id })
    }
}

impl FromStr for ECDSASignature {
    type Err = Error;

    /// Parses a hex-encoded string into an ECDSASignature.
    fn from_str(signature: &str) -> Result<Self, Self::Err> {
        let mut s = signature.trim();

        // Accept optional 0x prefix
        if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            s = rest;
        }

        // Decode the hex string into bytes.
        let bytes = hex::decode(s)?;

        // Construct the signature from the bytes.
        Self::from_bytes_le(&bytes)
    }
}

impl Debug for ECDSASignature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for ECDSASignature {
    /// Writes the signature as a hex string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_bytes_le().map_err(|_| fmt::Error)?))
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    pub(crate) type DefaultHasher = Keccak256;

    /// Samples a random ecdsa signature.
    pub(super) fn sample_ecdsa_signature<H: Hash<Output = Vec<bool>, Input = bool>>(
        num_bytes: usize,
        hasher: &H,
        rng: &mut TestRng,
    ) -> (SigningKey, Vec<u8>, ECDSASignature) {
        // Sample a random signing key.
        let signing_key = SigningKey::random(rng);

        // Sample a random message.
        let message: Vec<u8> = (0..num_bytes).map(|_| rng.r#gen()).collect::<Vec<_>>();

        // Sign the message.
        let signature = ECDSASignature::sign::<H>(&signing_key, hasher, &message.to_bits_le()).unwrap();

        // Return the signing key, message, and signature.
        (signing_key, message, signature)
    }
}
