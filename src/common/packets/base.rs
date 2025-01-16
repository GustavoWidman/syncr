pub trait PacketBase: Send + Sync + 'static {
    type BuildParams;
    const TYPE: &'static [u8; 4];

    fn build(params: Self::BuildParams) -> Self;
    fn get_type(&self) -> &[u8; 4] {
        Self::TYPE
    }
}
