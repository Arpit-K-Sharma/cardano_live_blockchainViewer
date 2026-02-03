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

// Wallet installation URLs
const WALLET_INSTALL_URLS: Record<string, string> = {
  eternl: 'https://chromewebstore.google.com/detail/eternl/kmhcihpebfmpgmihbkipmjlmmioameka',
  lace: 'https://chromewebstore.google.com/detail/lace/gafhhkghbfjjkeiendhlofajokpaflmk',
  typhon: 'https://chromewebstore.google.com/detail/typhon-wallet/kfdniefadaanbjodldohaedphafoffoh',
  yoroi: 'https://chromewebstore.google.com/detail/yoroi/ffnbelfdoeiohenkjibnmadjiehjhajb',
  nami: 'https://chromewebstore.google.com/detail/nami/lpfcbjknijpeeillifnkikgncikgfhdo',
  flint: 'https://chromewebstore.google.com/detail/flint-wallet/hnhobjmcibchnmglfbldbfabcgaknlkj',
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
      // Signature is in COSE-Sign1 format
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

  const handleInstall = (walletId: string) => {
    const url = WALLET_INSTALL_URLS[walletId]
    if (url) {
      window.open(url, '_blank', 'noopener,noreferrer')
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

      {/* Scrollable wallet list with max height */}
      <div className="space-y-3 max-h-[400px] overflow-y-auto pr-2 -mr-2">
        {SUPPORTED_WALLETS.map((wallet) => {
          const installed = installedWallets.some(w => w.id === wallet.id)
          const isSelected = selectedWallet === wallet.id
          const isProcessing = isConnecting && isSelected

          return (
            <div
              key={wallet.id}
              className="border rounded-lg p-4 hover:border-primary/50 transition-colors"
            >
              <div className="flex items-start gap-3">
                <span className="text-3xl">{wallet.icon}</span>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-semibold">{wallet.name}</span>
                    {installed ? (
                      <span className="text-xs px-2 py-0.5 rounded bg-green-500/10 text-green-600 border border-green-500/20">
                        ✓ Installed
                      </span>
                    ) : (
                      <span className="text-xs px-2 py-0.5 rounded bg-orange-500/10 text-orange-600 border border-orange-500/20">
                        Not Installed
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-muted-foreground mb-3">
                    {installed 
                      ? 'Ready to connect to your wallet'
                      : 'Install the browser extension to use this wallet'}
                  </div>
                  
                  {installed ? (
                    <Button
                      onClick={() => handleConnect(wallet.id)}
                      disabled={isConnecting}
                      variant={isSelected ? 'default' : 'outline'}
                      className="w-full"
                      size="sm"
                    >
                      {isProcessing ? (
                        <>
                          <span className="animate-spin mr-2">⏳</span>
                          Connecting...
                        </>
                      ) : (
                        'Connect Wallet'
                      )}
                    </Button>
                  ) : (
                    <Button
                      onClick={() => handleInstall(wallet.id)}
                      variant="secondary"
                      className="w-full"
                      size="sm"
                    >
                      <span className="mr-2">📥</span>
                      Install {wallet.name}
                    </Button>
                  )}
                </div>
              </div>
            </div>
          )
        })}
      </div>

      <div className="text-xs text-center text-muted-foreground pt-2 border-t">
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