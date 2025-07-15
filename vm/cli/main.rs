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

use snarkvm::{
    cli::{CLI, Updater},
    utilities::display_error,
};

use clap::Parser;

use std::panic::catch_unwind;

fn main() {
    // Parse the given arguments.
    let cli = CLI::parse();
    // Run the updater.
    if let Some(msg) = Updater::print_cli() {
        println!("{msg}");
    }

    // Set a custom hook here to show "pretty" errors when panicking.
    std::panic::set_hook(Box::new(|err| {
        eprintln!("⚠️ {}", err.to_string().replace("panicked at", "snarkVM encountered an unexpected error at"));
    }));

    // Run the CLI.
    // We use `catch_unwind` here to ensure a panic stops execution and not just a single thread.
    // Note: `catch_unwind` can be nested without problems.
    let result = catch_unwind(|| cli.command.parse());

    // Process any errors (including panics).
    match result {
        Ok(Ok(output)) => {
            println!("{output}\n");
            std::process::exit(0);
        }
        Ok(Err(err)) => {
            // A regular error occurred.
            display_error(&err);
            eprintln!();
            eprintln!("Use `--help` for instructions on how to use this command");
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!();
            eprintln!("This is most likely a bug!");
            eprintln!(
                "Please report it to the snarkVM developers: https://github.com/ProvableHQ/snarkVM/issues/new?template=bug.md"
            );
            std::process::exit(1);
        }
    }
}
