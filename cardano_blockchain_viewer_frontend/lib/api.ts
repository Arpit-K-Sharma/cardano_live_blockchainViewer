// API utilities for backend authentication

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'

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
  page: number = 1,
  count: number = 20
): Promise<TransactionResponse> {
  const response = await fetch(
    `${API_BASE_URL}/api/user/transactions?page=${page}&count=${count}`,
    {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    }
  )

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Failed to fetch transactions' }))
    throw new Error(error.error || 'Failed to fetch transactions')
  }

  return response.json()
}

/**
 * Get user's wallet summary (requires authentication)
 */
export async function getUserSummary(token: string): Promise<WalletSummary> {
  const response = await fetch(`${API_BASE_URL}/api/user/summary`, {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
  })

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Failed to fetch wallet summary' }))
    throw new Error(error.error || 'Failed to fetch wallet summary')
  }

  return response.json()
}