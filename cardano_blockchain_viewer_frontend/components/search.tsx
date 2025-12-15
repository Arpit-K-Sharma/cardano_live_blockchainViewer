"use client"

import { type FormEvent, useState } from "react"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"

interface SearchProps {
  onSearchBlock: (blockNumber: number) => void
  onSearchTx: (txHash: string) => void
}

export function Search({ onSearchBlock, onSearchTx }: SearchProps) {
  const [query, setQuery] = useState("")

  const handleSearch = (e: FormEvent) => {
    e.preventDefault()

    if (!query.trim()) return

    // Check if it's a number (block number)
    if (/^\d+$/.test(query)) {
      onSearchBlock(Number.parseInt(query))
    }
    // Otherwise assume it's a transaction hash
    else if (query.length > 10) {
      onSearchTx(query)
    }

    setQuery("")
  }

  return (
    <div className="border-b border-border bg-card">
      <div className="container mx-auto px-4 py-4">
        <form onSubmit={handleSearch} className="flex gap-2">
          <Input
            type="text"
            placeholder="Search by block number (e.g., 12345) or transaction hash..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="flex-1 bg-card/50 text-foreground placeholder:text-muted-foreground/50"
          />
          <Button type="submit" variant="default" className="gap-2 whitespace-nowrap">
            🔎 Search
          </Button>
        </form>
      </div>
    </div>
  )
}
