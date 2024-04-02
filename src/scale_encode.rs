#![allow(dead_code)]
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use sp_core::{crypto::Ss58Codec, ed25519::Public};
use sp_runtime::traits::ConstU32;

pub type BoundedAuthorityList = BoundedVec<(Public, u64), ConstU32<100_000>>;

#[derive(Encode, Decode, MaxEncodedLen)]
pub struct StoredPendingChange {
    /// The block number this was scheduled at.
    pub scheduled_at: u32,
    /// The delay in blocks until it will be applied.
    pub delay: u32,
    /// The next authority set, weakly bounded in size by `Limit`.
    pub next_authorities: BoundedAuthorityList,
    /// If defined it means the change was forced and the given block number
    /// indicates the median last finalized block when the change was signaled.
    pub forced: Option<u32>,
}

impl StoredPendingChange {
    // Constructor function to create an instance of StoredPendingChange
    pub fn new(
        scheduled_at: u32,
        delay: u32,
        next_authorities: BoundedAuthorityList,
        forced: Option<u32>,
    ) -> Self {
        StoredPendingChange {
            scheduled_at,
            delay,
            next_authorities,
            forced,
        }
    }
}

pub fn encode() {
    // Load ed25519 keys of  the validators
    let ss58_addresses = vec![
        "5FCPJZaG5DJrQ9q2roHqtkgHgFPk8Ky18JPv1y5fSn87GWJF",
        "5HndcxSPvryrobq9SLfBZQrZ5mdrXKBUztnAwDY4gDFUEQ6h",
        "5E7Xr6BcGVZJBN1Zc27ZiYMK9HLfJLEAjPCkWBz4qn3LWgrW",
        "5G6FZ54tergPzSd1SxFqQL3V6viJ2D2Hf2DPcKuRxXnK4xQ5",
        "5FxpAURrSgoTKWJxfk2q8ZBauqyf4m9BqDxVC82q8nNjcdVe",
        "5DVg4ufvzJRYHbqpL6zS8RnDx11479n1c6ETXMDCPzV8T9Jt",
        "5EX6TsXMCxkPUv7x6Tk1hWZfJMGW2WbhndHxiVPvQY5u473j",
    ];
    let scheduled_at = 344815;
    let delay = 1;
    let mut next_authorities_vec = Vec::<(Public, u64)>::new();
    for ss58_address in ss58_addresses {
        let public_key = Public::from_ss58check(ss58_address).expect("Error decoding SS58");
        next_authorities_vec.push((public_key, 1u64));
    }

    // Convert Vec to BoundedAuthorityList
    let next_authorities =
        BoundedVec::try_from(next_authorities_vec).expect("Error converting to bounded_vec");
    let forced = Some(344637);

    let pending_change = StoredPendingChange::new(scheduled_at, delay, next_authorities, forced);
    println!("{}", hex::encode(pending_change.encode()));
}
