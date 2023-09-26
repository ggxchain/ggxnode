use super::*;

const fn percent(x: i32) -> sp_arithmetic::FixedI64 {
	sp_arithmetic::FixedI64::from_rational(x as u128, 100)
}
use pallet_referenda::Curve;

use runtime_common::prod_or_fast;

const APP_ROOT: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_ROOT: Curve = Curve::make_linear(28, 28, percent(0), percent(50));

const HOURS: BlockNumber = 3600 / 2; // 2 seconds per block. Hacky solution, because TrackInfo doesn't support Decode trait.
const TRACKS_DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 1] = [(
	0,
	pallet_referenda::TrackInfo {
		name: "root",
		max_deciding: prod_or_fast!(1, 5), // 1 or 5 proposals can be decided at the same time for Root track (in parallel)
		decision_deposit: prod_or_fast!(5 * KGGX, 500 * GGX),
		prepare_period: 2 * HOURS,
		decision_period: prod_or_fast!(14 * (24 * HOURS), 10 * HOURS),
		confirm_period: prod_or_fast!(24 * HOURS, 10 * HOURS),
		min_enactment_period: prod_or_fast!(24 * HOURS, 2 * HOURS),
		min_approval: APP_ROOT,
		min_support: SUP_ROOT,
	},
)];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = u16;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
	fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
		&TRACKS_DATA
	}
	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}
pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);
