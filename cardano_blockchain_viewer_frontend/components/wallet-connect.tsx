"use client"

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useAuth } from '@/lib/auth-context'
import { getAvailableWallets, connectWallet, signData, SUPPORTED_WALLETS, type SupportedWallet } from '@/lib/cardano-wallet'
import { requestChallenge, verifySignature } from '@/lib/api'

interface WalletConnectProps {
  onSuccess?: () => void
  compact?: boolean // If true, renders without the card wrapper for use in dialogs
}

export function WalletConnect({ onSuccess, compact = false }: WalletConnectProps) {
  const { login } = useAuth()
  const [installedWallets, setInstalledWallets] = useState<SupportedWallet[]>([])
  const [isConnecting, setIsConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedWallet, setSelectedWallet] = useState<string | null>(null)

  useEffect(() => {
    const available = getAvailableWallets()
    setInstalledWallets(available)

    // Auto-select first installed wallet if any
    if (available.length === 1) {
      setSelectedWallet(available[0].id)
    }
  }, [])

  const handleConnect = async (walletId: string) => {
    setIsConnecting(true)
    setError(null)
    setSelectedWallet(walletId)

    try {
      // Step 1: Connect to wallet
      const { api, address, stakeAddress } = await connectWallet(walletId)

      // Identify address format for CIP-30 signData and backend usage
      const isBech32 = address.startsWith('addr')
      const looksHex = /^[0-9a-fA-F]+$/.test(address)
      
      // Step 2: Request challenge from backend (send address as provided by the wallet)
      const { message } = await requestChallenge(address)

      // Step 3: Sign challenge with wallet (CIP-30)
      const signature = await signData(api, address, message)

      // Step 4: Verify signature and get JWT
      const { token } = await verifySignature(address, signature.signature, signature.key, stakeAddress)

      // Step 5: Save authentication state (show wallet name after login)
      const wallet = SUPPORTED_WALLETS.find(w => w.id === walletId)
      login(api, address, stakeAddress, token, wallet?.name || walletId)

      if (onSuccess) onSuccess()
    } catch (err) {
      console.error('❌ Authentication failed:', err)
      setError(err instanceof Error ? err.message : 'Failed to connect wallet')
    } finally {
      setIsConnecting(false)
    }
  }

  const content = (
    <div className="flex flex-col gap-6">
      <div className="text-center">
        <div className="text-4xl mb-2">🔗</div>
        <h2 className="text-2xl font-bold mb-1">Connect Your Wallet</h2>
        <p className="text-muted-foreground">Select a CIP-30 compatible wallet</p>
      </div>

      {error && (
        <div className="p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
          <p className="text-sm text-destructive font-medium">❌ {error}</p>
        </div>
      )}

      <div className="space-y-3">
        {SUPPORTED_WALLETS.map((wallet) => {
          const installed = installedWallets.some(w => w.id === wallet.id)
          const disabled = !installed || isConnecting
          const isSelected = selectedWallet === wallet.id

          return (
            <Button
              key={wallet.id}
              onClick={() => installed && handleConnect(wallet.id)}
              disabled={disabled}
              variant={isSelected ? 'default' : 'outline'}
              className="w-full justify-start gap-3 h-auto py-4"
            >
              <span className="text-2xl">{wallet.icon}</span>
              <div className="text-left flex-1">
                <div className="font-semibold flex items-center gap-2">
                  <span>{wallet.name}</span>
                  {installed ? (
                    <span className="text-xs px-2 py-0.5 rounded bg-green-500/10 text-green-600 border border-green-500/20">Installed</span>
                  ) : (
                    <span className="text-xs px-2 py-0.5 rounded bg-muted text-muted-foreground">Not installed</span>
                  )}
                </div>
                <div className="text-xs opacity-70">
                  {isConnecting && isSelected
                    ? 'Connecting...'
                    : (installed ? 'Click to connect' : 'Install the extension to use this wallet')}
                </div>
              </div>
              {isConnecting && isSelected && (
                <div className="animate-spin">⏳</div>
              )}
            </Button>
          )
        })}
      </div>

      <div className="text-xs text-center text-muted-foreground">
        <p>🔒 Your wallet signature proves ownership without exposing your private keys</p>
      </div>
    </div>
  )

  if (compact) {
    return content
  }

  return (
    <Card className="p-8 max-w-md mx-auto">
      {content}
    </Card>
  )
}
