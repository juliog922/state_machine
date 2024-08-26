#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::{mpsc, Mutex};
    use std::collections::HashMap;
    use tokio::time::Duration;
    use tokio::net::TcpListener;
    use crate::node::{Node, state::State, message::{Message, MessageType}};
    use tokio::io;
    use uuid::Uuid;

    /// Sets up two nodes for testing, including their state, peers, and channels.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing two `Arc<Node>` instances representing the nodes.
    async fn setup_nodes() -> (Arc<Node>, Arc<Node>) {
        // Initialize state and proposal acknowledgments for nodes
        let state = Arc::new(Mutex::new(State::Init));
        let proposal_acknowledgments = Arc::new(Mutex::new(HashMap::new()));

        // Create channels for message passing
        let (tx1, rx1) = mpsc::channel(32);
        let (tx2, rx2) = mpsc::channel(32);

        // Create nodes with placeholder addresses
        let node1 = Node {
            id: 1,
            state: state.clone(),
            peers: HashMap::new(), // Peers will be updated after binding
            address: "127.0.0.1:0".to_string(), // Placeholder address
            tx: tx1.clone(),
            proposal_acknowledgments: proposal_acknowledgments.clone(),
        };

        let node2 = Node {
            id: 2,
            state: state.clone(),
            peers: HashMap::new(), // Peers will be updated after binding
            address: "127.0.0.1:0".to_string(), // Placeholder address
            tx: tx2.clone(),
            proposal_acknowledgments: proposal_acknowledgments.clone(),
        };

        // Bind listeners to get actual port numbers
        let listener1 = TcpListener::bind(&node1.address).await.unwrap();
        let listener2 = TcpListener::bind(&node2.address).await.unwrap();

        // Update nodes with actual addresses and ports
        let node1_address = listener1.local_addr().unwrap().to_string();
        let node2_address = listener2.local_addr().unwrap().to_string();

        let node1 = Node {
            id: 1,
            state: state.clone(),
            peers: HashMap::from([(2, node2_address.clone())]),
            address: node1_address.clone(),
            tx: tx1,
            proposal_acknowledgments: proposal_acknowledgments.clone(),
        };

        let node2 = Node {
            id: 2,
            state: state.clone(),
            peers: HashMap::from([(1, node1_address)]),
            address: node2_address,
            tx: tx2,
            proposal_acknowledgments,
        };

        // Recreate Arcs for nodes
        let node1 = Arc::new(node1);
        let node2 = Arc::new(node2);

        // Spawn tasks to handle incoming messages for each node
        let node1_clone_for_messages = Arc::clone(&node1);
        let node2_clone_for_messages = Arc::clone(&node2);
        let rx1_clone = rx1; // Pass directly, no cloning needed
        let rx2_clone = rx2; // Pass directly, no cloning needed
        tokio::spawn(async move {
            node1_clone_for_messages.handle_incoming_messages(rx1_clone).await;
        });
        tokio::spawn(async move {
            node2_clone_for_messages.handle_incoming_messages(rx2_clone).await;
        });

        // Spawn tasks to listen for incoming connections for each node
        let node1_clone_for_listen = Arc::clone(&node1);
        let node2_clone_for_listen = Arc::clone(&node2);
        tokio::spawn(async move {
            node1_clone_for_listen.listen().await.expect("Node 1 failed to listen");
        });
        tokio::spawn(async move {
            node2_clone_for_listen.listen().await.expect("Node 2 failed to listen");
        });

        (node1, node2)
    }

    /// Tests the communication between two nodes, ensuring that a proposal sent from node1 is received and processed by node2.
    #[tokio::test]
    async fn test_node_communication() -> io::Result<()> {
        let (node1, node2) = setup_nodes().await;

        // Allow time for servers to start up
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Broadcast a proposal from node1
        let proposal_id = node1.broadcast_proposal(State::Running).await;

        // Wait for the proposal to be acknowledged
        node1.wait_for_acknowledgments(proposal_id).await;

        // Allow additional time for message processing
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify that node2 has received and processed the proposal
        let state = node2.state.lock().await;
        assert_eq!(*state, State::Running);

        Ok(())
    }

    /// Tests the acknowledgment mechanism by sending a proposal from node1 to node2 and verifying that node2 acknowledges it.
    #[tokio::test]
    async fn test_acknowledgment() -> io::Result<()> {
        let (node1, node2) = setup_nodes().await;

        // Allow time for servers to start up
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Create a proposal message
        let proposal_id = Uuid::new_v4().to_string();
        let proposal_message = Message {
            sender_id: 1,
            message_type: MessageType::Proposal,
            proposed_state: State::Running,
            proposal_id: proposal_id.clone(),
        };

        // Send the proposal from node1 to node2
        node1.send_message(&proposal_message, &node2.address).await?;

        // Allow time for the message to be processed
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Wait for the acknowledgment of the proposal
        node1.wait_for_acknowledgments(proposal_id.clone()).await;

        // Verify that node2 has acknowledged the proposal
        let acknowledgments = node2.proposal_acknowledgments.lock().await;
        let acks = acknowledgments.get(&proposal_id);
        if acks.is_none() || acks.unwrap().is_empty() {
            eprintln!("No acknowledgments found for proposal_id: {}", proposal_id);
        }
        assert!(acks.is_some() && !acks.unwrap().is_empty());

        Ok(())
    }
}
