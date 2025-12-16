"use client"

import { useState, useEffect } from 'react'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { CopyButton } from '@/components/copy-button'
import { useAuth } from '@/lib/auth-context'
import { getUserTransactions, getUserSummary, type Transaction, type WalletSummary } from '@/lib/api'
import { formatDistanceToNow } from 'date-fns'

export function UserTransactionHistory() {
  const { isAuthenticated, token, walletApi } = useAuth()
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [summary, setSummary] = useState<WalletSummary | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [page, setPage] = useState(1)
  const [hasMore, setHasMore] = useState(true)

  // Get wallet address directly from wallet API (raw, hex or bech32)
  const getWalletAddress = async (): Promise<string | null> => {
    if (!walletApi) {
      setError('Wallet not connected. Please reconnect your wallet.')
      return null
    }

    try {
      // Try getChangeAddress first (primary payment address)
      try {
        const changeAddress = await walletApi.getChangeAddress()
        if (changeAddress) {
          return changeAddress
        }
      } catch (err) {
        console.warn('getChangeAddress() failed:', err)
      }

      // Fallback to getUsedAddresses
      try {
        const usedAddresses = await walletApi.getUsedAddresses()
        if (usedAddresses && usedAddresses.length > 0) {
          return usedAddresses[0]
        }
      } catch (err) {
        console.warn('getUsedAddresses() failed:', err)
      }

      // Fallback to getUnusedAddresses
      try {
        const unusedAddresses = await walletApi.getUnusedAddresses()
        if (unusedAddresses && unusedAddresses.length > 0) {
          return unusedAddresses[0]
        }
      } catch (err) {
        console.warn('getUnusedAddresses() failed:', err)
      }

      setError('No addresses found in wallet')
      return null
    } catch (err) {
      console.error('Failed to get wallet address:', err)
      setError(err instanceof Error ? err.message : 'Failed to get wallet address')
      return null
    }
  }

  const loadTransactions = async (pageNum: number = 1) => {
    if (!token) return

    try {
      setLoading(true)
      setError(null)

      // Get wallet address directly from wallet (raw)
      const rawWalletAddress = await getWalletAddress()
      if (!rawWalletAddress) {
        setLoading(false)
        return
      }

      // Load summary and transactions in parallel
      const [summaryData, txData] = await Promise.all([
        getUserSummary(token, rawWalletAddress),
        getUserTransactions(token, rawWalletAddress, pageNum, 20),
      ])

      setSummary(summaryData)
      
      if (pageNum === 1) {
        setTransactions(txData.transactions)
      } else {
        setTransactions((prev) => [...prev, ...txData.transactions])
      }

      setHasMore(txData.transactions.length === 20)
    } catch (err) {
      console.error('Failed to load transactions:', err)
      setError(err instanceof Error ? err.message : 'Failed to load transaction history')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (isAuthenticated && token && walletApi) {
      loadTransactions(1)
    } else {
      setTransactions([])
      setSummary(null)
      setLoading(false)
    }
  }, [isAuthenticated, token, walletApi])

  const formatTime = (timestamp: number) => {
    try {
      return formatDistanceToNow(new Date(timestamp * 1000), { addSuffix: true })
    } catch {
      return 'unknown'
    }
  }

  const formatADA = (lovelace: string) => {
    const amount = BigInt(lovelace)
    return (Number(amount) / 1_000_000).toFixed(2)
  }

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 8)}...${addr.slice(-8)}`
  }

  if (!isAuthenticated) {
    return null
  }

  return (
    <div className="container mx-auto px-4 py-8">
      {/* Wallet Summary Section */}
      {summary && (
        <Card className="p-6 mb-8 bg-gradient-to-r from-card to-card/80 border-border">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-2xl font-bold text-foreground flex items-center gap-2">
              <span>💼</span> Your Wallet
            </h2>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <p className="text-sm text-muted-foreground mb-1">Address</p>
              <div className="flex items-center gap-2">
                <p className="font-mono text-sm font-semibold text-foreground">
                  {formatAddress(summary.address)}
                </p>
                <CopyButton text={summary.address} displayText="📋" />
              </div>
            </div>
            <div>
              <p className="text-sm text-muted-foreground mb-1">Balance</p>
              <p className="text-2xl font-bold text-foreground">
                ₳{formatADA(summary.balance)}
              </p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground mb-1">Total Transactions</p>
              <p className="text-2xl font-bold text-foreground">
                {summary.transaction_count.toLocaleString()}
              </p>
            </div>
          </div>
          {summary.stake_address && (
            <div className="mt-4 pt-4 border-t border-border">
              <p className="text-sm text-muted-foreground mb-1">Stake Address</p>
              <div className="flex items-center gap-2">
                <p className="font-mono text-sm text-foreground">
                  {formatAddress(summary.stake_address)}
                </p>
                <CopyButton text={summary.stake_address} displayText="📋" />
              </div>
            </div>
          )}
        </Card>
      )}

      {/* Transaction History Section */}
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-foreground flex items-center gap-2">
            <span>📜</span> Transaction History
          </h2>
          {transactions.length > 0 && (
            <p className="text-sm text-muted-foreground">
              Showing {transactions.length} transaction{transactions.length !== 1 ? 's' : ''}
            </p>
          )}
        </div>

        {loading && transactions.length === 0 ? (
          <Card className="p-8 text-center">
            <div className="animate-spin text-4xl mb-4">⏳</div>
            <p className="text-muted-foreground">Loading your transactions...</p>
          </Card>
        ) : error ? (
          <Card className="p-6 border-destructive/50 bg-destructive/10">
            <p className="text-destructive font-medium">❌ {error}</p>
            <Button
              variant="outline"
              onClick={() => loadTransactions(1)}
              className="mt-4"
            >
              Retry
            </Button>
          </Card>
        ) : transactions.length === 0 ? (
          <Card className="p-8 text-center">
            <div className="text-4xl mb-4">📭</div>
            <p className="text-muted-foreground">No transactions found</p>
            <p className="text-sm text-muted-foreground mt-2">
              Your transaction history will appear here once you make transactions.
            </p>
          </Card>
        ) : (
          <>
            <div className="space-y-3">
              {transactions.map((tx, idx) => (
                <Card
                  key={`${tx.tx_hash}-${idx}`}
                  className="p-5 hover:bg-card/80 transition-colors border-border/50 hover:border-accent/50 bg-card"
                >
                  <div className="flex items-start justify-between mb-3 gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-xs text-muted-foreground">Transaction Hash:</span>
                        <CopyButton
                          text={tx.tx_hash}
                          displayText={`${tx.tx_hash.slice(0, 12)}...${tx.tx_hash.slice(-8)}`}
                        />
                      </div>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <span>Block: #{tx.block_height}</span>
                        <span>•</span>
                        <span>Slot: {tx.slot.toLocaleString()}</span>
                        <span>•</span>
                        <span>Index: {tx.index}</span>
                      </div>
                    </div>
                    <a
                      href={`https://preprod.cardanoscan.io/transaction/${tx.tx_hash}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-accent hover:text-accent/80 text-xs font-medium whitespace-nowrap hover:underline flex items-center gap-1"
                    >
                      View <span>→</span>
                    </a>
                  </div>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-4 pt-3 border-t border-border/50">
                    <div>
                      <p className="text-xs text-muted-foreground mb-1">Fee</p>
                      <p className="font-semibold text-foreground">₳{formatADA(tx.fees)}</p>
                    </div>
                    <div>
                      <p className="text-xs text-muted-foreground mb-1">Block Hash</p>
                      <p className="font-mono text-xs text-foreground truncate">
                        {tx.block.slice(0, 12)}...
                      </p>
                    </div>
                    <div>
                      <p className="text-xs text-muted-foreground mb-1">Time</p>
                      <p className="text-xs font-semibold text-foreground">
                        {formatTime(tx.block_time)}
                      </p>
                    </div>
                  </div>
                </Card>
              ))}
            </div>

            {/* Load More Button */}
            {hasMore && (
              <div className="mt-6 text-center">
                <Button
                  variant="outline"
                  onClick={() => {
                    const nextPage = page + 1
                    setPage(nextPage)
                    loadTransactions(nextPage)
                  }}
                  disabled={loading}
                  className="gap-2"
                >
                  {loading ? (
                    <>
                      <span className="animate-spin">⏳</span> Loading...
                    </>
                  ) : (
                    <>
                      <span>⬇️</span> Load More Transactions
                    </>
                  )}
                </Button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}

