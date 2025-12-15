"use client"

import type { StatsMessage } from "@/app/page"

interface ConnectionStatusProps {
  connected: boolean
  stats: StatsMessage["data"] | null
}

export function ConnectionStatus({ connected, stats }: ConnectionStatusProps) {
  return (
    <div className="border-b border-border bg-card backdrop-blur">
      <div className="container mx-auto px-4 py-4">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div className="flex items-center gap-3">
            <div
              className={`w-3 h-3 rounded-full transition-colors ${
                connected ? "bg-green-400 animate-pulse" : "bg-red-400"
              }`}
            />
            <span className="text-sm font-medium">
              {connected ? (
                <span className="text-green-400">✅ Connected to blockchain</span>
              ) : (
                <span className="text-red-400">❌ Disconnected - reconnecting...</span>
              )}
            </span>
          </div>

          {stats && (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <p className="text-muted-foreground text-xs mb-1">Events</p>
                <p className="font-semibold text-foreground">{stats.total_events}</p>
              </div>
              <div>
                <p className="text-muted-foreground text-xs mb-1">Blocks</p>
                <p className="font-semibold text-foreground">{stats.blocks_count}</p>
              </div>
              <div>
                <p className="text-muted-foreground text-xs mb-1">Transactions</p>
                <p className="font-semibold text-foreground">{stats.transactions_count}</p>
              </div>
              <div>
                <p className="text-muted-foreground text-xs mb-1">Epoch</p>
                <p className="font-semibold text-foreground">
                  {stats.last_slot ? Math.floor(stats.last_slot / 432000) : 0}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
