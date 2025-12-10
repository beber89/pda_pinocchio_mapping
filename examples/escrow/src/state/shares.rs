use bytemuck::{Pod, Zeroable};
use pda_pinocchio_mapping::Bumpy;

impl Bumpy for Share {
    fn bump(&self) -> u8 {
        self.bump
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Zeroable, Pod)]
pub struct Share {
    pub maker: [u8; 32],
    pub taker: [u8; 32],
    pub amount: [u8; 8],
    pub bump: u8,
}
