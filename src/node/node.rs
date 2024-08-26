use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use uuid::Uuid;

use super::message::{Message, MessageType};
use super::state::State;

/// Represents a node in the distributed system.
pub struct Node {
    /// Unique identifier for the node.
    pub id: u64,

    /// The current state of the node, wrapped in an `Arc` and `Mutex` for concurrency.
    pub state: Arc<Mutex<State>>,

    /// Map of peer node IDs to their network addresses.
    pub peers: HashMap<u64, String>,

    /// Address the node is listening on for incoming connections.
    pub address: String,

    /// Channel sender used to send messages to the node's message handler.
    pub tx: mpsc::Sender<Message>,

    /// Tracks acknowledgments for each proposal, with `proposal_id` as the key and a set of node IDs as the value.
    pub proposal_acknowledgments: Arc<Mutex<HashMap<String, HashSet<u64>>>>,
}

impl Node {
    /// Sends a message to a specified address.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be sent.
    /// * `address` - The destination address.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result<()>` indicating success or failure.
    pub async fn send_message(&self, message: &Message, address: &str) -> io::Result<()> {
        let mut stream = TcpStream::connect(address).await?;
        let serialized_message = serde_json::to_vec(message)?;
        stream.write_all(&serialized_message).await?;
        stream.flush().await
    }

    /// Broadcasts a proposal to all peer nodes.
    ///
    /// # Arguments
    ///
    /// * `new_state` - The proposed state to be broadcasted.
    ///
    /// # Returns
    ///
    /// Returns the `proposal_id` of the broadcasted proposal.
    pub async fn broadcast_proposal(&self, new_state: State) -> String {
        let proposal_id = Uuid::new_v4().to_string();
        let proposal_message = Message {
            sender_id: self.id,
            message_type: MessageType::Proposal,
            proposed_state: new_state,
            proposal_id: proposal_id.clone(),
        };

        for peer_address in self.peers.values() {
            if let Err(e) = self.send_message(&proposal_message, peer_address).await {
                eprintln!("Failed to send proposal: {:?}", e);
            }
        }

        println!("Node {} broadcasted the proposal: {}", self.id, proposal_id);

        proposal_id
    }

    /// Waits for acknowledgments of a proposal and commits it if a majority is reached.
    ///
    /// # Arguments
    ///
    /// * `proposal_id` - The unique identifier of the proposal to wait for.
    pub async fn wait_for_acknowledgments(&self, proposal_id: String) {
        let majority = (self.peers.len() / 2) + 1; // Simple majority

        loop {
            let ack_count = {
                let acks = self.proposal_acknowledgments.lock().await;
                acks.get(&proposal_id)
                    .map(|acks| acks.len())
                    .unwrap_or(0)
            };

            if ack_count >= majority {
                // Create and send commit message
                let commit_message = Message {
                    sender_id: self.id,
                    message_type: MessageType::Commit,
                    proposed_state: State::Running, // This should match the state proposed earlier
                    proposal_id: proposal_id.clone(),
                };

                for address in self.peers.values() {
                    self.send_message(&commit_message, address).await.unwrap();
                }

                println!("Node {} committed the proposal: {}", self.id, proposal_id);
                break;
            }

            // Sleep briefly before checking again
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Handles incoming messages from a receiver.
    ///
    /// # Arguments
    ///
    /// * `receiver` - The channel receiver used to receive messages.
    pub async fn handle_incoming_messages(&self, mut receiver: mpsc::Receiver<Message>) {
        while let Some(message) = receiver.recv().await {
            match message.message_type {
                MessageType::Proposal => {
                    // Handle proposal
                    println!("Node {} received proposal: {:?}", self.id, message);
                    let ack_message = Message {
                        sender_id: self.id,
                        message_type: MessageType::Acknowledgment,
                        proposed_state: message.proposed_state.clone(),
                        proposal_id: message.proposal_id.clone(),
                    };
                    if let Err(e) = self.send_message(&ack_message, &self.peers[&message.sender_id]).await {
                        eprintln!("Failed to send acknowledgment: {:?}", e);
                    }

                    // Update the state to the proposed state
                    let mut state = self.state.lock().await;
                    *state = message.proposed_state;
                    println!("Node {} updated state to {:?}", self.id, *state);
                }
                MessageType::Acknowledgment => {
                    // Handle acknowledgment
                    println!("Node {} received acknowledgment: {:?}", self.id, message);
                    let mut acks = self.proposal_acknowledgments.lock().await;
                    if let Some(ack_set) = acks.get_mut(&message.proposal_id) {
                        ack_set.insert(message.sender_id);
                    } else {
                        let mut new_ack_set = HashSet::new();
                        new_ack_set.insert(message.sender_id);
                        acks.insert(message.proposal_id.clone(), new_ack_set);
                    }
                }
                _ => {}
            }
        }
    }

    /// Listens for incoming connections and processes them.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result<()>` indicating success or failure.
    pub async fn listen(&self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.address).await?;
        println!("Node {} listening on {}", self.id, self.address);

        loop {
            let (mut socket, _) = listener.accept().await?;

            let tx = self.tx.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            println!("Connection closed");
                            break; // Connection was closed
                        }
                        Ok(n) => {
                            if let Ok(message) = serde_json::from_slice::<Message>(&buf[..n]) {
                                tx.send(message).await.expect("Failed to send message to channel");
                            } else {
                                println!("Failed to deserialize message");
                            }
                        }
                        Err(e) => {
                            println!("Failed to read from socket: {:?}", e);
                            break;
                        }
                    }
                }
            });
        }
    }
}
