/**
 * Address utility functions for Cardano address conversion
 */

import { bech32 } from 'bech32'

/**
 * Convert hex address to bech32 format
 * @param hexAddress - Hex-encoded address bytes
 * @returns Bech32 address string (addr_test1... or addr1...)
 */
export function hexToBech32(hexAddress: string): string {
  try {
    // Decode hex string to bytes
    const hexBytes = hexAddress.startsWith('0x') ? hexAddress.slice(2) : hexAddress
    const bytes = Buffer.from(hexBytes, 'hex')

    if (bytes.length === 0) {
      throw new Error('Invalid hex address: empty bytes')
    }

    // Determine network prefix based on address type
    // Cardano addresses: testnet uses "addr_test", mainnet uses "addr"
    // We'll use "addr_test" for testnet (PreProd) by default
    const prefix = 'addr_test'

    // Convert bytes to bech32
    const words = bech32.toWords(bytes)
    const bech32Address = bech32.encode(prefix, words)

    return bech32Address
  } catch (error) {
    console.error('Failed to convert hex to bech32:', error)
    throw new Error(`Address conversion failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Check if an address is in bech32 format
 */
export function isBech32(address: string): boolean {
  return address.startsWith('addr') || address.startsWith('stake')
}

/**
 * Normalize address to bech32 format
 * If already bech32, returns as-is. If hex, converts to bech32.
 */
export function normalizeToBech32(address: string): string {
  if (isBech32(address)) {
    return address
  }
  return hexToBech32(address)
}
