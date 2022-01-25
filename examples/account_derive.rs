use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub struct Cool(String);

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub struct AccountTest {
    pub data: u8,
    pub cool: Cool,
    pub stuff: u64,
}
