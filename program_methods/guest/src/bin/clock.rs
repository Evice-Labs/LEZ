use nssa_core::program::{AccountPostState, ProgramInput, read_nssa_inputs, write_nssa_outputs};

type Instruction = nssa_core::Timestamp;

fn main() {
    let (
        ProgramInput {
            pre_states,
            instruction: timestamp,
        },
        instruction_words,
    ) = read_nssa_inputs::<Instruction>();

    let Ok([pre]) = <[_; 1]>::try_from(pre_states) else {
        return;
    };

    let account_pre = &pre.account;
    let account_pre_data = account_pre.data.clone().into_inner();
    let block_id = u64::from_le_bytes(
        account_pre_data[..8]
            .try_into()
            .expect("Block context program account data should contain a LE-encoded block_id u64"),
    );

    let mut account_post = account_pre.clone();
    let next_block_id = block_id
        .checked_add(1)
        .expect("Next block id should be within u64 boundaries");
    let mut data = [0u8; 16];
    data[..8].copy_from_slice(&next_block_id.to_le_bytes());
    data[8..].copy_from_slice(&timestamp.to_le_bytes());
    account_post.data = data
        .to_vec()
        .try_into()
        .expect("16 bytes should fit in account data");

    let post = AccountPostState::new(account_post);

    write_nssa_outputs(instruction_words, vec![pre], vec![post]);
}
