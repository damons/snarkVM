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

use crate::{QueryTrait, StaticQuery};
use console::{
    network::prelude::*,
    program::{ProgramID, StatePath},
    types::Field,
};
use snarkvm_ledger_store::{BlockStorage, BlockStore};
use snarkvm_synthesizer_program::Program;

use anyhow::{Context, Result};
// ureq re-exports the `http` crate.
use ureq::http::{self, uri};

#[derive(Clone)]
pub enum Query<N: Network, B: BlockStorage<N>> {
    /// The block store from the VM.
    VM(BlockStore<N, B>),
    /// The base URL of the node.
    REST(http::Uri),
    /// The local state to query.
    STATIC(StaticQuery<N>),
}

impl<N: Network, B: BlockStorage<N>> From<BlockStore<N, B>> for Query<N, B> {
    fn from(block_store: BlockStore<N, B>) -> Self {
        Self::VM(block_store)
    }
}

impl<N: Network, B: BlockStorage<N>> From<&BlockStore<N, B>> for Query<N, B> {
    fn from(block_store: &BlockStore<N, B>) -> Self {
        Self::VM(block_store.clone())
    }
}

impl<N: Network, B: BlockStorage<N>> TryFrom<String> for Query<N, B> {
    type Error = anyhow::Error;

    fn try_from(string_representation: String) -> Result<Self> {
        Self::try_from(string_representation.as_str())
    }
}

impl<N: Network, B: BlockStorage<N>> TryFrom<&String> for Query<N, B> {
    type Error = anyhow::Error;

    fn try_from(string_representation: &String) -> Result<Self> {
        Self::try_from(string_representation.as_str())
    }
}

impl<N: Network, B: BlockStorage<N>> TryFrom<&str> for Query<N, B> {
    type Error = anyhow::Error;

    fn try_from(str_representation: &str) -> Result<Self> {
        str_representation.parse::<Self>()
    }
}

impl<N: Network, B: BlockStorage<N>> FromStr for Query<N, B> {
    type Err = anyhow::Error;

    fn from_str(str_representation: &str) -> Result<Self> {
        // A static query is represented as JSON and a valid URI does not start with `}`.
        if str_representation.trim().starts_with('{') {
            let static_query =
                str_representation.parse::<StaticQuery<N>>().with_context(|| "Failed to parse static query")?;
            Ok(Self::STATIC(static_query))
        } else {
            let uri = str_representation.parse::<http::Uri>().with_context(|| "Failed to parse URL")?;

            if let Some(scheme) = uri.scheme()
                && *scheme != uri::Scheme::HTTP
                && *scheme != uri::Scheme::HTTPS
            {
                bail!("Invalid scheme in URL: {scheme}");
            }

            if let Some(s) = uri.host()
                && s.is_empty()
            {
                bail!("Invalid URL. Empty hostname given.");
            } else if uri.host().is_none() {
                bail!("Invalid URL. No hostname given.");
            }

            if uri.query().is_some() {
                bail!("Query URL cannot contain a query");
            }

            Ok(Self::REST(uri))
        }
    }
}

#[cfg_attr(feature = "async", async_trait(?Send))]
impl<N: Network, B: BlockStorage<N>> QueryTrait<N> for Query<N, B> {
    /// Returns the current state root.
    fn current_state_root(&self) -> Result<N::StateRoot> {
        match self {
            Self::VM(block_store) => Ok(block_store.current_state_root()),
            Self::REST(url) => {
                Ok(Self::get_request(&format!("{url}{}/stateRoot/latest", N::SHORT_NAME))?.body_mut().read_json()?)
            }
            Self::STATIC(query) => query.current_state_root(),
        }
    }

    /// Returns the current state root.
    #[cfg(feature = "async")]
    async fn current_state_root_async(&self) -> Result<N::StateRoot> {
        match self {
            Self::VM(block_store) => Ok(block_store.current_state_root()),
            Self::REST(url) => {
                Ok(Self::get_request_async(&format!("{url}{}/stateRoot/latest", N::SHORT_NAME)).await?.json().await?)
            }
            Self::STATIC(_query) => bail!("Async calls are not supported by StaticQuery"),
        }
    }

    /// Returns a state path for the given `commitment`.
    fn get_state_path_for_commitment(&self, commitment: &Field<N>) -> Result<StatePath<N>> {
        match self {
            Self::VM(block_store) => block_store.get_state_path_for_commitment(commitment),
            Self::REST(url) => Ok(Self::get_request(&format!("{url}{}/statePath/{commitment}", N::SHORT_NAME))?
                .body_mut()
                .read_json()?),
            Self::STATIC(query) => query.get_state_path_for_commitment(commitment),
        }
    }

    /// Returns a state path for the given `commitment`.
    #[cfg(feature = "async")]
    async fn get_state_path_for_commitment_async(&self, commitment: &Field<N>) -> Result<StatePath<N>> {
        match self {
            Self::VM(block_store) => block_store.get_state_path_for_commitment(commitment),
            Self::REST(url) => Ok(Self::get_request_async(&format!("{url}{}/statePath/{commitment}", N::SHORT_NAME))
                .await?
                .json()
                .await?),
            Self::STATIC(_query) => bail!("Async calls are not supported by StaticQuery"),
        }
    }

    /// Returns a state path for the given `commitment`.
    fn current_block_height(&self) -> Result<u32> {
        match self {
            Self::VM(block_store) => Ok(block_store.max_height().unwrap_or_default()),
            Self::REST(url) => {
                Ok(Self::get_request(&format!("{url}{}/block/height/latest", N::SHORT_NAME))?.body_mut().read_json()?)
            }
            Self::STATIC(query) => query.current_block_height(),
        }
    }

    /// Returns a state path for the given `commitment`.
    #[cfg(feature = "async")]
    async fn current_block_height_async(&self) -> Result<u32> {
        match self {
            Self::VM(block_store) => Ok(block_store.max_height().unwrap_or_default()),
            Self::REST(url) => Ok(Self::get_request_async(&format!("{url}{}/block/height/latest", N::SHORT_NAME))
                .await?
                .json()
                .await?),
            Self::STATIC(_query) => bail!("Async calls are not supported by StaticQuery"),
        }
    }
}

impl<N: Network, B: BlockStorage<N>> Query<N, B> {
    /// Returns the program for the given program ID.
    pub fn get_program(&self, program_id: &ProgramID<N>) -> Result<Program<N>> {
        match self {
            Self::VM(block_store) => block_store
                .get_latest_program(program_id)?
                .ok_or_else(|| anyhow!("Program {program_id} not found in storage")),
            Self::REST(url) => Ok(Self::get_request(&format!("{url}{}/program/{program_id}", N::SHORT_NAME))?
                .body_mut()
                .read_json()?),
            Self::STATIC(_query) => unimplemented!("get_program is not supported by StaticQuery"),
        }
    }

    /// Returns the program for the given program ID.
    #[cfg(feature = "async")]
    pub async fn get_program_async(&self, program_id: &ProgramID<N>) -> Result<Program<N>> {
        match self {
            Self::VM(block_store) => block_store
                .get_latest_program(program_id)?
                .ok_or_else(|| anyhow!("Program {program_id} not found in storage")),
            Self::REST(url) => Ok(Self::get_request_async(&format!("{url}{}/program/{program_id}", N::SHORT_NAME))
                .await?
                .json()
                .await?),
            Self::STATIC(_query) => unimplemented!("get_program_async is not supported by StaticQuery"),
        }
    }

    /// Performs a GET request to the given URL.
    fn get_request(url: &str) -> Result<http::Response<ureq::Body>> {
        let response = ureq::get(url).call()?;
        if response.status() == http::StatusCode::OK { Ok(response) } else { bail!("Failed to fetch from {url}") }
    }

    /// Performs a GET request to the given URL.
    #[cfg(feature = "async")]
    async fn get_request_async(url: &str) -> Result<reqwest::Response> {
        let response = reqwest::get(url).await?;
        if response.status() == http::StatusCode::OK { Ok(response) } else { bail!("Failed to fetch from {url}") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::network::TestnetV0;

    use snarkvm_ledger_store::helpers::memory::BlockMemory;

    type CurrentNetwork = TestnetV0;
    type CurrentQuery = Query<CurrentNetwork, BlockMemory<CurrentNetwork>>;

    #[test]
    fn test_static_query_parse() {
        let json = r#"{"state_root": "sr1dz06ur5spdgzkguh4pr42mvft6u3nwsg5drh9rdja9v8jpcz3czsls9geg", "height": 14}"#
            .to_string();
        let query = CurrentQuery::try_from(json).unwrap();

        assert!(matches!(query, Query::STATIC(_)));
    }

    #[test]
    fn test_static_query_parse_invalid() {
        let json = r#"{"invalid_key": "sr1dz06ur5spdgzkguh4pr42mvft6u3nwsg5drh9rdja9v8jpcz3czsls9geg", "height": 14}"#
            .to_string();
        let result = json.parse::<CurrentQuery>();

        assert!(result.is_err());
    }

    #[test]
    fn test_rest_url_parse() {
        let str = "http://localhost:3030";
        let query = str.parse::<CurrentQuery>().unwrap();

        assert!(matches!(query, Query::REST(_)));
    }

    #[test]
    fn test_rest_url_parse_invalid_scheme() {
        let str = "ftp://localhost:3030";
        let result = CurrentQuery::try_from(str);

        assert!(result.is_err());
    }

    #[test]
    fn test_rest_url_parse_invalid_host() {
        let str = "http://:3030";
        let result = CurrentQuery::try_from(str);

        assert!(result.is_err());
    }
}
