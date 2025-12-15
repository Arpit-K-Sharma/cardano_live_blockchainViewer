"use client"

import type { BlockEvent, TransactionEvent } from "@/app/page"
import { Card } from "@/components/ui/card"
import { CopyButton } from "@/components/copy-button"
import { formatDistanceToNow } from "date-fns"

interface DashboardProps {
  blocks: BlockEvent[]
  transactions: TransactionEvent[]
}

export function Dashboard({ blocks, transactions }: DashboardProps) {
  const formatTime = (timestamp: number) => {
    try {
      return formatDistanceToNow(new Date(timestamp * 1000), { addSuffix: true })
    } catch {
      return "just now"
    }
  }

  const formatADA = (lovelace: number) => {
    return (lovelace / 1_000_000).toFixed(2)
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Blocks */}
        <section>
          <h2 className="text-xl font-bold mb-4 text-foreground flex items-center gap-2">
            <span>📦</span> Recent Blocks (Top 10)
          </h2>
          <div className="space-y-3">
            {blocks.length === 0 ? (
              <Card className="p-4 text-center text-muted-foreground bg-card">Waiting for blocks...</Card>
            ) : (
              blocks.map((block, idx) => (
                <Card
                  key={`block-${block.number}-${idx}`}
                  className="p-4 hover:bg-card/80 transition-colors cursor-pointer border-border/50 hover:border-accent/50 bg-card"
                >
                  <div className="flex items-start justify-between mb-2 gap-3">
                    <div className="flex-1">
                      <p className="font-semibold text-foreground">Block #{block.number}</p>
                      <div className="mt-2 flex items-center gap-2">
                        <span className="text-xs text-muted-foreground">Hash:</span>
                        <CopyButton text={block.hash} displayText={block.hash.slice(0, 16) + "..."} />
                      </div>
                    </div>
                    <a
                      href={`https://preprod.cardanoscan.io/block/${block.number}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-accent hover:text-accent/80 text-xs font-medium whitespace-nowrap hover:underline"
                    >
                      CardanoScan →
                    </a>
                  </div>
                  <div className="grid grid-cols-3 gap-2 text-xs">
                    <div>
                      <p className="text-muted-foreground">TXs</p>
                      <p className="font-semibold text-foreground">{block.tx_count}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Slot</p>
                      <p className="font-semibold text-foreground">{block.slot}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Time</p>
                      <p className="font-semibold text-foreground text-xs">{formatTime(block.timestamp)}</p>
                    </div>
                  </div>
                </Card>
              ))
            )}
          </div>
        </section>

        {/* Recent Transactions */}
        <section>
          <h2 className="text-xl font-bold mb-4 text-foreground flex items-center gap-2">
            <span>💳</span> Recent Transactions (Top 10)
          </h2>
          <div className="space-y-3">
            {transactions.length === 0 ? (
              <Card className="p-4 text-center text-muted-foreground bg-card">Waiting for transactions...</Card>
            ) : (
              transactions.map((tx, idx) => (
                <Card
                  key={`tx-${tx.hash}-${idx}`}
                  className="p-4 hover:bg-card/80 transition-colors cursor-pointer border-border/50 hover:border-accent/50 bg-card"
                >
                  <div className="flex items-start justify-between mb-2 gap-3">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-xs text-muted-foreground">TX Hash:</span>
                        <CopyButton text={tx.hash} displayText={tx.hash.slice(0, 12) + "..."} />
                      </div>
                    </div>
                    <a
                      href={`https://preprod.cardanoscan.io/transaction/${tx.hash}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-accent hover:text-accent/80 text-xs font-medium whitespace-nowrap hover:underline"
                    >
                      CardanoScan →
                    </a>
                  </div>
                  <div className="grid grid-cols-3 gap-2 text-xs mb-2">
                    <div>
                      <p className="text-muted-foreground">Fee</p>
                      <p className="font-semibold text-foreground">₳{formatADA(tx.fee)}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Total</p>
                      <p className="font-semibold text-foreground">₳{formatADA(tx.total_output)}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">I/O</p>
                      <p className="font-semibold text-foreground">
                        {tx.inputs}→{tx.outputs}
                      </p>
                    </div>
                  </div>
                  <p className="text-xs text-muted-foreground">{formatTime(tx.timestamp)}</p>
                </Card>
              ))
            )}
          </div>
        </section>
      </div>
    </div>
  )
}
