# Authentication Flow - Cardano Blockchain Viewer

## Overview

The Cardano Blockchain Viewer now has **optional wallet authentication**. Users can view the live blockchain without connecting their wallet, but wallet connection unlocks personalized features.

## Key Features

### Public Access (No Login Required)
- ✅ Live blockchain viewer (Dashboard & Explorer views)
- ✅ Real-time block and transaction updates via WebSocket
- ✅ View blockchain statistics
- ✅ Browse recent blocks and transactions

### Authenticated Access (Wallet Required)
- 🔐 User transaction history
- 🔐 Portfolio summary
- 🔐 Personalized analytics (future feature)

## How It Works

### 1. Public Blockchain Viewing
- Users can immediately view the live Cardano blockchain without any authentication
- WebSocket connection to `ws://localhost:8080` is established automatically
- All blockchain data (blocks, transactions, stats) is publicly accessible

### 2. Optional Wallet Connection
- Users can click **"Connect Wallet"** button in the header to authenticate
- A dialog appears showing available Cardano wallets (Eternl, Lace, Typhon, Yoroi, Nami, Flint)
- Connection process:
  1. User selects a wallet
  2. Wallet extension prompts for permission
  3. Backend generates a challenge message
  4. User signs the challenge with their wallet
  5. Backend verifies the signature and issues a JWT token
  6. User is now authenticated

### 3. Authenticated Features
When authenticated, users get access to:
- **My Wallet** view (future implementation)
- **Transaction History** for their wallet address
- **Portfolio Summary** with balance and assets

## Technical Implementation

### Frontend Structure

#### [app/page.tsx](app/page.tsx)
- Main application page
- WebSocket connection is **always** established (no authentication required)
- Renders Dashboard or Explorer view based on user selection
- No authentication wall - blockchain viewer is public

#### [components/header.tsx](components/header.tsx)
- Navigation bar with Dashboard and Explorer tabs (always visible)
- Shows "Connect Wallet" button when not authenticated
- Shows wallet info and "Disconnect" button when authenticated
- Wallet connection happens via a dialog modal

#### [components/wallet-connect.tsx](components/wallet-connect.tsx)
- Handles wallet connection flow
- Supports `compact` mode for use in dialogs
- Detects available Cardano wallets
- Manages authentication process with backend

#### [lib/auth-context.tsx](lib/auth-context.tsx)
- React context for authentication state
- Stores: wallet address, stake address, JWT token, wallet name
- Persists authentication to localStorage
- Provides `login()` and `logout()` functions

### Backend API

#### Authentication Endpoints
- `POST /api/auth/challenge` - Request a challenge message to sign
- `POST /api/auth/verify` - Verify signature and receive JWT token

#### Protected Routes (require JWT)
- `GET /api/user/transactions` - Get user's transaction history
- `GET /api/user/summary` - Get user's portfolio summary

## User Experience Flow

### First Visit (No Wallet)
```
1. User opens application
2. Sees live blockchain dashboard immediately
3. Can switch between Dashboard and Explorer views
4. "Connect Wallet" button is visible but optional
```

### With Wallet Connection
```
1. User opens application
2. Sees live blockchain dashboard
3. Clicks "Connect Wallet" button
4. Selects wallet from dialog
5. Signs authentication message
6. Dialog closes, wallet info appears in header
7. Can now access "My Wallet" features (future)
```

### Returning User (Previously Connected)
```
1. User opens application
2. JWT token loaded from localStorage
3. Wallet info appears in header automatically
4. User is pre-authenticated for personalized features
5. Can still view public blockchain data
```

## Future Enhancements

### Planned Features
1. **My Wallet View**
   - Add a third view tab: Dashboard | Explorer | My Wallet
   - Only visible when authenticated
   - Shows user's transaction history and portfolio

2. **Transaction History**
   - Paginated list of user's transactions
   - Filter and search capabilities
   - Export to CSV/JSON

3. **Portfolio Summary**
   - ADA balance
   - Native tokens/assets
   - Staking information
   - NFT holdings

4. **Notifications**
   - Alert user when their address is involved in a transaction
   - Real-time updates for user's wallet activity

## Security Considerations

- ✅ Wallet signature proves ownership without exposing private keys
- ✅ JWT tokens stored in localStorage (consider httpOnly cookies for production)
- ✅ Backend validates all signatures before issuing tokens
- ✅ Wallet connection is client-side only (no backend wallet access)
- ⚠️ For production: implement token refresh and expiration handling

## Development Notes

### Running the Application

```bash
# Frontend
cd cardano_blockchain_viewer_frontend
npm install
npm run dev
# Runs on http://localhost:3000

# Backend
cd cardano_blockchain_viewer
cargo run
# WebSocket on ws://localhost:8080
# API on http://localhost:8080
```

### Testing Without Wallet
- Install a Cardano wallet extension (Eternl, Lace, etc.)
- Make sure it's connected to PreProd testnet
- Alternatively, test public features without installing a wallet

### Environment Variables
```env
# Backend
RUST_LOG=debug
# Add other backend config as needed

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080
```

## Summary

The Cardano Blockchain Viewer is now a **public-first** application where:
- **Everyone** can view the live blockchain
- **Wallet connection is optional** for personalized features
- **Authentication is seamless** via wallet signatures
- **User experience is smooth** with no barriers to entry

This architecture allows the application to serve both casual viewers and authenticated users effectively.
