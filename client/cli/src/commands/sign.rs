// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or 
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Implementation of the `sign` subcommand
use crate::{error, pair_from_suri, CliConfiguration, KeystoreParams, with_crypto_scheme, CryptoSchemeFlag};
use super::{SharedParams, read_message, read_uri};
use structopt::StructOpt;

/// The `sign` command
#[derive(Debug, StructOpt, Clone)]
#[structopt(
	name = "sign",
	about = "Sign a message, with a given (secret) key"
)]
pub struct SignCmd {
	/// The secret key URI.
	/// If the value is a file, the file content is used as URI.
	/// If not given, you will be prompted for the URI.
	#[structopt(long)]
	suri: Option<String>,

	/// Message to sign, if not provided you will be prompted to
	/// pass the message via STDIN
	#[structopt(long)]
	message: Option<String>,

	/// The message on STDIN is hex-encoded data
	#[structopt(long)]
	hex: bool,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub keystore_params: KeystoreParams,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub shared_params: SharedParams,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub crypto_scheme: CryptoSchemeFlag,
}


impl SignCmd {
	/// Run the command
	pub fn run(&self) -> error::Result<()> {
		let message = read_message(self.message.as_ref(), self.hex)?;
		let suri = read_uri(self.suri.as_ref())?;
		let password = self.keystore_params.read_password()?;

		let signature = with_crypto_scheme!(
			self.crypto_scheme.scheme,
			sign(&suri, &password, message)
		)?;

		println!("{}", signature);
		Ok(())
	}
}

impl CliConfiguration for SignCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		Some(&self.keystore_params)
	}
}

fn sign<P: sp_core::Pair>(suri: &str, password: &str, message: Vec<u8>) ->  error::Result<String> {
	let pair = pair_from_suri::<P>(suri, password);
	Ok(format!("{}", hex::encode(pair.sign(&message))))
}