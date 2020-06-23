use crate::traits::{
    Apply, Approved, ConsistentThresholdStructure, DeriveThresholdRequirement, Rejected, Revert,
    VoteVector,
};
use codec::{Decode, Encode};
use frame_support::Parameter;
use sp_runtime::PerThing;
use sp_std::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, sp_runtime::RuntimeDebug)]
/// The voter options (direction)
pub enum VoterView {
    /// Not yet voted
    NoVote,
    /// Voted in favor
    InFavor,
    /// Voted against
    Against,
    /// Acknowledged but abstained
    Abstain,
}

#[derive(new, Clone, Copy, PartialEq, Eq, Encode, Decode, sp_runtime::RuntimeDebug)]
/// Binary vote to express for/against with magnitude
/// ~ vectors have direction and magnitude, not to be confused with `Vec`
pub struct Vote<Signal, Hash> {
    magnitude: Signal,
    direction: VoterView,
    justification: Option<Hash>,
}

impl<Signal: Copy, Hash: Clone> Vote<Signal, Hash> {
    pub fn set_new_view(
        &self,
        new_direction: VoterView,
        new_justification: Option<Hash>,
    ) -> Option<Self> {
        if self.direction == new_direction {
            // new view not set because same object
            None
        } else {
            Some(Vote {
                magnitude: self.magnitude,
                direction: new_direction,
                justification: new_justification,
            })
        }
    }
}

impl<Signal: Copy, Hash: Clone> VoteVector<Signal, VoterView, Hash> for Vote<Signal, Hash> {
    fn magnitude(&self) -> Signal {
        self.magnitude
    }
    fn direction(&self) -> VoterView {
        self.direction
    }
    fn justification(&self) -> Option<Hash> {
        self.justification.clone()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Encode, Decode, sp_runtime::RuntimeDebug)]
/// This is the threshold configuration
/// - evaluates passage of vote state
pub struct ThresholdConfig<Signal> {
    /// Support threshold
    support_required: Signal,
    /// Required turnout
    turnout_required: Option<Signal>,
}

impl<Signal: PartialOrd + Copy> ThresholdConfig<Signal> {
    pub fn new(support_required: Signal, turnout_required: Option<Signal>) -> Option<Self> {
        if let Some(turnout_threshold) = turnout_required {
            if support_required < turnout_threshold {
                Some(ThresholdConfig {
                    support_required,
                    turnout_required,
                })
            } else {
                None
            }
        } else {
            Some(ThresholdConfig {
                support_required,
                turnout_required: None,
            })
        }
    }
    pub fn support_threshold(&self) -> Signal {
        self.support_required
    }
    pub fn turnout_threshold(&self) -> Option<Signal> {
        self.turnout_required
    }
    pub fn new_support_threshold(support_required: Signal) -> Self {
        ThresholdConfig {
            support_required,
            turnout_required: None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, sp_runtime::RuntimeDebug)]
/// The state of an ongoing vote
pub struct VoteState<Signal, BlockNumber, Hash> {
    /// Vote state must often be anchored to offchain state, cid
    topic: Option<Hash>,
    /// All signal in favor
    in_favor: Signal,
    /// All signal against
    against: Signal,
    /// All signal that votes at all
    turnout: Signal,
    /// All signal that can vote
    all_possible_turnout: Signal,
    /// The threshold requirement for passage
    passage_threshold: ThresholdConfig<Signal>,
    /// The threshold requirement for rejection
    rejection_threshold: Option<ThresholdConfig<Signal>>,
    /// The time at which this vote state is initialized
    initialized: BlockNumber,
    /// The time at which this vote state expires
    expires: Option<BlockNumber>,
    /// The vote outcome
    outcome: VoteOutcome,
}

impl<
        Signal: Copy
            + From<u32>
            + Default
            + sp_std::ops::Add<Output = Signal>
            + sp_std::ops::Sub<Output = Signal>,
        BlockNumber: Parameter + Copy + Default,
        Hash: Clone,
    > Default for VoteState<Signal, BlockNumber, Hash>
{
    fn default() -> VoteState<Signal, BlockNumber, Hash> {
        VoteState {
            topic: None,
            in_favor: 0u32.into(),
            against: 0u32.into(),
            turnout: 0u32.into(),
            all_possible_turnout: 0u32.into(),
            passage_threshold: ThresholdConfig::default(),
            rejection_threshold: None,
            initialized: BlockNumber::default(),
            expires: None,
            outcome: VoteOutcome::default(),
        }
    }
}

impl<
        Signal: Copy
            + From<u32>
            + Default
            + sp_std::ops::Add<Output = Signal>
            + sp_std::ops::Sub<Output = Signal>
            + PartialOrd,
        BlockNumber: Parameter + Copy + Default,
        Hash: Clone,
    > VoteState<Signal, BlockNumber, Hash>
{
    pub fn new(
        topic: Option<Hash>,
        all_possible_turnout: Signal,
        passage_threshold: ThresholdConfig<Signal>,
        rejection_threshold: Option<ThresholdConfig<Signal>>,
        initialized: BlockNumber,
        expires: Option<BlockNumber>,
    ) -> VoteState<Signal, BlockNumber, Hash> {
        VoteState {
            topic,
            all_possible_turnout,
            passage_threshold,
            rejection_threshold,
            initialized,
            expires,
            outcome: VoteOutcome::Voting,
            ..Default::default()
        }
    }
    pub fn new_unanimous_consent(
        topic: Option<Hash>,
        all_possible_turnout: Signal,
        initialized: BlockNumber,
        expires: Option<BlockNumber>,
    ) -> VoteState<Signal, BlockNumber, Hash> {
        let unanimous_passage_threshold =
            ThresholdConfig::new_support_threshold(all_possible_turnout);
        VoteState {
            topic,
            all_possible_turnout,
            passage_threshold: unanimous_passage_threshold,
            rejection_threshold: None,
            initialized,
            expires,
            outcome: VoteOutcome::Voting,
            ..Default::default()
        }
    }
    pub fn topic(&self) -> Option<Hash> {
        self.topic.clone()
    }
    pub fn in_favor(&self) -> Signal {
        self.in_favor
    }
    pub fn against(&self) -> Signal {
        self.against
    }
    pub fn turnout(&self) -> Signal {
        self.turnout
    }
    pub fn all_possible_turnout(&self) -> Signal {
        self.all_possible_turnout
    }
    pub fn expires(&self) -> Option<BlockNumber> {
        self.expires
    }
    pub fn passage_threshold(&self) -> ThresholdConfig<Signal> {
        self.passage_threshold
    }
    pub fn rejection_threshold(&self) -> Option<ThresholdConfig<Signal>> {
        self.rejection_threshold
    }
    pub fn outcome(&self) -> VoteOutcome {
        self.outcome
    }
    pub fn add_in_favor_vote(&self, magnitude: Signal) -> Self {
        let new_turnout = self.turnout() + magnitude;
        let new_in_favor = self.in_favor() + magnitude;
        VoteState {
            in_favor: new_in_favor,
            turnout: new_turnout,
            ..self.clone()
        }
    }
    pub fn add_against_vote(&self, magnitude: Signal) -> Self {
        let new_turnout = self.turnout() + magnitude;
        let new_against = self.against() + magnitude;
        VoteState {
            against: new_against,
            turnout: new_turnout,
            ..self.clone()
        }
    }
    pub fn add_abstention(&self, magnitude: Signal) -> Self {
        let new_turnout = self.turnout() + magnitude;
        VoteState {
            turnout: new_turnout,
            ..self.clone()
        }
    }
    pub fn remove_in_favor_vote(&self, magnitude: Signal) -> Self {
        let new_turnout = self.turnout() - magnitude;
        let new_in_favor = self.in_favor() - magnitude;
        VoteState {
            in_favor: new_in_favor,
            turnout: new_turnout,
            ..self.clone()
        }
    }
    pub fn remove_against_vote(&self, magnitude: Signal) -> Self {
        let new_turnout = self.turnout() - magnitude;
        let new_against = self.against() - magnitude;
        VoteState {
            against: new_against,
            turnout: new_turnout,
            ..self.clone()
        }
    }
    pub fn remove_abstention(&self, magnitude: Signal) -> Self {
        let new_turnout = self.turnout() - magnitude;
        VoteState {
            turnout: new_turnout,
            ..self.clone()
        }
    }
    pub fn update_topic_and_clear_state(&self, new_topic: Hash) -> Self {
        VoteState {
            in_favor: 0u32.into(),
            against: 0u32.into(),
            turnout: 0u32.into(),
            topic: Some(new_topic),
            ..self.clone()
        }
    }
    pub fn update_topic_without_clearing_state(&self, new_topic: Hash) -> Self {
        VoteState {
            topic: Some(new_topic),
            ..self.clone()
        }
    }
    // private method for setting the outcome, only accessible in the scope of this module
    fn set_outcome(&self, new_outcome: VoteOutcome) -> Self {
        VoteState {
            outcome: new_outcome,
            ..self.clone()
        }
    }
}

impl<
        Signal: Parameter
            + Copy
            + From<u32>
            + Default
            + PartialOrd
            + sp_std::ops::Add<Output = Signal>
            + sp_std::ops::Sub<Output = Signal>,
        BlockNumber: Parameter + Copy + Default,
        Hash: Clone,
    > Approved for VoteState<Signal, BlockNumber, Hash>
{
    fn approved(&self) -> bool {
        self.in_favor() > self.passage_threshold().support_threshold()
            && if let Some(turnout_threshold) = self.passage_threshold().turnout_threshold() {
                turnout_threshold > self.turnout()
            } else {
                true
            }
    }
}

impl<
        Signal: Parameter
            + Copy
            + From<u32>
            + Default
            + PartialOrd
            + sp_std::ops::Add<Output = Signal>
            + sp_std::ops::Sub<Output = Signal>,
        BlockNumber: Parameter + Copy + Default,
        Hash: Clone,
    > Rejected for VoteState<Signal, BlockNumber, Hash>
{
    fn rejected(&self) -> Option<bool> {
        if let Some(rejection_threshold_set) = self.rejection_threshold() {
            Some(
                self.against() > rejection_threshold_set.support_threshold()
                    && if let Some(turnout_threshold) = rejection_threshold_set.turnout_threshold()
                    {
                        turnout_threshold > self.turnout()
                    } else {
                        true
                    },
            )
        } else {
            // rejection threshold not set!
            None
        }
    }
}

impl<
        Signal: Parameter
            + Copy
            + From<u32>
            + Default
            + sp_std::ops::Add<Output = Signal>
            + sp_std::ops::Sub<Output = Signal>
            + PartialOrd,
        Hash: Clone,
        BlockNumber: Parameter + Copy + Default,
    > Apply<Vote<Signal, Hash>> for VoteState<Signal, BlockNumber, Hash>
{
    fn apply(&self, vote: Vote<Signal, Hash>) -> VoteState<Signal, BlockNumber, Hash> {
        let new_vote_state = match vote.direction() {
            VoterView::InFavor => self.add_in_favor_vote(vote.magnitude()),
            VoterView::Against => self.add_against_vote(vote.magnitude()),
            VoterView::Abstain => self.add_abstention(vote.magnitude()),
            _ => self.clone(),
        };
        let rejected = if let Some(rejection_outcome) = new_vote_state.rejected() {
            rejection_outcome
        } else {
            false
        }; // vacuously false
           // update outcome based on new vote state and return it
        if new_vote_state.approved() {
            new_vote_state.set_outcome(VoteOutcome::Approved)
        } else if rejected {
            new_vote_state.set_outcome(VoteOutcome::Rejected)
        } else {
            new_vote_state
        }
    }
}

impl<
        Signal: Parameter
            + Copy
            + From<u32>
            + Default
            + sp_std::ops::Add<Output = Signal>
            + sp_std::ops::Sub<Output = Signal>
            + PartialOrd,
        Hash: Clone,
        BlockNumber: Parameter + Copy + Default,
    > Revert<Vote<Signal, Hash>> for VoteState<Signal, BlockNumber, Hash>
{
    fn revert(&self, vote: Vote<Signal, Hash>) -> VoteState<Signal, BlockNumber, Hash> {
        let new_vote_state = match vote.direction() {
            VoterView::InFavor => self.remove_in_favor_vote(vote.magnitude()),
            VoterView::Against => self.remove_against_vote(vote.magnitude()),
            VoterView::Abstain => self.remove_abstention(vote.magnitude()),
            _ => self.clone(),
        };
        let rejected = if let Some(rejection_outcome) = new_vote_state.rejected() {
            rejection_outcome
        } else {
            false
        }; // vacuously false
           // update outcome based on new vote state and return it
        if new_vote_state.approved() {
            new_vote_state.set_outcome(VoteOutcome::Approved)
        } else if rejected {
            new_vote_state.set_outcome(VoteOutcome::Rejected)
        } else {
            new_vote_state
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, sp_runtime::RuntimeDebug)]
#[non_exhaustive]
/// The vote's state and outcome
pub enum VoteOutcome {
    /// The VoteId in question has been reserved but is not yet open for voting (context is schedule)
    NotStarted,
    /// The VoteState associated with the VoteId is open to voting by the given `ShareId`
    Voting,
    /// The VoteState is approved
    Approved,
    /// The VoteState is rejected
    Rejected,
}

impl Default for VoteOutcome {
    fn default() -> Self {
        VoteOutcome::NotStarted
    }
}
