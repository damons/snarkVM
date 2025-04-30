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
use circuit::Eject;

#[derive(Clone, Debug)]
pub struct InclusionAssignment<N: Network> {
    pub(crate) state_path: StatePath<N>,
    commitment: Field<N>,
    gamma: Group<N>,
    serial_number: Field<N>,
    check_record_index: bool,
    migration_record_index: u64, // TODO (raychu86): Updated Inclusion - Rename this.
    local_state_root: N::TransactionID,
    is_global: bool,
}

impl<N: Network> InclusionAssignment<N> {
    /// Initializes a new inclusion assignment.
    pub fn new(
        state_path: StatePath<N>,
        commitment: Field<N>,
        gamma: Group<N>,
        serial_number: Field<N>,
        check_record_index: bool,
        migration_record_index: u64,
        local_state_root: N::TransactionID,
        is_global: bool,
    ) -> Self {
        Self {
            state_path,
            commitment,
            gamma,
            serial_number,
            check_record_index,
            migration_record_index,
            local_state_root,
            is_global,
        }
    }

    /// The circuit for state path verification.
    ///
    /// # Diagram
    /// The `[[ ]]` notation is used to denote public inputs.
    /// ```ignore
    ///             [[ global_state_root ]] || [[ local_state_root ]]
    ///                        |                          |
    ///                        -------- is_global --------
    ///                                     |
    ///                                state_path
    ///                                    |
    /// [[ serial_number ]] := Commit( commitment || Hash( COFACTOR * gamma ) )
    /// ```
    pub fn to_circuit_assignment<A: circuit::Aleo<Network = N>>(&self) -> Result<circuit::Assignment<N::Field>> {
        use circuit::Inject;

        // Ensure the circuit environment is clean.
        assert_eq!(A::count(), (0, 1, 0, 0, (0, 0, 0)));
        A::reset();

        // Inject the state path as `Mode::Private` (with a global state root as `Mode::Public`).
        let state_path = circuit::StatePath::<A>::new(circuit::Mode::Private, self.state_path.clone());
        // Inject the commitment as `Mode::Private`.
        let commitment = circuit::Field::<A>::new(circuit::Mode::Private, self.commitment);
        // Inject the gamma as `Mode::Private`.
        let gamma = circuit::Group::<A>::new(circuit::Mode::Private, self.gamma);
        // Inject the local state root as `Mode::Public`.
        let local_state_root = circuit::Field::<A>::new(circuit::Mode::Public, *self.local_state_root);
        // Inject the 'is_global' flag as `Mode::Private`.
        let is_global = circuit::Boolean::<A>::new(circuit::Mode::Private, self.is_global);
        // Inject the serial number as `Mode::Public`.
        let serial_number = circuit::Field::<A>::new(circuit::Mode::Public, self.serial_number);
        // Inject the check_record_index as `Mode::Public`.
        let check_record_index = circuit::Boolean::<A>::new(circuit::Mode::Public, self.check_record_index);
        // Inject the migration_record_index as `Mode::Public`.
        // This is cast into a u64 to prevent requiring 64 field elements as input.
        let migration_record_index_field = circuit::Field::<A>::new(
            circuit::Mode::Public,
            console::types::Field::<N>::from_u64(self.migration_record_index),
        );
        let migration_record_index = circuit::U64::from_field_lossy(&migration_record_index_field);

        // Compute the candidate serial number.
        let candidate_serial_number =
            circuit::Record::<A, circuit::Plaintext<A>>::serial_number_from_gamma(&gamma, commitment.clone());
        // Enforce that the candidate serial number is equal to the serial number.
        A::assert_eq(candidate_serial_number, serial_number);

        // Enforce the starting leaf is the claimed commitment.
        A::assert_eq(state_path.transition_leaf().id(), commitment);
        // Enforce the state path from leaf to root is correct.
        A::assert(state_path.verify(&is_global, &local_state_root));

        // Fetch the record index from the state path.
        let record_index = state_path.record_index();

        // Determine if the record index is past migration.
        let is_record_index_past_migration = record_index.is_greater_than_or_equal(&migration_record_index);

        // TODO (raychu86): Updated Inclusion - Implement the enforcement that upgrades can only be called on records before the `migration_record_index`.
        // Enforce the record index if `check_record_index` is set.
        let accept = circuit::Boolean::<A>::new(circuit::Mode::Private, true);
        let is_valid_index = circuit::Boolean::ternary(&check_record_index, &is_record_index_past_migration, &accept);
        A::assert(is_valid_index);

        #[cfg(debug_assertions)]
        Stack::log_circuit::<A, _>(&format!("State Path for {}", self.serial_number));

        // Eject the assignment and reset the circuit environment.
        Ok(A::eject_assignment_and_reset())
    }
}
