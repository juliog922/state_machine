use serde::{Deserialize, Serialize};
use super::state::State;

/// Represents the type of message being sent between nodes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MessageType {
    /// A proposal message indicating that a node is proposing a state change.
    Proposal,

    /// An acknowledgment message indicating that a node has acknowledged a proposal.
    Acknowledgment,

    /// A commit message indicating that a proposal has been committed.
    Commit,
}

/// Represents a message exchanged between nodes in the system.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Message {
    /// The unique identifier of the sender node.
    pub sender_id: u64,

    /// The type of the message (e.g., Proposal, Acknowledgment, Commit).
    pub message_type: MessageType,

    /// The state that is being proposed or acknowledged.
    pub proposed_state: State,

    /// The unique identifier of the proposal.
    pub proposal_id: String,
}
