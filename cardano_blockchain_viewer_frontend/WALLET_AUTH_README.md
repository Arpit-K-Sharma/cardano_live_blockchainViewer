# Cardano Wallet Authentication - Frontend Setup

This frontend now includes **CIP-30 Cardano wallet authentication** with support for multiple wallet providers.

## Supported Wallets

- ✅ **Eternl** (eternl.io)
- ✅ **Lace** (lace.io)
- ✅ **Typhon** (typhonwallet.io)
- ✅ **Yoroi** (yoroi-wallet.com)
- ✅ **Nami** (namiwallet.io)
- ✅ **Flint** (flint-wallet.com)

## Setup Instructions

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Environment Variables

Create a `.env.local` file:

```bash
cp .env.example .env.local
```

Edit `.env.local`:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080
```

### 3. Start the Development Server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### 4. Install a Cardano Wallet

If you don't have a wallet installed, install one of the supported wallets:

- **Eternl**: [Chrome Web Store](https://chrome.google.com/webstore/detail/eternl/kmhcihpebfmpgmihbkipmjlmmioameka)
- **Lace**: [lace.io](https://www.lace.io/)
- **Typhon**: [typhonwallet.io](https://typhonwallet.io/)
- **Yoroi**: [yoroi-wallet.com](https://yoroi-wallet.com/)

Make sure you're using the **PreProd testnet** in your wallet settings.

## How It Works

### Authentication Flow

1. **User visits the app** → Sees wallet connection screen
2. **User selects a wallet** → Browser extension opens
3. **User approves connection** → Wallet shares address
4. **Backend sends challenge** → Random nonce to sign
5. **User signs challenge** → Wallet creates signature (COSE_Sign1 + COSE_Key)
6. **Backend verifies signature** → Validates signature and address match
7. **JWT token issued** → User is authenticated
8. **Blockchain data loads** → WebSocket connection established

### Security Features

✅ **CIP-30 Compliant** - Follows Cardano wallet standard
✅ **COSE_Sign1 Parsing** - Properly extracts Ed25519 signatures
✅ **COSE_Key Parsing** - Securely extracts public keys
✅ **Address Verification** - Ensures public key matches address
✅ **Challenge-Response** - Prevents replay attacks
✅ **JWT Authentication** - Secure session management

## File Structure

```
cardano_blockchain_viewer_frontend/
├── app/
│   ├── layout.tsx           # AuthProvider wrapper
│   └── page.tsx             # Main app with auth gate
├── components/
│   ├── header.tsx           # Updated with wallet status
│   └── wallet-connect.tsx   # Wallet connection UI
├── lib/
│   ├── auth-context.tsx     # Authentication context & hook
│   ├── api.ts               # Backend API calls
│   └── cardano-wallet.ts    # CIP-30 wallet integration
└── .env.example             # Environment variables template
```

## Usage

### Connect Wallet

```tsx
import { WalletConnect } from '@/components/wallet-connect'

<WalletConnect onSuccess={() => console.log('Authenticated!')} />
```

### Use Auth State

```tsx
import { useAuth } from '@/lib/auth-context'

function MyComponent() {
  const { isAuthenticated, address, walletName, logout } = useAuth()

  if (!isAuthenticated) {
    return <div>Please connect your wallet</div>
  }

  return (
    <div>
      <p>Connected: {walletName}</p>
      <p>Address: {address}</p>
      <button onClick={logout}>Disconnect</button>
    </div>
  )
}
```

### Protected Routes

The app automatically shows the wallet connection screen when not authenticated. Once authenticated, users can access the blockchain viewer.

## API Endpoints

The frontend calls these backend endpoints:

- `POST /api/auth/challenge` - Request authentication challenge
- `POST /api/auth/verify` - Verify signature and get JWT

## Troubleshooting

### Wallet not detected

1. Make sure you've installed a wallet extension
2. Refresh the page after installing
3. Check that the wallet is enabled in your browser

### Connection failed

1. Verify the backend is running on `http://localhost:8080`
2. Check the browser console for error messages
3. Ensure wallet is set to PreProd testnet

### Signature verification failed

1. Make sure you're signing with the same address that requested the challenge
2. Try disconnecting and reconnecting your wallet
3. Check backend logs for detailed error messages

## Development

### Test Wallet Authentication

```bash
# Start backend (in backend directory)
cd ../cardano_blockchain_viewer
cargo run

# Start frontend (in this directory)
npm run dev
```

### Debug Mode

Open browser DevTools console to see authentication flow logs:

- 🔌 Wallet connection
- 🔑 Challenge request
- ✍️ Signature creation
- 🔐 Verification
- ✅ Authentication success

## Production Deployment

### Environment Variables

Set these in your production environment:

```env
NEXT_PUBLIC_API_URL=https://your-api-domain.com
```

### Build

```bash
npm run build
npm start
```

## CIP-30 Resources

- [CIP-30 Specification](https://cips.cardano.org/cip/CIP-0030)
- [CIP-8 Message Signing](https://cips.cardano.org/cip/CIP-8)
- [COSE (RFC 8152)](https://datatracker.ietf.org/doc/html/rfc8152)
