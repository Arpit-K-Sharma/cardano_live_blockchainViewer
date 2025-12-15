"use client"

import type { BlockGroup } from "@/app/page"
import { Card } from "@/components/ui/card"
import { CopyButton } from "@/components/copy-button"
import { useState } from "react"
import { formatDistanceToNow } from "date-fns"

interface BlockExplorerProps {
  blockGroups: Array<BlockGroup & { timestamp: number }>
}

export function BlockExplorer({ blockGroups }: BlockExplorerProps) {
  const [expandedBlocks, setExpandedBlocks] = useState<Set<number>>(new Set())
  const [expandedTxs, setExpandedTxs] = useState<Set<string>>(new Set())
  const [page, setPage] = useState(1)

  const toggleBlockExpanded = (timestamp: number) => {
    const newSet = new Set(expandedBlocks)
    if (newSet.has(timestamp)) {
      newSet.delete(timestamp)
    } else {
      newSet.add(timestamp)
    }
    setExpandedBlocks(newSet)
  }

  const toggleTxExpanded = (txHash: string) => {
    const newSet = new Set(expandedTxs)
    if (newSet.has(txHash)) {
      newSet.delete(txHash)
    } else {
      newSet.add(txHash)
    }
    setExpandedTxs(newSet)
  }

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

  const ITEMS_PER_PAGE = 10
  const startIdx = (page - 1) * ITEMS_PER_PAGE
  const endIdx = startIdx + ITEMS_PER_PAGE
  const paginatedGroups = blockGroups.slice(startIdx, endIdx)
  const totalPages = Math.ceil(blockGroups.length / ITEMS_PER_PAGE)

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="space-y-6">
        {blockGroups.length === 0 ? (
          <Card className="p-8 text-center text-muted-foreground">Waiting for blocks...</Card>
        ) : (
          <>
            {paginatedGroups.map((group) => (
              <Card key={`block-${group.timestamp}`} className="p-6 border-border/50 bg-card">
                {group.block ? (
                  <div className="space-y-4">
                    {/* Block Header - Clickable area to expand */}
                    <div
                      onClick={() => toggleBlockExpanded(group.timestamp)}
                      className="cursor-pointer hover:opacity-80 transition-opacity"
                    >
                      <div className="flex items-start justify-between mb-3 gap-3">
                        <div className="flex-1">
                          <h3 className="text-lg font-bold text-foreground flex items-center gap-2">
                            <span>📦</span> Block #{group.block.number}
                          </h3>
                          <div className="mt-3 flex flex-col gap-2">
                            <div className="flex items-center gap-2">
                              <span className="text-xs text-muted-foreground">Hash:</span>
                              <CopyButton text={group.block.hash} displayText={group.block.hash.slice(0, 16) + "..."} />
                            </div>
                          </div>
                        </div>
                        <a
                          href={`https://preprod.cardanoscan.io/block/${group.block.number}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-accent hover:text-accent/80 text-sm font-medium whitespace-nowrap hover:underline"
                          onClick={(e) => e.stopPropagation()}
                        >
                          CardanoScan →
                        </a>
                      </div>

                      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
                        <div className="bg-card/50 p-3 rounded border border-border/30">
                          <p className="text-muted-foreground text-xs mb-1">Slot</p>
                          <p className="font-semibold text-foreground">{group.block.slot}</p>
                        </div>
                        <div className="bg-card/50 p-3 rounded border border-border/30">
                          <p className="text-muted-foreground text-xs mb-1">Epoch</p>
                          <p className="font-semibold text-foreground">{group.block.epoch}</p>
                        </div>
                        <div className="bg-card/50 p-3 rounded border border-border/30">
                          <p className="text-muted-foreground text-xs mb-1">Transactions</p>
                          <p className="font-semibold text-foreground">{group.block.tx_count}</p>
                        </div>
                        <div className="bg-card/50 p-3 rounded border border-border/30">
                          <p className="text-muted-foreground text-xs mb-1">Time</p>
                          <p className="font-semibold text-foreground text-xs">{formatTime(group.block.timestamp)}</p>
                        </div>
                      </div>

                      <div className="text-xs text-muted-foreground mt-3">
                        {expandedBlocks.has(group.timestamp) ? "▼" : "▶"} Click to expand transactions
                      </div>
                    </div>

                    {/* Expanded Transactions Section */}
                    {expandedBlocks.has(group.timestamp) && (
                      <div className="mt-4 pt-4 border-t border-border/30 space-y-3">
                        <h4 className="font-semibold text-foreground text-sm">
                          Transactions in Block #{group.block.number} ({group.transactions.length})
                        </h4>
                        {group.transactions.length === 0 ? (
                          <p className="text-xs text-muted-foreground">No transactions in this block</p>
                        ) : (
                          <div className="space-y-3">
                            {group.transactions.map((tx, txIdx) => (
                              <div
                                key={`${group.block!.number}-tx-${txIdx}`}
                                className="bg-card/30 rounded p-4 border border-border/40"
                              >
                                {/* Transaction Header - Clickable to expand */}
                                <div
                                  onClick={() => toggleTxExpanded(tx.hash)}
                                  className="cursor-pointer hover:opacity-80 transition-opacity"
                                >
                                  <div className="flex items-start justify-between gap-3 mb-3">
                                    <div className="flex-1 min-w-0">
                                      <div className="flex items-center gap-2 mb-2 flex-wrap">
                                        <span className="text-xs text-muted-foreground">TX:</span>
                                        <CopyButton text={tx.hash} displayText={tx.hash.slice(0, 12) + "..."} />
                                      </div>
                                      <div className="grid grid-cols-3 gap-2 text-xs">
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
                                    </div>
                                    <div className="text-right flex flex-col items-end gap-2 whitespace-nowrap">
                                      <a
                                        href={`https://preprod.cardanoscan.io/transaction/${tx.hash}`}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-accent hover:text-accent/80 text-xs font-medium hover:underline"
                                        onClick={(e) => e.stopPropagation()}
                                      >
                                        CardanoScan →
                                      </a>
                                      <span className="text-xs text-muted-foreground">
                                        {expandedTxs.has(tx.hash) ? "▼" : "▶"}
                                      </span>
                                    </div>
                                  </div>
                                </div>

                                {/* Expanded Transaction Details */}
                                {expandedTxs.has(tx.hash) && (
                                  <div className="mt-3 pt-3 border-t border-border/30 space-y-3">
                                    <div>
                                      <p className="font-semibold text-foreground text-xs mb-2">
                                        Inputs ({group.inputs.filter((i) => i.tx_hash === tx.hash).length})
                                      </p>
                                      {group.inputs.filter((i) => i.tx_hash === tx.hash).length === 0 ? (
                                        <p className="text-xs text-muted-foreground">No inputs</p>
                                      ) : (
                                        <div className="space-y-2">
                                          {group.inputs
                                            .filter((i) => i.tx_hash === tx.hash)
                                            .map((input, idx) => (
                                              <div
                                                key={`${tx.hash}-input-${idx}`}
                                                className="text-muted-foreground font-mono text-xs bg-card/20 p-2 rounded border border-border/20 flex items-center justify-between gap-2"
                                              >
                                                <span className="truncate">{"{..."}</span>
                                                <CopyButton
                                                  text={input.input_tx_id}
                                                  displayText={input.input_tx_id.slice(0, 8) + "..."}
                                                />
                                                <span>#{input.input_index}</span>
                                              </div>
                                            ))}
                                        </div>
                                      )}
                                    </div>

                                    <div>
                                      <p className="font-semibold text-foreground text-xs mb-2">
                                        Outputs ({group.outputs.filter((o) => o.tx_hash === tx.hash).length})
                                      </p>
                                      {group.outputs.filter((o) => o.tx_hash === tx.hash).length === 0 ? (
                                        <p className="text-xs text-muted-foreground">No outputs</p>
                                      ) : (
                                        <div className="space-y-2">
                                          {group.outputs
                                            .filter((o) => o.tx_hash === tx.hash)
                                            .map((output, idx) => (
                                              <div
                                                key={`${tx.hash}-output-${idx}`}
                                                className="text-muted-foreground font-mono text-xs bg-card/20 p-3 rounded border border-border/20"
                                              >
                                                <p className="mb-2 font-semibold text-foreground">
                                                  ₳{formatADA(output.amount)}
                                                </p>
                                                <div className="flex items-center gap-2">
                                                  <span className="text-muted-foreground text-xs whitespace-nowrap">
                                                    Address:
                                                  </span>
                                                  <CopyButton
                                                    text={output.address}
                                                    displayText={output.address.slice(0, 16) + "..."}
                                                  />
                                                </div>
                                              </div>
                                            ))}
                                        </div>
                                      )}
                                    </div>
                                  </div>
                                )}
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="text-muted-foreground text-center py-4">
                    {group.transactions.length} transactions (waiting for block header...)
                  </div>
                )}
              </Card>
            ))}

            {/* Pagination Controls */}
            {totalPages > 1 && (
              <div className="flex items-center justify-center gap-4 mt-8 pb-4">
                <button
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                  disabled={page === 1}
                  className="px-4 py-2 rounded text-sm bg-accent/10 text-accent hover:bg-accent/20 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
                >
                  ← Previous
                </button>
                <span className="text-sm text-foreground font-medium">
                  Page {page} of {totalPages}
                </span>
                <button
                  onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                  disabled={page === totalPages}
                  className="px-4 py-2 rounded text-sm bg-accent/10 text-accent hover:bg-accent/20 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
                >
                  Next →
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}
