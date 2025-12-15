"use client"

import type React from "react"

import { useState } from "react"

interface CopyButtonProps {
  text: string
  displayText?: string
  className?: string
  onClick?: (e: React.MouseEvent) => void
}

export function CopyButton({ text, displayText, className = "", onClick }: CopyButtonProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation()
    if (onClick) onClick(e)
    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (error) {
      console.error("Failed to copy:", error)
    }
  }

  return (
    <span
      onClick={handleCopy}
      className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs bg-accent/10 text-accent hover:bg-accent/20 transition-colors cursor-pointer font-mono ${copied ? "bg-green-500/20 text-green-400" : ""} ${className}`}
      title="Click to copy"
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          handleCopy(e as any)
        }
      }}
    >
      {copied ? "✓ Copied!" : displayText || text.slice(0, 12) + "..."}
    </span>
  )
}
