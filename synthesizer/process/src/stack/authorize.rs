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

impl<N: Network> Stack<N> {
    const PUBLIC_CREDITS_FUNCTION_NAMES: [&str; 8] = [
        "bond_validator",
        "bond_public",
        "unbond_public",
        "claim_unbond_public",
        "set_validator_state",
        "transfer_public",
        "transfer_public_as_signer",
        "fee_public",
    ];

    /// Authorizes a call to the program function for the given inputs.
    #[inline]
    pub fn authorize<A: circuit::Aleo<Network = N>, R: Rng + CryptoRng>(
        &self,
        private_key: &PrivateKey<N>,
        function_name: impl TryInto<Identifier<N>>,
        inputs: impl ExactSizeIterator<Item = impl TryInto<Value<N>>>,
        rng: &mut R,
    ) -> Result<Authorization<N>> {
        let timer = timer!("Stack::authorize");

        // Get the program ID.
        let program_id = *self.program.id();
        // Prepare the function name.
        let function_name = function_name.try_into().map_err(|_| anyhow!("Invalid function name"))?;
        // Retrieve the input types.
        let input_types = self.get_function(&function_name)?.input_types();
        lap!(timer, "Retrieve the input types");
        // Set is_root to true.
        let is_root = true;

        // This is the root request and does not have a caller.
        let caller = None;
        // This is the root request and we do not have a root_tvk to pass on.
        let root_tvk = None;
        // Compute the request.
        let request =
            Request::sign(private_key, program_id, function_name, inputs, &input_types, root_tvk, is_root, rng)?;
        lap!(timer, "Compute the request");
        // Initialize the authorization.
        let authorization = Authorization::new(request.clone());
        // Construct the call stack.
        let call_stack = CallStack::Authorize(vec![request], *private_key, authorization.clone());
        // Construct the authorization from the function.
        let _response = self.evaluate_function::<A, R>(call_stack, caller, root_tvk, rng)?;
        finish!(timer, "Construct the authorization from the function");

        // Return the authorization.
        Ok(authorization)
    }

    /// Authorizes a call to the program function for the given inputs.
    /// Compared to `authorize`, this method also checks for circuit satisfiability of the request.
    #[inline]
    pub fn authorize_checked<A: circuit::Aleo<Network = N>, R: Rng + CryptoRng>(
        &self,
        private_key: &PrivateKey<N>,
        function_name: impl TryInto<Identifier<N>>,
        inputs: impl ExactSizeIterator<Item = impl TryInto<Value<N>>>,
        rng: &mut R,
    ) -> Result<Authorization<N>> {
        let timer = timer!("Stack::authorize_checked");

        // Get the program ID.
        let program_id = *self.program.id();
        // Prepare the function name.
        let function_name = function_name.try_into().map_err(|_| anyhow!("Invalid function name"))?;
        // Retrieve the input types.
        let input_types = self.get_function(&function_name)?.input_types();
        lap!(timer, "Retrieve the input types");
        // Set is_root to true.
        let is_root = true;

        // This is the root request and does not have a caller.
        let caller = None;
        // This is the root request and we do not have a root_tvk to pass on.
        let root_tvk = None;
        // Compute the request.
        let request =
            Request::sign(private_key, program_id, function_name, inputs, &input_types, root_tvk, is_root, rng)?;
        lap!(timer, "Compute the request");
        // Initialize the authorization.
        let authorization = Authorization::new(request.clone());
        // Construct the call stack.
        let call_stack = CallStack::Authorize(vec![request], *private_key, authorization.clone());
        // Construct the authorization from the function.
        let _response = self.execute_function::<A, R>(call_stack, caller, root_tvk, rng)?;
        finish!(timer, "Construct the authorization from the function");

        // Return the authorization.
        Ok(authorization)
    }

    /// Authorizes calls to public credits.aleo functions for the given request.
    // TODO: we could generalize to any function without private inputs or outputs.
    #[inline]
    pub fn authorize_credits_public<A: circuit::Aleo<Network = N>, R: Rng + CryptoRng>(
        &self,
        private_key: &PrivateKey<N>,
        request: Request<N>,
        rng: &mut R,
    ) -> Result<Authorization<N>> {
        let timer = timer!("Stack::authorize_credits_public");

        // Get the program ID.
        let program_id = *self.program.id();
        // Ensure the program ID is credits.aleo.
        ensure!(program_id.to_string() == "credits.aleo", "Program ID must be credits.aleo");
        // Ensure the request is for a public function.
        let function_name = request.function_name();
        ensure!(
            Self::PUBLIC_CREDITS_FUNCTION_NAMES.contains(&function_name.to_string().as_str()),
            "Function name must be one of: {:?}",
            Self::PUBLIC_CREDITS_FUNCTION_NAMES
        );

        // Initialize the authorization.
        let authorization = Authorization::new(request.clone());
        // Construct the call stack.
        let call_stack = CallStack::Authorize(vec![request], *private_key, authorization.clone());
        // This is the root request and does not have a caller.
        let caller = None;
        // This is the root request and we do not have a root_tvk to pass on.
        let root_tvk = None;
        // Construct the authorization from the function.
        let _response = self.evaluate_function::<A, R>(call_stack, caller, root_tvk, rng)?;
        finish!(timer, "Construct the authorization from the function");

        // Return the authorization.
        Ok(authorization)
    }
}
