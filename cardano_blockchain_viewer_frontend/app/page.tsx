"use client"

import { useEffect, useState, useRef } from "react"
import { BlockExplorer } from "@/components/block-explorer"
import { Dashboard } from "@/components/dashboard"
import { Header } from "@/components/header"
import { ConnectionStatus } from "@/components/connection-status"
import { UserTransactionHistory } from "@/components/user-transaction-history"
import { useAuth } from "@/lib/auth-context"

export interface BlockEvent {
  type: "Block"
  slot: number
  hash: string
  number: number
  epoch: number
  tx_count: number
  timestamp: number
  details?: Record<string, unknown>
}

export interface TransactionEvent {
  type: "Transaction"
  hash: string
  fee: number
  inputs: number
  outputs: number
  total_output: number
  timestamp: number
  details?: Record<string, unknown>
}

export interface TxInputEvent {
  type: "TxInput"
  tx_hash: string
  input_tx_id: string
  input_index: number
  timestamp: number
}

export interface TxOutputEvent {
  type: "TxOutput"
  tx_hash: string
  address: string
  amount: number
  timestamp: number
}

export interface RollBackEvent {
  type: "RollBack"
  block_hash: string
  block_slot: number
  timestamp: number
}

export interface StatsMessage {
  type: "stats"
  data: {
    total_events: number
    blocks_count: number
    transactions_count: number
    inputs_count: number
    outputs_count: number
    buffer_size: number
    last_block_number: number
    last_slot: number
  }
}

export interface BlockGroup {
  block: BlockEvent | null
  transactions: TransactionEvent[]
  inputs: TxInputEvent[]
  outputs: TxOutputEvent[]
}

export interface AppState {
  connected: boolean
  stats: StatsMessage["data"] | null
  blockGroups: Record<number, BlockGroup>
  recentBlocks: BlockEvent[]
  recentTransactions: TransactionEvent[]
}

type MessageEvent = BlockEvent | TransactionEvent | TxInputEvent | TxOutputEvent | RollBackEvent | StatsMessage

export default function Home() {
  const { isAuthenticated } = useAuth()
  const [state, setState] = useState<AppState>({
    connected: false,
    stats: null,
    blockGroups: {},
    recentBlocks: [],
    recentTransactions: [],
  })
  const [view, setView] = useState<"dashboard" | "explorer" | "history">("dashboard")
  const wsRef = useRef<WebSocket | null>(null)

  useEffect(() => {
    // Connect WebSocket regardless of authentication status
    // Authentication is only needed for personalized features
    const connectWebSocket = () => {
      try {
        const ws = new WebSocket("ws://localhost:8080/ws")

        ws.onopen = () => {
          console.log("✅ Connected to Cardano blockchain!")
          setState((prev) => ({ ...prev, connected: true }))
        }

        ws.onmessage = (event) => {
          try {
            const data: MessageEvent = JSON.parse(event.data)
            handleMessage(data)
          } catch (error) {
            console.error("Error parsing message:", error)
          }
        }

        ws.onerror = (error) => {
          console.error("❌ WebSocket error:", error)
          setState((prev) => ({ ...prev, connected: false }))
        }

        ws.onclose = () => {
          console.log("⚠️ Disconnected. Reconnecting in 3 seconds...")
          setState((prev) => ({ ...prev, connected: false }))
          setTimeout(connectWebSocket, 3000)
        }

        wsRef.current = ws
      } catch (error) {
        console.error("Failed to connect:", error)
        setTimeout(connectWebSocket, 3000)
      }
    }

    connectWebSocket()

    return () => {
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, []) // Removed isAuthenticated dependency - always connect

  const handleMessage = (data: MessageEvent) => {
    console.log("[v0] Received message:", data.type, data)

    if (data.type === "stats") {
      console.log("[v0] Stats updated:", data.data)
      setState((prev) => ({
        ...prev,
        stats: data.data,
      }))
      return
    }

    const timestamp = data.timestamp

    setState((prev) => {
      const newState = { ...prev }

      // Ensure blockGroup exists
      if (!newState.blockGroups[timestamp]) {
        newState.blockGroups[timestamp] = {
          block: null,
          transactions: [],
          inputs: [],
          outputs: [],
        }
      }

      switch (data.type) {
        case "Block":
          newState.blockGroups[timestamp].block = data
          newState.recentBlocks = [data, ...newState.recentBlocks].slice(0, 10)
          break

        case "Transaction":
          newState.blockGroups[timestamp].transactions.push(data)
          newState.recentTransactions = [data, ...newState.recentTransactions].slice(0, 10)
          break

        case "TxInput":
          newState.blockGroups[timestamp].inputs.push(data)
          break

        case "TxOutput":
          newState.blockGroups[timestamp].outputs.push(data)
          break



        case "RollBack":
          // Handle rollback by removing events after the rollback slot
          Object.keys(newState.blockGroups).forEach((ts) => {
            const group = newState.blockGroups[Number.parseInt(ts)]
            if (group && group.block && group.block.slot > data.block_slot) {
              delete newState.blockGroups[Number.parseInt(ts)]
            }
          })
          break
      }

      return newState
    })
  }

  const blockGroups = Object.entries(state.blockGroups)
    .sort(([a], [b]) => Number.parseInt(b) - Number.parseInt(a))
    .map(([timestamp, group]) => ({
      timestamp: Number.parseInt(timestamp),
      ...group,
    }))

  // Show blockchain viewer to everyone (authentication not required)
  return (
    <main className="min-h-screen bg-background">
      <Header view={view} setView={setView} />
      <ConnectionStatus connected={state.connected} stats={state.stats} />

      {view === "dashboard" ? (
        <Dashboard blocks={state.recentBlocks} transactions={state.recentTransactions} />
      ) : view === "explorer" ? (
        <BlockExplorer blockGroups={blockGroups} />
      ) : (
        <UserTransactionHistory />
      )}
    </main>
  )
}
