mod node;
mod tests;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use std::collections::HashMap;
use tokio::time::Duration;
use node::{Node, state::State};


/// Main function for running the node communication simulation.
///
/// This function initializes two nodes, sets up their communication channels, and simulates
/// the process of broadcasting a proposal and handling acknowledgments between nodes.
#[tokio::main]
async fn main() {
    // Initialize shared state and proposal acknowledgments
    let state = Arc::new(Mutex::new(State::Init));
    let proposal_acknowledgments = Arc::new(Mutex::new(HashMap::new()));

    // Create channels for message passing
    let (tx1, rx1) = mpsc::channel(32);
    let node1 = Arc::new(Node {
        id: 1,
        state: state.clone(),
        peers: HashMap::from([(2, "127.0.0.1:8081".to_string())]), // Peer node2's address
        address: "127.0.0.1:8080".to_string(), // Node1's address
        tx: tx1,
        proposal_acknowledgments: proposal_acknowledgments.clone(),
    });

    let (tx2, rx2) = mpsc::channel(32);
    let node2 = Arc::new(Node {
        id: 2,
        state: state.clone(),
        peers: HashMap::from([(1, "127.0.0.1:8080".to_string())]), // Peer node1's address
        address: "127.0.0.1:8081".to_string(), // Node2's address
        tx: tx2,
        proposal_acknowledgments,
    });

    // Spawn tasks for handling incoming messages for each node
    let node1_clone_for_messages = Arc::clone(&node1);
    tokio::spawn(async move {
        node1_clone_for_messages.handle_incoming_messages(rx1).await;
    });

    let node2_clone_for_messages = Arc::clone(&node2);
    tokio::spawn(async move {
        node2_clone_for_messages.handle_incoming_messages(rx2).await;
    });

    // Spawn tasks to listen for incoming connections for each node
    let node1_clone_for_listen = Arc::clone(&node1);
    tokio::spawn(async move {
        node1_clone_for_listen.listen().await.expect("Node 1 failed to listen");
    });

    let node2_clone_for_listen = Arc::clone(&node2);
    tokio::spawn(async move {
        node2_clone_for_listen.listen().await.expect("Node 2 failed to listen");
    });

    // Ensure the servers have time to start up and bind to their addresses
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Use node1 to broadcast a proposal
    let proposal_id = node1.broadcast_proposal(State::Running).await;

    // Wait for acknowledgments for the proposal
    node1.wait_for_acknowledgments(proposal_id).await;

    // Allow additional time for Node 2 to process the acknowledgment and update its state
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check if Node 2 has updated its state to the proposed state
    let state = node2.state.lock().await;
    assert_eq!(*state, State::Running);

    // Print success message if the communication was successful
    println!("Communication completed successfully!");
}
