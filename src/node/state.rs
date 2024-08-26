use serde::{Deserialize, Serialize};

/// Represents the state of a node in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum State {
    /// The node has just been initialized and is not yet running any processes.
    Init,

    /// The node is actively running and processing proposals.
    Running,

    /// The node has been stopped and is not processing any proposals.
    Stopped,
}