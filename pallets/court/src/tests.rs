use super::*;
use frame_support::{
    assert_noop,
    assert_ok,
    impl_outer_event,
    impl_outer_origin,
    parameter_types,
    weights::Weight,
};
use frame_system::{self as system,};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::IdentityLookup,
    Perbill,
};
use util::{
    meta::VoteCall,
    organization::Organization,
    traits::GroupMembership,
    vote::{
        Threshold,
        VoterView,
    },
};

// type aliases
pub type AccountId = u64;
pub type BlockNumber = u64;

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}

mod court {
    pub use super::super::*;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        system<T>,
        pallet_balances<T>,
        org<T>,
        vote<T>,
        court<T>,
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = ();
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type AvailableBlockRatio = AvailableBlockRatio;
    type MaximumBlockLength = MaximumBlockLength;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
}
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Trait for Test {
    type Balance = u64;
    type Event = TestEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}
impl org::Trait for Test {
    type Event = TestEvent;
    type IpfsReference = u32; // TODO: replace with utils_identity::Cid
    type OrgId = u64;
    type Shares = u64;
}
impl vote::Trait for Test {
    type Event = TestEvent;
    type VoteId = u64;
    type Signal = u64;
}
parameter_types! {
    pub const MinimumDisputeAmount: u64 = 10;
}
impl Trait for Test {
    type Event = TestEvent;
    type Currency = Balances;
    type DisputeId = u64;
    type MinimumDisputeAmount = MinimumDisputeAmount;
}
pub type System = system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
pub type Org = org::Module<Test>;
pub type Vote = vote::Module<Test>;
pub type Court = Module<Test>;

fn get_last_event() -> RawEvent<u64, u64, u64, u64, u64> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let TestEvent::court(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 100), (2, 98), (3, 200), (4, 75), (5, 10), (6, 69)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    org::GenesisConfig::<Test> {
        first_organization_supervisor: 1,
        first_organization_value_constitution: 1738,
        first_organization_flat_membership: vec![1, 2, 3, 4, 5, 6],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext: sp_io::TestExternalities = t.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn genesis_config_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Org::organization_counter(), 1);
        let constitution = 1738;
        let expected_organization =
            Organization::new(Some(1), None, constitution);
        let org_in_storage = Org::organization_states(1u64).unwrap();
        assert_eq!(expected_organization, org_in_storage);
        for i in 1u64..7u64 {
            assert!(Org::is_member_of_group(1u64, &i));
        }
        assert!(System::events().is_empty());
    });
}

#[test]
fn dispute_registration_works() {
    new_test_ext().execute_with(|| {
        let one = Origin::signed(1);
        let signal_threshold = Threshold::new(1, None);
        let new_resolution_metadata = VoteMetadata::Signal(VoteCall::new(
            OrgRep::Equal(1),
            signal_threshold,
            None,
        ));
        assert_noop!(
            Court::register_dispute_type_with_resolution_path(
                one.clone(),
                9,
                2,
                new_resolution_metadata.clone(),
                None,
            ),
            Error::<Test>::DisputeMustExceedModuleMinimum
        );
        assert_noop!(
            Court::register_dispute_type_with_resolution_path(
                one.clone(),
                101,
                2,
                new_resolution_metadata.clone(),
                None,
            ),
            DispatchError::Module {
                index: 0,
                error: 3,
                message: Some("InsufficientBalance")
            }
        );
        assert_ok!(Court::register_dispute_type_with_resolution_path(
            one.clone(),
            10,
            2,
            new_resolution_metadata,
            None,
        ));
        assert_eq!(
            get_last_event(),
            RawEvent::RegisteredDisputeWithResolutionPath(
                1,
                1,
                10,
                2,
                OrgRep::Equal(1)
            )
        );
    });
}

#[test]
fn dispute_raised_works() {
    new_test_ext().execute_with(|| {
        let one = Origin::signed(1);
        let two = Origin::signed(2);
        let signal_threshold = Threshold::new(1, None);
        let new_resolution_metadata = VoteMetadata::Signal(VoteCall::new(
            OrgRep::Equal(1),
            signal_threshold,
            None,
        ));
        assert_noop!(
            Court::raise_dispute_to_trigger_vote(two.clone(), 1),
            Error::<Test>::CannotRaiseDisputeIfDisputeStateDNE
        );
        assert_ok!(Court::register_dispute_type_with_resolution_path(
            one.clone(),
            10,
            2,
            new_resolution_metadata,
            None,
        ));
        assert_noop!(
            Court::raise_dispute_to_trigger_vote(one.clone(), 1),
            Error::<Test>::SignerNotAuthorizedToRaiseThisDispute
        );
        assert_ok!(Court::raise_dispute_to_trigger_vote(two.clone(), 1));
        assert_eq!(
            get_last_event(),
            RawEvent::DisputeRaisedAndVoteTriggered(
                1,
                1,
                10,
                2,
                OrgRep::Equal(1),
                1
            )
        );
    })
}

#[test]
fn poll_dispute_to_execute_outcome_works() {
    new_test_ext().execute_with(|| {
        let one = Origin::signed(1);
        let two = Origin::signed(2);
        let signal_threshold = Threshold::new(1, None);
        let new_resolution_metadata = VoteMetadata::Signal(VoteCall::new(
            OrgRep::Equal(1),
            signal_threshold,
            None,
        ));
        assert_ok!(Court::register_dispute_type_with_resolution_path(
            one.clone(),
            10,
            2,
            new_resolution_metadata,
            None,
        ));
        assert_noop!(
            Court::poll_dispute_to_execute_outcome(one.clone(), 1),
            Error::<Test>::ActiveDisputeCannotBePolledFromCurrentState
        );
        assert_ok!(Court::raise_dispute_to_trigger_vote(two.clone(), 1));
        assert_noop!(
            Court::poll_dispute_to_execute_outcome(one.clone(), 1),
            Error::<Test>::VoteOutcomeInconclusiveSoPollCannotExecuteOutcome
        );
        // use vote to pass the proposal
        assert_ok!(Vote::submit_vote(one.clone(), 1, VoterView::InFavor, None));
        // then poll again to execute
        assert_ok!(Court::poll_dispute_to_execute_outcome(one.clone(), 1));
    });
}
