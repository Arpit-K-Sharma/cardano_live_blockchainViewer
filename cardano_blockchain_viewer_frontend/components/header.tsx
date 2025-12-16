"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { useAuth } from "@/lib/auth-context"
import { WalletConnect } from "@/components/wallet-connect"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"

interface HeaderProps {
  view: "dashboard" | "explorer" | "history"
  setView: (view: "dashboard" | "explorer" | "history") => void
}

export function Header({ view, setView }: HeaderProps) {
  const { isAuthenticated, address, walletName, logout } = useAuth()
  const [showWalletDialog, setShowWalletDialog] = useState(false)

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 8)}...${addr.slice(-8)}`
  }

  return (
    <>
      <header className="border-b border-border bg-card/50 backdrop-blur">
        <div className="container mx-auto px-4 py-6">
          <div className="flex items-center justify-between gap-4 flex-wrap">
            <div>
              <h1 className="text-3xl font-bold text-foreground flex items-center gap-2">
                <span className="text-2xl">⛓️</span> Cardano Live Viewer
              </h1>
              <p className="text-sm text-muted-foreground mt-1">Real-time Cardano PreProd testnet explorer</p>
            </div>
            <div className="flex gap-2 whitespace-nowrap items-center flex-wrap">
              {/* Navigation - Always visible */}
              <Button
                variant={view === "dashboard" ? "default" : "outline"}
                onClick={() => setView("dashboard")}
                className="gap-2"
              >
                <span>📊</span> Dashboard
              </Button>
              <Button
                variant={view === "explorer" ? "default" : "outline"}
                onClick={() => setView("explorer")}
                className="gap-2"
              >
                <span>🔍</span> Explorer
              </Button>
              {isAuthenticated && (
                <Button
                  variant={view === "history" ? "default" : "outline"}
                  onClick={() => setView("history")}
                  className="gap-2"
                >
                  <span>📜</span> My Transactions
                </Button>
              )}

              {/* Wallet section */}
              {isAuthenticated ? (
                <>
                  <div className="flex items-center gap-2 px-3 py-2 bg-card rounded-lg border border-border">
                    <span className="text-xs text-muted-foreground">{walletName}</span>
                    <span className="text-xs font-mono">{address ? formatAddress(address) : ''}</span>
                  </div>
                  <Button variant="outline" onClick={logout} className="gap-2">
                    <span>🚪</span> Disconnect
                  </Button>
                </>
              ) : (
                <Button
                  variant="default"
                  onClick={() => setShowWalletDialog(true)}
                  className="gap-2"
                >
                  <span>👛</span> Connect Wallet
                </Button>
              )}
            </div>
          </div>
        </div>
      </header>

      {/* Wallet Connection Dialog */}
      <Dialog open={showWalletDialog} onOpenChange={setShowWalletDialog}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Connect Your Wallet</DialogTitle>
            <DialogDescription>
              Connect your Cardano wallet to access personalized features like transaction history and portfolio summary.
            </DialogDescription>
          </DialogHeader>
          <WalletConnect onSuccess={() => setShowWalletDialog(false)} compact />
        </DialogContent>
      </Dialog>
    </>
  )
}
