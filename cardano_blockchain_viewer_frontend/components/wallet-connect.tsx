"use client"

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useAuth } from '@/lib/auth-context'
import { getAvailableWallets, connectWallet, signData, type SupportedWallet } from '@/lib/cardano-wallet'
import { requestChallenge, verifySignature } from '@/lib/api'

interface WalletConnectProps {
  onSuccess?: () => void
  compact?: boolean // If true, renders without the card wrapper for use in dialogs
}

export function WalletConnect({ onSuccess }: WalletConnectProps) {
  const { login } = useAuth()
  const [availableWallets, setAvailableWallets] = useState<SupportedWallet[]>([])
  const [isConnecting, setIsConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedWallet, setSelectedWallet] = useState<string | null>(null)

  useEffect(() => {
    // Check for available wallets
    const wallets = getAvailableWallets()
    setAvailableWallets(wallets)

    // Auto-select if only one wallet available
    if (wallets.length === 1) {
      setSelectedWallet(wallets[0].id)
    }
  }, [])

  const handleConnect = async (walletId: string) => {
    setIsConnecting(true)
    setError(null)
    setSelectedWallet(walletId)

    try {
      // Step 1: Connect to wallet
      console.log('🔌 Connecting to wallet...', walletId)
      const { api, address, stakeAddress } = await connectWallet(walletId)
      console.log('✅ Wallet connected:', { address, stakeAddress })

      // Step 2: Request challenge from backend
      console.log('🔑 Requesting challenge from backend...')
      const { message, nonce } = await requestChallenge(address)
      console.log('✅ Challenge received:', { nonce, message: message.substring(0, 50) + '...' })

      // Step 3: Sign challenge with wallet
      console.log('✍️ Signing challenge with wallet...')
      const signature = await signData(api, address, message)
      console.log('✅ Signature created')

      // Step 4: Verify signature and get JWT
      console.log('🔐 Verifying signature with backend...')
      const { token } = await verifySignature(address, signature.signature, signature.key, stakeAddress)
      console.log('✅ Authentication successful!')

      // Step 5: Save authentication state
      const wallet = availableWallets.find(w => w.id === walletId)
      login(api, address, stakeAddress, token, wallet?.name || walletId)

      // Call success callback
      if (onSuccess) {
        onSuccess()
      }
    } catch (err) {
      console.error('❌ Authentication failed:', err)
      setError(err instanceof Error ? err.message : 'Failed to connect wallet')
    } finally {
      setIsConnecting(false)
    }
  }

  if (availableWallets.length === 0) {
    return (
      <Card className="p-8 text-center max-w-md mx-auto">
        <div className="text-4xl mb-4">💳</div>
        <h3 className="text-xl font-semibold mb-2">No Wallet Detected</h3>
        <p className="text-muted-foreground mb-6">
          Please install a Cardano wallet extension to continue
        </p>
        <div className="space-y-2 text-sm text-left">
          <p className="font-semibold">Supported wallets:</p>
          <ul className="list-disc list-inside space-y-1 text-muted-foreground">
            <li>Eternl (eternl.io)</li>
            <li>Lace (lace.io)</li>
            <li>Typhon (typhonwallet.io)</li>
            <li>Yoroi (yoroi-wallet.com)</li>
            <li>Nami (namiwallet.io)</li>
            <li>Flint (flint-wallet.com)</li>
          </ul>
        </div>
      </Card>
    )
  }

  return (
    <Card className="p-8 max-w-md mx-auto">
      <div className="text-center mb-6">
        <div className="text-4xl mb-4">🔗</div>
        <h2 className="text-2xl font-bold mb-2">Connect Your Wallet</h2>
        <p className="text-muted-foreground">
          Select a wallet to authenticate with the Cardano Blockchain Viewer
        </p>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
          <p className="text-sm text-destructive font-medium">❌ {error}</p>
        </div>
      )}

      <div className="space-y-3">
        {availableWallets.map((wallet) => (
          <Button
            key={wallet.id}
            onClick={() => handleConnect(wallet.id)}
            disabled={isConnecting}
            variant={selectedWallet === wallet.id ? "default" : "outline"}
            className="w-full justify-start gap-3 h-auto py-4"
          >
            <span className="text-2xl">{wallet.icon}</span>
            <div className="text-left flex-1">
              <div className="font-semibold">{wallet.name}</div>
              <div className="text-xs opacity-70">
                {isConnecting && selectedWallet === wallet.id ? 'Connecting...' : 'Click to connect'}
              </div>
            </div>
            {isConnecting && selectedWallet === wallet.id && (
              <div className="animate-spin">⏳</div>
            )}
          </Button>
        ))}
      </div>

      <div className="mt-6 text-xs text-center text-muted-foreground">
        <p>🔒 Your wallet signature proves ownership without exposing your private keys</p>
      </div>
    </Card>
  )
}
