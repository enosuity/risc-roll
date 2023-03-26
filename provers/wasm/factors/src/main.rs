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
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::Prover;

 fn main()  {
    // Number to square in the WASM
    let a: i32 = 6;

    // Multiply them inside the ZKP
    // First, we make the prover, loading the 'multiply' method
    let mut prover = Prover::new(MULTIPLY_ELF).expect(
        "Prover should be constructed from valid method source code and corresponding method ID",
    );

    // Send the input to the gues
    prover.add_input_u32_slice(&to_vec(&a).expect("should be serializable"));
    // Run prover & generate receipt
    let receipt = prover
        .run()
        .expect("Should be able to prove valid code that fits in the cycle count.");

    // Extract journal of receipt (i.e. output c, where c = a * b)
    let c: i32 = from_slice(&receipt.journal).expect(
        "Journal output should deserialize into the same types (& order) that it was written",
    );

    // Print an assertion
    println!("The execution result is {:?}", c);

    // Here is where one would send 'receipt' over the network...

    // Verify receipt, panic if it's wrong
    receipt.verify(&MULTIPLY_ID).expect(
        "Code you have proven should successfully verify; did you specify the correct method ID?",
    );
}
