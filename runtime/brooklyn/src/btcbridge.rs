
parameter_types! {
  pub const ParachainBlocksPerBitcoinBlock: BlockNumber = BITCOIN_BLOCK_SPACING;
}

impl btc_relay::Config for Runtime {
  type RuntimeEvent = RuntimeEvent;
  type WeightInfo = weights::btc_relay::WeightInfo<Runtime>;
  type ParachainBlocksPerBitcoinBlock = ParachainBlocksPerBitcoinBlock;
}

parameter_types! {
  pub const MaxExpectedValue: UnsignedFixedPoint = UnsignedFixedPoint::from_inner(<UnsignedFixedPoint as FixedPointNumber>::DIV);
}

impl fee::Config for Runtime {
  type FeePalletId = FeePalletId;
  type WeightInfo = weights::fee::WeightInfo<Runtime>;
  type SignedFixedPoint = SignedFixedPoint;
  type SignedInner = SignedInner;
  type CapacityRewards = VaultCapacity;
  type VaultRewards = VaultRewards;
  type VaultStaking = VaultStaking;
  type OnSweep = currency::SweepFunds<Runtime, FeeAccount>;
  type MaxExpectedValue = MaxExpectedValue;
  type NominationApi = Nomination;
}