#!/bin/bash
set -e

echo "====================================================================="
echo "LEZ General Calls via Tail Calls (LP-0015) E2E Demo"
echo "====================================================================="

export RISC0_DEV_MODE=0
export RUST_LOG=info
export LOGOS_BLOCKCHAIN_CIRCUITS=~/.logos-blockchain-circuits
export ROCKSDB_LIB_DIR=/usr/lib
export CXX=clang++

echo -e "\n Demonstrating the Positive Scenario: User -> A -> B -> A"
cargo test -p integration_tests --test general_calls -- test_multi_hop_positive_chain --nocapture

echo -e "\n Demonstrating Direct Access Prevention"
cargo test -p integration_tests --test general_calls -- test_negative_direct_call --nocapture

echo -e "\n Demonstrating Replay Attack & Forged Ticket"
cargo test -p integration_tests --test general_calls -- test_replay_and_forged_ticket --nocapture
