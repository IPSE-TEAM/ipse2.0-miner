use codec::Encode;
use core::marker::PhantomData;
use frame_support::Parameter;
use sp_runtime::{
    traits::{AtLeast32Bit, Scale},
    generic::{Header as SHeader},
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature, OpaqueExtrinsic,
};


use sub_runtime::ipse::{Order, Miner};
use substrate_subxt::{balances::{Balances, AccountData, BalancesEventsDecoder}, module, PairSigner, extrinsic::{DefaultExtra}, Runtime, Client, system::{System, SystemEventsDecoder}, Call, Store};
use sp_core::{sr25519::Pair, Pair as PairT};

use frame_support::sp_runtime::SaturatedConversion;

use crate::error::MinerError;
use crate::settings::Settings;
use sp_core::crypto::{Ss58Codec, AccountId32};

pub type AccountId = <IpseRuntime as System>::AccountId;
pub type Balance = <IpseRuntime as Balances>::Balance;


// define module, Store, Call , Event macro

// define submit module
#[module]
pub trait Ipse: System + Balances {}


// define Store
#[derive(Encode, Store)]
pub struct MinersStore<T: Ipse> {
    #[store(returns = Option < Miner < AccountId, Balance >>)]
    pub key: AccountId,
    pub _runtime: PhantomData<T>,
}

#[derive(Encode, Store)]
pub struct OrdersStore<T: Ipse> {
    #[store(returns = Vec < Order < AccountId, Balance >>)]
    pub _runtime: PhantomData<T>,
}

// define call
#[derive(Encode, Call)]
pub struct RegisterMinerCall<T: Ipse> {
    pub _runtime: PhantomData<T>,
    pub nickname: Vec<u8>,
    pub region: Vec<u8>,
    pub url: Vec<u8>,
    pub public_key: Vec<u8>,
    pub income_address: AccountId,
    pub capacity: u64,
    pub unit_price: Balance,
}

#[derive(Encode, Call)]
pub struct ConfirmOrderCall<T: Ipse> {
    pub _runtime: PhantomData<T>,
    pub order_id: u64,
    pub url: Vec<u8>,
}

#[derive(Encode, Call)]
pub struct DeleteOrderCall<T: Ipse> {
    pub _runtime: PhantomData<T>,
    pub order_id: u64,
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IpseRuntime;


impl Ipse for IpseRuntime {}

impl Runtime for IpseRuntime {
    type Signature = MultiSignature;
    type Extra = DefaultExtra<Self>;
}

pub trait Timestamp: System {
    type Moment: Parameter
    + Default
    + AtLeast32Bit
    + Scale<Self::BlockNumber, Output=Self::Moment>
    + Copy;
}


impl System for IpseRuntime {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
    type Address = pallet_indices::address::Address<Self::AccountId, u32>;
    type Header = SHeader<Self::BlockNumber, BlakeTwo256>;
    type Extrinsic = OpaqueExtrinsic;
    type AccountData = AccountData<<Self as Balances>::Balance>;
}

impl Balances for IpseRuntime {
    type Balance = u128;
}

impl Timestamp for IpseRuntime {
    type Moment = u128;
}


pub async fn register_miner(settings: &Settings, pair: Pair, sub_client: Client<IpseRuntime>) -> Result<sp_core::H256, MinerError> {

    // https://stackoverflow.com/questions/56081117/how-do-you-convert-between-substrate-specific-types-and-rust-primitive-types

    let signer = PairSigner::new(pair);

    let res = sub_client.register_miner(
        &signer,
        settings.miner.nickname.as_bytes().to_vec(),
        settings.miner.region.as_bytes().to_vec(),
        settings.miner.url.as_bytes().to_vec(),
        settings.miner.public_key.as_bytes().to_vec(),
        AccountId32::from_string(settings.miner.income_address.as_str())?,
        settings.miner.capacity as u64,
        // (settings.miner.unit_price * (10 as u64).pow(14)).saturated_into::<Balance>(),
        settings.miner.unit_price .saturated_into::<Balance>(),
    ).await?;

    Ok(res)
}

pub async fn confirm_order(pair: Pair, sub_client: Client<IpseRuntime>, order_id: u64, url: String) -> Result<sp_core::H256, MinerError> {
    let signer = PairSigner::new(pair);
    let res = sub_client.confirm_order(
        &signer,
        order_id,
        url.into_bytes(),
    ).await?;
    Ok(res)
}

pub async fn delete_order(pair: Pair, sub_client: Client<IpseRuntime>, order_id: u64) -> Result<sp_core::H256, MinerError> {
    let signer = PairSigner::new(pair);
    let res = sub_client.delete_order(
        &signer,
        order_id,
    ).await?;
    Ok(res)
}