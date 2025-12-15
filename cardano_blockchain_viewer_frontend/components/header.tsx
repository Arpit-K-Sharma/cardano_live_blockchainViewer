"use client"

import { Button } from "@/components/ui/button"

interface HeaderProps {
  view: "dashboard" | "explorer"
  setView: (view: "dashboard" | "explorer") => void
}

export function Header({ view, setView }: HeaderProps) {
  return (
    <header className="border-b border-border bg-card/50 backdrop-blur">
      <div className="container mx-auto px-4 py-6">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold text-foreground flex items-center gap-2">
              <span className="text-2xl">⛓️</span> Cardano Live Viewer
            </h1>
            <p className="text-sm text-muted-foreground mt-1">Real-time Cardano PreProd testnet explorer</p>
          </div>
          <div className="flex gap-2 whitespace-nowrap">
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
          </div>
        </div>
      </div>
    </header>
  )
}
