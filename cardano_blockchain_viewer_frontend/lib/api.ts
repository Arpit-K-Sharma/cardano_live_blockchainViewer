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
