# Cardano Blockchain Viewer

A real-time Cardano blockchain explorer with optional wallet authentication. View live blockchain data from the PreProd testnet and connect your wallet for personalized features.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Next.js](https://img.shields.io/badge/next.js-16.0.7-black.svg)

[![Deploy on Railway](https://img.shields.io/badge/Railway-Staging-purple?logo=railway)](cardanoliveblockchainviewer-production-f3c2.up.railway.app/)
[![Deploy on Vercel](https://img.shields.io/badge/Vercel-Production-black?logo=vercel)](https://cardano-live-blockchain-viewer.vercel.app/)



## 🌟 Features

### Public Access (No Authentication Required)
- ✅ **Live Blockchain Viewer** - Real-time block and transaction updates
- ✅ **Dashboard View** - Overview of recent blocks and transactions
- ✅ **Block Explorer** - Detailed view of blocks with full transaction data
- ✅ **WebSocket Connection** - Live streaming blockchain events

### Authenticated Access (Wallet Required)
- 🔐 **CIP-30 Wallet Authentication** - Secure signature-based login
- 🔐 **Transaction History** - View your personal transaction history
- 🔐 **Portfolio Summary** - Check your wallet balance and activity
- 🔐 **Multi-Wallet Support** - Works with Eternl, Lace, Typhon, Yoroi, Nami, and Flint

## 📋 Table of Contents

- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Backend Setup](#backend-setup)
  - [Frontend Setup](#frontend-setup)
- [Configuration](#configuration)
- [Running the Application](#running-the-application)
- [Usage Guide](#usage-guide)
- [API Documentation](#api-documentation)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend                             │
│  (Next.js 16 + React 19 + TypeScript + Tailwind CSS)       │
│                                                              │
│  • Dashboard View (Recent Blocks & Transactions)            │
│  • Block Explorer (Detailed Block/TX Data)                  │
│  • Wallet Authentication (CIP-30)                           │
│  • User Transaction History                                 │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   │ WebSocket (Real-time Events)
                   │ REST API (Authentication & Data)
                   │
┌──────────────────▼──────────────────────────────────────────┐
│                         Backend                              │
│            (Rust + Axum + Tokio + Oura)                     │
│                                                              │
│  • WebSocket Server (Blockchain Events)                     │
│  • REST API (Auth & User Data)                              │
│  • JWT Authentication                                        │
│  • Blockfrost Integration                                   │
│  • CIP-30 Signature Verification                            │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   │ Oura (Blockchain Data Streaming)
                   │
┌──────────────────▼──────────────────────────────────────────┐
│              Cardano PreProd Testnet                        │
│         (preprod-node.world.dev.cardano.org)                │
└─────────────────────────────────────────────────────────────┘
```

## 📦 Prerequisites

### Backend Requirements
- **Rust** 1.70 or higher ([Install Rust](https://www.rust-lang.org/tools/install))
- **Oura** ([Install Oura](https://github.com/txpipe/oura))
- **Blockfrost API Key** ([Get API Key](https://blockfrost.io/))

### Frontend Requirements
- **Node.js** 20.9.0 or higher ([Install Node.js](https://nodejs.org/))
- **npm** or **yarn** package manager

### Optional
- **Cardano Wallet Extension** (Eternl, Lace, Typhon, Yoroi, Nami, or Flint)

## 🚀 Installation

### Backend Setup

#### 1. Navigate to Backend Directory
```bash
cd cardano_blockchain_viewer
```

#### 2. Install Oura
Oura is required for streaming blockchain data.

**macOS (Homebrew):**
```bash
brew install txpipe/tap/oura
```

**Linux/Unix:**
```bash
# Download latest release
curl -L https://github.com/txpipe/oura/releases/latest/download/oura-x86_64-unknown-linux-gnu.tar.gz -o oura.tar.gz

# Extract and install
tar -xzf oura.tar.gz
sudo mv oura /usr/local/bin/
```

**Verify Installation:**
```bash
oura --version
```

#### 3. Install Rust Dependencies
```bash
cargo build --release
```

#### 4. Configure Environment Variables
Create a `.env` file in the `cardano_blockchain_viewer` directory:

```env
# JWT Secret (change this in production!)
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production

# Blockfrost API Key (PreProd Network)
BLOCKFROST_API_KEY=your-blockfrost-api-key-here

# Optional: Logging level
RUST_LOG=info
```

**Get Your Blockfrost API Key:**
1. Visit [blockfrost.io](https://blockfrost.io/)
2. Sign up for a free account
3. Create a new project for **PreProd** network
4. Copy the API key and paste it in the `.env` file

### Frontend Setup

#### 1. Navigate to Frontend Directory
```bash
cd cardano_blockchain_viewer_frontend
```

#### 2. Install Dependencies
```bash
npm install
# or
yarn install
```

#### 3. Configure Environment Variables
Create a `.env.local` file:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080
```

## ⚙️ Configuration

### Backend Configuration

#### Network Configuration
The backend is configured for PreProd testnet by default. To change networks, edit `src/config.rs`:

```rust
impl Default for CardanoConfig {
    fn default() -> Self {
        Self::preprod()  // Options: preprod(), preview(), mainnet()
    }
}
```

#### Server Settings
Default server address is `127.0.0.1:8080`. Change in `src/config.rs`:

```rust
pub const SERVER_ADDR: &str = "127.0.0.1:8080";
```

### Frontend Configuration

#### API Endpoints
Configure API endpoints in `.env.local`:

```env
# Backend API base URL
NEXT_PUBLIC_API_URL=http://localhost:8080

# WebSocket URL for live blockchain data
NEXT_PUBLIC_WS_URL=ws://localhost:8080
```

## 🏃 Running the Application

### Start Backend Server

```bash
cd cardano_blockchain_viewer
cargo run --release
```

**Expected Output:**
```
INFO  Starting Cardano Blockchain Viewer Backend
INFO  Network: PreProd Testnet
INFO  🔐 JWT Manager initialized
INFO  🌐 Blockfrost client initialized (preprod network)
INFO  🌍 Server starting on: http://127.0.0.1:8080
INFO     REST API Endpoints:
INFO     - POST http://127.0.0.1:8080/api/auth/challenge
INFO     - POST http://127.0.0.1:8080/api/auth/verify
INFO     - GET  http://127.0.0.1:8080/api/user/transactions (protected)
INFO     - GET  http://127.0.0.1:8080/api/user/summary (protected)
INFO     WebSocket Endpoint:
INFO     - ws://127.0.0.1:8080/ws
```

### Start Frontend Development Server

```bash
cd cardano_blockchain_viewer_frontend
npm run dev
```

**Expected Output:**
```
   ▲ Next.js 16.0.7
   - Local:        http://localhost:3000
   - Network:      http://192.168.1.x:3000

 ✓ Starting...
 ✓ Ready in 2.3s
```

### Access the Application

Open your browser and navigate to:
- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8080
- **WebSocket**: ws://localhost:8080/ws

## 📖 Usage Guide

### Viewing the Blockchain (Public Access)

1. **Open the Application**
   - Navigate to http://localhost:3000
   - The dashboard loads automatically with live blockchain data

2. **Dashboard View**
   - See the 10 most recent blocks
   - View the 10 most recent transactions
   - Real-time updates via WebSocket
   - Click any block or transaction to view on CardanoScan

3. **Explorer View**
   - Click "Explorer" tab in the header
   - View detailed block information
   - Expand blocks to see all transactions
   - Expand transactions to see inputs/outputs
   - Navigate through pages of blocks

### Connecting Your Wallet (Authentication)

1. **Install a Wallet Extension**
   - Install one of the supported wallets (Eternl, Lace, Typhon, Yoroi, Nami, or Flint)
   - Make sure it's set to **PreProd testnet**

2. **Connect Wallet**
   - Click "Connect Wallet" button in the header
   - Select your wallet from the dialog
   - Approve the connection in your wallet extension
   - Sign the authentication challenge
   - Wait for verification

3. **Authentication Success**
   - Your wallet address appears in the header
   - "My Transactions" tab becomes available
   - You can now access personalized features

### Viewing Your Transactions (Authenticated)

1. **Navigate to My Transactions**
   - Click "My Transactions" tab in the header
   - View your wallet summary (balance, transaction count)
   - See your complete transaction history

2. **Transaction Details**
   - Each transaction shows:
     - Transaction hash
     - Block number and slot
     - Fee amount
     - Timestamp
   - Click "CardanoScan →" to view on block explorer

3. **Load More Transactions**
   - Transactions are paginated (20 per page)
   - Click "Load More Transactions" button at the bottom
   - Scroll through your complete history

### Disconnecting Wallet

- Click "Disconnect" button in the header
- Your authentication session ends
- Public blockchain viewing remains available

## 📡 API Documentation

### Public Endpoints

#### POST /api/auth/challenge
Request a challenge message for wallet authentication.

**Request:**
```json
{
  "address": "addr_test1..."
}
```

**Response:**
```json
{
  "message": "Sign this message to authenticate...",
  "nonce": "1234567890"
}
```

#### POST /api/auth/verify
Verify wallet signature and receive JWT token.

**Request:**
```json
{
  "address": "addr_test1...",
  "signature": "84a401276761646472657373...",
  "key": "a4010382026658...",
  "stake_address": "stake_test1..."
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "address": "addr_test1..."
}
```

### Protected Endpoints (Require JWT)

#### GET /api/user/transactions
Get user's transaction history.

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Query Parameters:**
- `address` (required): Wallet address
- `page` (optional): Page number (default: 1)
- `count` (optional): Items per page (default: 20)

**Response:**
```json
{
  "transactions": [
    {
      "tx_hash": "abc123...",
      "block": "block_hash...",
      "block_height": 12345,
      "block_time": 1702800000,
      "slot": 54321,
      "index": 0,
      "fees": "170000"
    }
  ],
  "total": 50,
  "page": 1
}
```

#### GET /api/user/summary
Get user's wallet summary.

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Query Parameters:**
- `address` (required): Wallet address

**Response:**
```json
{
  "address": "addr_test1...",
  "stake_address": "stake_test1...",
  "balance": "10000000",
  "transaction_count": 50
}
```

### WebSocket Endpoint

#### WS /ws
Real-time blockchain event stream.

**Connection:**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
```

**Message Types:**

**Block Event:**
```json
{
  "type": "Block",
  "slot": 54321,
  "hash": "abc123...",
  "number": 12345,
  "epoch": 100,
  "tx_count": 5,
  "timestamp": 1702800000
}
```

**Transaction Event:**
```json
{
  "type": "Transaction",
  "hash": "def456...",
  "fee": 170000,
  "inputs": 2,
  "outputs": 3,
  "total_output": 5000000,
  "timestamp": 1702800000
}
```

**Statistics Event:**
```json
{
  "type": "stats",
  "data": {
    "total_events": 1000,
    "blocks_count": 100,
    "transactions_count": 500,
    "inputs_count": 1000,
    "outputs_count": 1500,
    "buffer_size": 100,
    "last_block_number": 12345,
    "last_slot": 54321
  }
}
```

## 🔧 Troubleshooting

### Backend Issues

#### Oura Not Found
```
Error: oura: command not found
```
**Solution:** Install Oura following the [installation instructions](#2-install-oura)

#### Blockfrost API Error
```
Error: Blockfrost returned HTML instead of JSON
```
**Solution:** 
- Check your `BLOCKFROST_API_KEY` in `.env`
- Verify the key is for PreProd network
- Ensure the key is active in your Blockfrost dashboard

#### Port Already in Use
```
Error: Address already in use (os error 48)
```
**Solution:** 
- Stop any other service using port 8080
- Or change `SERVER_ADDR` in `src/config.rs`

### Frontend Issues

#### Cannot Connect to Backend
```
Error: Failed to fetch
```
**Solution:**
- Verify backend is running on `http://localhost:8080`
- Check `NEXT_PUBLIC_API_URL` in `.env.local`
- Ensure no firewall is blocking connections

#### WebSocket Connection Failed
```
WebSocket connection to 'ws://localhost:8080/ws' failed
```
**Solution:**
- Confirm backend server is running
- Check `NEXT_PUBLIC_WS_URL` in `.env.local`
- Try refreshing the page

### Wallet Authentication Issues

#### Wallet Not Detected
**Solution:**
- Install a supported wallet extension
- Refresh the page after installation
- Check that the wallet extension is enabled

#### Signature Verification Failed
**Solution:**
- Ensure wallet is set to PreProd testnet
- Try disconnecting and reconnecting the wallet
- Check backend logs for detailed error messages
- Make sure you're using the latest wallet extension version

#### No Transactions Found
**Solution:**
- Verify your wallet has transactions on PreProd testnet
- Check that your wallet address is correct
- Confirm Blockfrost API key is working
- Try viewing your address on CardanoScan PreProd

## 🛠️ Development

### Building for Production

**Backend:**
```bash
cd cardano_blockchain_viewer
cargo build --release
./target/release/cardano_blockchain_viewer
```

**Frontend:**
```bash
cd cardano_blockchain_viewer_frontend
npm run build
npm start
```

### Running Tests

**Backend:**
```bash
cd cardano_blockchain_viewer
cargo test
```

**Frontend:**
```bash
cd cardano_blockchain_viewer_frontend
npm test
```

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [Cardano](https://cardano.org/) - The blockchain platform
- [Oura](https://github.com/txpipe/oura) - Data streaming pipeline
- [Blockfrost](https://blockfrost.io/) - Cardano API service
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Next.js](https://nextjs.org/) - React framework
- [Tailwind CSS](https://tailwindcss.com/) - CSS framework

## 📞 Support

For issues, questions, or suggestions:
- Open an issue on GitHub
- Check existing documentation
- Review troubleshooting guide

---

**Built with ❤️ for the Cardano community**