// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use methods::{MULTIPLY_ELF, MULTIPLY_ID};

use risc0_zkvm::{
    serde::{from_slice, to_vec},
    Executor, ExecutorEnv, SessionReceipt,
};

use subxt::{
	config::WithExtrinsicParams,
	ext::{
		sp_core::{sr25519::Pair as SubxtPair, Pair as SubxtPairT},
		sp_runtime::{AccountId32, MultiAddress},
	},
	SubstrateConfig,
	tx::{BaseExtrinsicParams, PairSigner, PlainTip},
	OnlineClient, PolkadotConfig,
};

// // Runtime types, etc
#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod substrate_node {}

use crate::substrate_node::runtime_types::{
    frame_system::AccountInfo,
    pallet_balances::AccountData
};

type ApiType = OnlineClient<WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>>;

fn alice() -> subxt::ext::sp_core::sr25519::Pair  {
    SubxtPair::from_string("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a", None).unwrap()
}

fn bob() -> subxt::ext::sp_core::sr25519::Pair  {
    SubxtPair::from_string("0x398f0c28f98885e046333d4a41c19cee4c37368a9832c6502f6cfd182e2aef89", None).unwrap()
}

async fn account_query(api: &ApiType, account: AccountId32)  -> Result<Option<AccountInfo<u32, AccountData<u128>>>, subxt::Error> {
    let query = substrate_node::storage().system().account(&account);
    let query_result = api.storage().fetch(&query, None).await;
	query_result
}

#[tokio::main]
async fn main() {
    let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();

    let alice_result = account_query(&api, alice().public().into()).await;
    let bob_result = account_query(&api, bob().public().into()).await;

    let alice_free_balance = alice_result.unwrap().unwrap().data.free;
    let bob_free_balance = bob_result.unwrap().unwrap().data.free;

    // Pick two numbers TODO: pass alice and bob free balance into the guest
    let (receipt, _) = factors(17, 23);

    // Here is where one would send 'receipt' over the network...

    // Verify receipt, panic if it's wrong
    receipt.verify(MULTIPLY_ID).expect(
        "Code you have proven should successfully verify; did you specify the correct image ID?",
    );

    // TODO: Below needs update to use changes to receipts in 0.14.0
    // let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
    // let restored_key = SubxtPair::from_string("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a", None).unwrap();
    // let signer = PairSigner::new(restored_key);

    // api
    //     .tx()
    //     .sign_and_submit_then_watch_default(
    //         &myexamplenode::tx().template_module().send_factors_receipt(
    //             // receipt.journal,
    //             receipt.seal,
    //             // MULTIPLY_ID
    //         ),
    //         &signer
    //     )
    //     .await.unwrap()
    //     .wait_for_finalized()
    //     .await.unwrap();

}

// Multiply them inside the ZKP
fn factors(a: u64, b: u64) -> (SessionReceipt, u64) {
    let env = ExecutorEnv::builder()
        // Send a & b to the guest
        .add_input(&to_vec(&a).unwrap())
        .add_input(&to_vec(&b).unwrap())
        .build();

    // First, we make an executor, loading the 'multiply' ELF binary.
    let mut exec = Executor::from_elf(env, MULTIPLY_ELF).unwrap();

    // Run the executor to produce a session.
    let session = exec.run().unwrap();

    // Prove the session to produce a receipt.
    let receipt = session.prove().unwrap();

    // Extract journal of receipt (i.e. output c, where c = a * b)
    let c: u64 = from_slice(&receipt.journal).expect(
        "Journal output should deserialize into the same types (& order) that it was written",
    );

    // Print an assertion
    println!("I know the factors of {}, and I can prove it!", c);

    (receipt, c)
}

#[cfg(test)]
mod tests {
    #[test]
    fn factors() {
        const TEST_FACTOR_ONE: u64 = 17;
        const TEST_FACTOR_TWO: u64 = 23;
        let (_, result) = super::factors(17, 23);
        assert_eq!(
            result,
            TEST_FACTOR_ONE * TEST_FACTOR_TWO,
            "We expect the zkVM output to be the product of the inputs"
        )
    }
}