use serde_json::json as sjson;
use bip39::{MnemonicType, Mnemonic, Language};
use sp_core::{
    sr25519::{self, Public as srPublic},
    Pair, hexdisplay::HexDisplay,
    crypto::{Ss58Codec, Ss58AddressFormat},
};


use sp_runtime::{MultiSigner, traits::IdentifyAccount};


use crate::settings::Settings;
use crate::error::{MinerError, Result};

/// Public key type for Runtime
pub type PublicFor<P> = <P as sp_core::Pair>::Public;
/// Seed type for Runtime
pub type SeedFor<P> = <P as sp_core::Pair>::Seed;

pub fn format_seed<P: sp_core::Pair>(seed: SeedFor<P>) -> String {
    // format!("0x{}", HexDisplay::from(&seed.as_ref()))
    format!("{}", HexDisplay::from(&seed.as_ref()))
}

fn format_public_key<P: sp_core::Pair>(public_key: PublicFor<P>) -> String {
    // format!("0x{}", HexDisplay::from(&public_key.as_ref()))
    format!("{}", HexDisplay::from(&public_key.as_ref()))
}

pub fn generate(words: &str) -> Result<()> {
    let words = words.parse::<usize>().unwrap_or(12);


    let mnemonic = Mnemonic::new(MnemonicType::for_word_count(words).unwrap(), Language::English);


    if let Ok((pair, seed)) = sr25519::Pair::from_phrase(mnemonic.phrase(), None) {
        let json = sjson!({
					"secret_phrase": mnemonic.phrase(),
					"secret_seed": format_seed::<sr25519::Pair>(seed),
					"public_key": format_public_key::<sr25519::Pair>(pair.public().clone()),
					"miner_id": pair.public(),
				});
        println!("{}", serde_json::to_string_pretty(&json).expect("Json pretty print failed"));
    }
    Ok(())
}