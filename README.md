# Rust-Based Distributed State Management System

This project illustrates a basic implementation of a distributed state management system using Rust. It demonstrates how nodes communicate, propose state changes, and reach a consensus in a distributed network.

## Project Summary

The system models a network of nodes where each node can propose changes to its state and achieve consensus with other nodes through a simple protocol. This project aims to provide a foundational understanding of distributed systems, including node interactions, message exchanges, and consensus protocols.

## Key Features

- **TCP Node Communication**: Establishes TCP connections for nodes to exchange messages.
- **Proposal Handling**: Nodes can propose state changes and handle proposals from other nodes.
- **Acknowledgment System**: Processes and tracks acknowledgments for received proposals.
- **Consensus Protocol**: Uses a majority-based approach to commit proposals once a sufficient number of acknowledgments are received.
- **State Updates**: Nodes update their state based on proposals and consensus.

## Getting Started

### Prerequisites

To get started with this project, you will need:

- **Rust Programming Language**: Ensure that Rust is installed on your machine. You can download it from [the Rust official site](https://www.rust-lang.org/learn/get-started).

### Installation

1. **Clone the Repository**:

    ```sh
    git clone https://github.com/juliog922/state_machine.git
    cd state_machine
    ```

2. **Build the Project**:

    ```sh
    cargo build
    ```

3. **Run the Application**:

    Start the application using:

    ```sh
    cargo run
    ```

    This command initiates two nodes that will simulate proposal broadcasting and consensus.

### System Components

The project includes the following main components:

- **Node**: Represents an individual node in the network. It handles sending and receiving messages, and processes state changes based on proposals.

- **Message**: Defines the format of messages exchanged between nodes. This includes proposals, acknowledgments, and commit messages.

- **State**: An enumeration describing the possible states a node can be in, such as `Init`, `Running`, or `Stopped`.

- **MessageType**: An enumeration for different message types like `Proposal`, `Acknowledgment`, and `Commit`.

### Running Tests

To verify the functionality of the system, run the provided tests:

```sh
cargo test