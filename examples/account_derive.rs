use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Cool(String);

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountTest {
    pub data: u8,
    pub cool: Cool,
    pub stuff: u64,
}
