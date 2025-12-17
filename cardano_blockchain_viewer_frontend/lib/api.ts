// API utilities for backend authentication


const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'https://cardanoliveblockchainviewer-production.up.railway.app';

export interface ChallengeResponse {
  message: string
  nonce: string
}

export interface VerifyResponse {
  token: string
}

/**
 * Request a challenge from the backend
 */
export async function requestChallenge(address: string): Promise<ChallengeResponse> {
  const response = await fetch(`${API_BASE_URL}/api/auth/challenge`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ address }),
  })

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Failed to request challenge' }))
    throw new Error(error.error || 'Failed to request challenge')
  }

  return response.json()
}

/**
 * Verify signature and get JWT token
 */
export async function verifySignature(
  address: string,
  signature: string,
  key: string,
  stakeAddress?: string | null
): Promise<VerifyResponse> {
  const response = await fetch(`${API_BASE_URL}/api/auth/verify`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      address,
      signature,
      key,
      stake_address: stakeAddress || undefined,
    }),
  })

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Signature verification failed' }))
    throw new Error(error.error || 'Signature verification failed')
  }

  return response.json()
}

// User transaction and wallet data types
export interface Transaction {
  tx_hash: string
  block: string
  block_height: number
  block_time: number
  slot: number
  index: number
  fees: string
}

export interface TransactionResponse {
  transactions: Transaction[]
  total: number
  page: number
}

export interface WalletSummary {
  address: string
  stake_address: string | null
  balance: string
  transaction_count: number
}

/**
 * Get user's transaction history (requires authentication)
 */
export async function getUserTransactions(
  token: string,
  address: string,
  page: number = 1,
  count: number = 20
): Promise<TransactionResponse> {
  const response = await fetch(
    `${API_BASE_URL}/api/user/transactions?address=${encodeURIComponent(address)}&page=${page}&count=${count}`,
    {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    }
  )

  if (!response.ok) {
    // Treat 404 (new/empty wallet) as empty history
    if (response.status === 404) {
      return { transactions: [], total: 0, page }
    }

    const contentType = response.headers.get('content-type')
    let errorMessage = 'Failed to fetch transactions'
    
    if (contentType?.includes('application/json')) {
      try {
        const error = await response.json()
        // Blockfrost-style not found message
        if (error?.message?.includes('component has not been found')) {
          return { transactions: [], total: 0, page }
        }
        errorMessage = error.error || errorMessage
      } catch {
        // JSON parsing failed, use default message
      }
    } else {
      // Non-JSON response (likely HTML error page)
      const text = await response.text().catch(() => '')
      if (text.includes('<!DOCTYPE html>') || text.includes('<html')) {
        errorMessage = `Server returned HTML instead of JSON. This usually means the backend server is not running or encountered an error. Status: ${response.status}`
      } else {
        errorMessage = `Server error: ${response.status}. Response: ${text.substring(0, 200)}`
      }
    }
    
    throw new Error(errorMessage)
  }

  return response.json()
}

/**
 * Get user's wallet summary (requires authentication)
 */
export async function getUserSummary(token: string, address: string): Promise<WalletSummary> {
  const response = await fetch(`${API_BASE_URL}/api/user/summary?address=${encodeURIComponent(address)}`, {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
  })

  if (!response.ok) {
    // Treat 404 (new/empty wallet) as zero balance/tx
    if (response.status === 404) {
      return {
        address,
        stake_address: null,
        balance: '0',
        transaction_count: 0,
      }
    }

    const contentType = response.headers.get('content-type')
    let errorMessage = 'Failed to fetch account info'
    
    if (contentType?.includes('application/json')) {
      try {
        const error = await response.json()
        if (error?.message?.includes('component has not been found')) {
          return {
            address,
            stake_address: null,
            balance: '0',
            transaction_count: 0,
          }
        }
        errorMessage = error.error || errorMessage
      } catch {
        // JSON parsing failed, use default message
      }
    } else {
      // Non-JSON response (likely HTML error page)
      const text = await response.text().catch(() => '')
      if (text.includes('<!DOCTYPE html>') || text.includes('<html')) {
        errorMessage = `Failed to fetch account info: Server returned HTML instead of JSON. This usually means the backend server is not running or encountered an error. Status: ${response.status}`
      } else {
        errorMessage = `Failed to fetch account info: Server error ${response.status}. Response: ${text.substring(0, 200)}`
      }
    }
    
    throw new Error(errorMessage)
  }

  return response.json()
}