// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "full_crypto")]
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use libsm::sm2 as sm2;
use sp_std::cmp::Ordering;
use codec::{Encode, Decode};

#[cfg(feature = "full_crypto")]
use core::convert::{TryFrom, TryInto};
#[cfg(feature = "std")]
use substrate_bip39::seed_from_entropy;
#[cfg(feature = "std")]
use bip39::{Mnemonic, Language, MnemonicType};
#[cfg(feature = "full_crypto")]
use crate::{hashing::blake2_256, crypto::{Pair as TraitPair, DeriveJunction, SecretStringError}};
use crate::crypto::{Public as TraitPublic, CryptoTypePublicPair, UncheckedFrom, CryptoType, Derive, CryptoTypeId};
#[cfg(feature = "std")]
use crate::crypto::Ss58Codec;
#[cfg(feature = "std")]
use serde::{de, Serializer, Serialize, Deserializer, Deserialize};
#[cfg(feature = "full_crypto")]
use sm2::signature::{Seckey, Pubkey};
use sp_runtime_interface::pass_by::{PassBy, PassByInner, Codec};



/// An identifier used to match public keys against sm2 keys
pub const CRYPTO_ID: CryptoTypeId = CryptoTypeId(*b"csm2");

/// A secret seed (which is bytewise essentially equivalent to a SecretKey).
///
/// We need it as a different type because `Seed` is expected to be AsRef<[u8]>.
#[cfg(feature = "full_crypto")]
type Seed = [u8; 32];

/// the SM2 conpressed key.
#[derive(Clone, Encode, Decode, PassByInner)]
pub struct Public([u8; 33]);

impl PartialOrd for Public {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Public {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_ref().cmp(&other.as_ref())
	}
}

impl PartialEq for Public {
	fn eq(&self, other: &Self) -> bool {
		self.as_ref() == other.as_ref()
	}
}

impl Eq for Public {}


/// An error type for SS58 decoding.
#[cfg(feature = "std")]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum PublicError {
	/// Bad alphabet.
	BadBase58,
	/// Bad length.
	BadLength,
	/// Unknown version.
	UnknownVersion,
	/// Invalid checksum.
	InvalidChecksum,
}

impl Public {
	/// A new instance from the given 33-byte `data`.
	///
	/// NOTE: No checking goes on to ensure this is a real public key. Only use it if
	/// you are certain that the array actually is a pubkey. GIGO!
	pub fn from_raw(data: [u8; 33]) -> Self {
		Self(data)
	}

	/// Create a new instance from the given full public key.
	///
	/// This will convert the full public key into the compressed format.
	#[cfg(feature = "std")]
	pub fn from_full(full: &[u8]) -> Result<Self, ()> {
		match sm2::utils::full_pk_to_compress(full) {
			Ok(c) => Ok(Public::from_raw(c)),
			Err(_) => Err(()),
		}
	}

	pub fn into_slice(&self) -> &[u8; 33] {
		&self.0
	}
}


impl TraitPublic for Public {
	/// A new instance from the given slice that should be 33 bytes long.
	///
	/// NOTE: No checking goes on to ensure this is a real public key. Only use it if
	/// you are certain that the array actually is a pubkey. GIGO!
	fn from_slice(data: &[u8]) -> Self {
		let mut r = [0u8; 33];
		r.copy_from_slice(data);
		Self(r)
	}

	fn to_public_crypto_pair(&self) -> CryptoTypePublicPair {
		CryptoTypePublicPair(CRYPTO_ID, self.to_raw_vec())
	}
}

impl From<Public> for CryptoTypePublicPair {
	fn from(key: Public) -> Self {
		(&key).into()
	}
}

impl From<&Public> for CryptoTypePublicPair {
	fn from(key: &Public) -> Self {
		CryptoTypePublicPair(CRYPTO_ID, key.to_raw_vec())
	}
}

impl Derive for Public {}

impl Default for Public {
	fn default() -> Self {
		Public([0u8; 33])
	}
}

impl AsRef<[u8]> for Public {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

impl AsMut<[u8]> for Public {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0[..]
	}
}

impl sp_std::convert::TryFrom<&[u8]> for Public {
	type Error = ();

	fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
		if data.len() == 33 {
			Ok(Self::from_slice(data))
		} else {

			Err(())
		}
	}
}


#[cfg(feature = "full_crypto")]
impl From<Pair> for Public {
	fn from(x: Pair) -> Self {
		x.public()
	}
}

impl UncheckedFrom<[u8; 33]> for Public {
	fn unchecked_from(x: [u8; 33]) -> Self {
		Public(x)
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for Public {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.to_ss58check())
	}
}

impl sp_std::fmt::Debug for Public {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let s = self.to_ss58check();
		write!(f, "{} ({}...)", crate::hexdisplay::HexDisplay::from(&self.as_ref()), &s[0..8])
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

#[cfg(feature = "std")]
impl Serialize for Public {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&self.to_ss58check())
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Public {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		Public::from_ss58check(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))
	}
}

#[cfg(feature = "full_crypto")]
impl sp_std::hash::Hash for Public {
	fn hash<H: sp_std::hash::Hasher>(&self, state: &mut H) {
		self.as_ref().hash(state);
	}
}

/// A signature
#[derive(Encode, Decode, PassByInner)]
pub struct Signature([u8; 97]);


#[cfg(feature = "std")]
impl From<(sm2::signature::Signature, Public)> for Signature {
	fn from(x: (sm2::signature::Signature, Public)) -> Signature {
		let mut buf = [0u8; 97];
		let r = x.0.get_r();
		let s = x.0.get_s();
		buf[0..32].copy_from_slice(&r.to_bytes_be());
		buf[32..64].copy_from_slice(&s.to_bytes_be());
		buf[64..97].copy_from_slice(&x.1.as_ref());
		Signature(buf)
	}
}

impl sp_std::convert::TryFrom<&[u8]> for Signature {
	type Error = ();

	fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
		if data.len() == 97 {
			let mut inner = [0u8; 97];
			inner.copy_from_slice(data);
			Ok(Signature(inner))
		} else {
			Err(())
		}
	}
}

#[cfg(feature = "std")]
impl Serialize for Signature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&hex::encode(self))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Signature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		let signature_hex = hex::decode(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))?;
		Ok(Signature::try_from(signature_hex.as_ref())
			.map_err(|e| de::Error::custom(format!("{:?}", e)))?)
	}
}


impl Clone for Signature {
	fn clone(&self) -> Self {
		let mut r = [0u8; 97];
		r.copy_from_slice(&self.0[..]);
		Signature(r)
	}
}

impl Default for Signature {
	fn default() -> Self {
		Signature([0u8; 97])
	}
}

impl PartialEq for Signature {
	fn eq(&self, b: &Self) -> bool {
		self.0[..] == b.0[..]
	}
}

impl Eq for Signature {}

impl From<Signature> for [u8; 97] {
	fn from(v: Signature) -> [u8; 97] {
		v.0
	}
}

impl AsRef<[u8; 97]> for Signature {
	fn as_ref(&self) -> &[u8; 97] {
		&self.0
	}
}

impl AsRef<[u8]> for Signature {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

impl AsMut<[u8]> for Signature {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0[..]
	}
}

impl sp_std::fmt::Debug for Signature {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "{}", crate::hexdisplay::HexDisplay::from(&self.0.as_ref()))
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

#[cfg(feature = "full_crypto")]
impl sp_std::hash::Hash for Signature {
	fn hash<H: sp_std::hash::Hasher>(&self, state: &mut H) {
		sp_std::hash::Hash::hash(&self.0[..], state);
	}
}

impl Signature {
	/// A new instance from the given 97-byte `data`.
	///
	/// NOTE: No checking goes on to ensure this is a real signature. Only use it if
	/// you are certain that the array actually is a signature. GIGO!
	pub fn from_raw(data: [u8; 97]) -> Signature {
		Signature(data)
	}

	/// A new instance from the given slice that should be 97 bytes long.
	///
	/// NOTE: No checking goes on to ensure this is a real signature. Only use it if
	/// you are certain that the array actually is a signature. GIGO!
	pub fn from_slice(data: &[u8]) -> Self {
		let mut r = [0u8; 97];
		r.copy_from_slice(data);
		Signature(r)
	}

	pub fn into_sm2_sig(&self) -> [u8; 64] {
		let mut buf = [0u8; 64];
		buf.copy_from_slice(&self.0[0..64]);
		buf
	}

	pub fn into_sm2_pk(&self) -> [u8; 33] {
		let mut buf = [0u8; 33];
		buf.copy_from_slice(&self.0[64..97]);
		buf
	}


}


/// Derive a single hard junction.
#[cfg(feature = "full_crypto")]
fn derive_hard_junction(secret_seed: &Seed, cc: &[u8; 32]) -> Seed {
	("SCA256HDKD", secret_seed, cc).using_encoded(|data| {
		let mut res = [0u8; 32];
		res.copy_from_slice(blake2_rfc::blake2b::blake2b(32, &[], data).as_bytes());
		res
	})
}

/// An error when deriving a key.
#[cfg(feature = "full_crypto")]
pub enum DeriveError {
	/// A soft key was found in the path (and is unsupported).
	SoftKeyInPath,
}

/// A key pair.
#[cfg(feature = "full_crypto")]
#[derive(Clone)]
pub struct Pair {
	public: Pubkey,
	secret: Seckey,
}


#[cfg(feature = "full_crypto")]
impl TraitPair for Pair {
	type Public = Public;
	type Seed = Seed;
	type Signature = Signature;
	type DeriveError = DeriveError;

	#[cfg(feature = "std")]
	fn generate_with_phrase(password: Option<&str>) -> (Pair, String, Seed) {
		let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
		let phrase = mnemonic.phrase();
		let (pair, seed) = Self::from_phrase(phrase, password)
			.expect("All phrases generated by Mnemonic are valid; qed");
		(
			pair,
			phrase.to_owned(),
			seed,
		)
	}

	#[cfg(feature = "std")]
	fn from_phrase(phrase: &str, password: Option<&str>) -> Result<(Pair, Seed), SecretStringError> {
		let big_seed = seed_from_entropy(
			Mnemonic::from_phrase(phrase, Language::English)
				.map_err(|_| SecretStringError::InvalidPhrase)?.entropy(),
			password.unwrap_or(""),
		).map_err(|_| SecretStringError::InvalidSeed)?;
		let mut seed = Seed::default();
		seed.copy_from_slice(&big_seed[0..32]);
		Self::from_seed_slice(&big_seed[0..32]).map(|x| (x, seed))
	}


	/// Derive a child key from a series of given junctions.
	fn derive<Iter: Iterator<Item=DeriveJunction>>(&self,
	                                               path: Iter,
	                                               _seed: Option<Seed>
	) -> Result<(Pair, Option<Seed>), DeriveError> {
		let mut acc = sm2::utils::sk_to_bytes(&self.secret);
		for j in path {
			match j {
				DeriveJunction::Soft(_cc) => return Err(DeriveError::SoftKeyInPath),
				DeriveJunction::Hard(cc) => acc = derive_hard_junction(&acc, &cc),
			}
		}
		Ok((Self::from_seed(&acc), Some(acc)))
	}

	/// Make a new key pair from secret seed material.
	///
	/// You should never need to use this; generate(), generate_with_phrase
	fn from_seed(seed: &Seed) -> Pair {
		Self::from_seed_slice(&seed[..]).expect("seed has valid length; qed")
	}

	/// Make a new key pair from secret seed material. The slice must be 32 bytes long or it
	/// will return `None`.
	///
	/// You should never need to use this; generate(), generate_with_phrase
	fn from_seed_slice(seed_slice: &[u8]) -> Result<Pair, SecretStringError> {
		let secret = sm2::utils::parse_sk(seed_slice)
			.map_err(|_| SecretStringError::InvalidSeedLength)?;
		let public = sm2::utils::pk_from_sk(&secret);
		Ok(Pair{ secret, public })
	}

	/// Sign a message.
	fn sign(&self, message: &[u8]) -> Signature {
		let (sig, pk) = sm2::utils::sign(&message, &self.secret, &self.public);
		Signature::from((sig, Public::from_raw(pk)))

	}

	/// Verify a signature on a message. Returns true if the signature is good.
	fn verify<M: AsRef<[u8]>>(sig: &Self::Signature, message: M, pubkey: &Self::Public) -> bool {
		let sm2_sig = sm2::signature::Signature::parse(&sig.into_sm2_sig());
		let pk = match sm2::utils::parse_pk(pubkey.as_ref()) {
			Ok(p) => p,
			Err(_) => return false,
		};
		sm2::utils::verify(message.as_ref(), &pk, &sm2_sig)
	}

	/// Verify a signature on a message. Returns true if the signature is good.
	/// actually the same with `verify`
	fn verify_weak<P: AsRef<[u8]>, M: AsRef<[u8]>>(sig: &[u8], message: M, pubkey: P) -> bool {
		if sig.len() != 97 { return false }
		Self::verify(&Signature::from_slice(sig.as_ref()), message, &Self::Public::from_slice(pubkey.as_ref()))
	}

	/// Get the public key.
	fn public(&self) -> Public {
		Public(sm2::utils::point_to_compress(&self.public))
	}

	/// Return a vec filled with raw data.
	fn to_raw_vec(&self) -> Vec<u8> {
		self.seed().to_vec()
	}

}

#[cfg(feature = "full_crypto")]
impl Pair {
	/// Get the seed for this key.
	pub fn seed(&self) -> Seed {
		sm2::utils::sk_to_bytes(&self.secret)
	}

	/// Exactly as `from_string` except that if no matches are found then, the the first 32
	/// characters are taken (padded with spaces as necessary) and used as the MiniSecretKey.
	#[cfg(feature = "std")]
	pub fn from_legacy_string(s: &str, password_override: Option<&str>) -> Pair {
		Self::from_string(s, password_override).unwrap_or_else(|_| {
			let mut padded_seed: Seed = [' ' as u8; 32];
			let len = s.len().min(32);
			padded_seed[..len].copy_from_slice(&s.as_bytes()[..len]);
			Self::from_seed(&padded_seed)
		})
	}
}


impl CryptoType for Public {
	#[cfg(feature="full_crypto")]
	type Pair = Pair;
}

impl CryptoType for Signature {
	#[cfg(feature="full_crypto")]
	type Pair = Pair;
}

#[cfg(feature="full_crypto")]
impl CryptoType for Pair {
	type Pair = Pair;
}
