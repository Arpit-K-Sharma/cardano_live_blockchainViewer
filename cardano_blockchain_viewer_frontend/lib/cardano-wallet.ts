// Cardano CIP-30 Wallet Integration
// Supports: Eternl, Lace, Typhoon, Yoroi, Nami, Flint, and more

export interface WalletApi {
  enable(): Promise<CardanoWalletApi>
  isEnabled(): Promise<boolean>
  name: string
  icon: string
  apiVersion: string
}

export interface CardanoWalletApi {
  getNetworkId(): Promise<number>
  getUsedAddresses(): Promise<string[]>
  getUnusedAddresses(): Promise<string[]>
  getChangeAddress(): Promise<string>
  getRewardAddresses(): Promise<string[]>
  signData(address: string, payload: string): Promise<DataSignature>
}

export interface DataSignature {
  signature: string  // hex-encoded COSE_Sign1
  key: string        // hex-encoded COSE_Key
}

export interface SupportedWallet {
  id: string
  name: string
  icon: string
  checkAvailable: () => boolean
  getApi: () => WalletApi | undefined
}

// Supported CIP-30 wallets
export const SUPPORTED_WALLETS: SupportedWallet[] = [
  {
    id: 'eternl',
    name: 'Eternl',
    icon: '🦋',
    checkAvailable: () => typeof window !== 'undefined' && 'cardano' in window && 'eternl' in (window as any).cardano,
    getApi: () => (window as any).cardano?.eternl,
  },
  {
    id: 'lace',
    name: 'Lace',
    icon: '🔷',
    checkAvailable: () => typeof window !== 'undefined' && 'cardano' in window && 'lace' in (window as any).cardano,
    getApi: () => (window as any).cardano?.lace,
  },
  {
    id: 'typhon',
    name: 'Typhon',
    icon: '🌊',
    checkAvailable: () => typeof window !== 'undefined' && 'cardano' in window && 'typhon' in (window as any).cardano,
    getApi: () => (window as any).cardano?.typhon,
  },
  {
    id: 'yoroi',
    name: 'Yoroi',
    icon: '💎',
    checkAvailable: () => typeof window !== 'undefined' && 'cardano' in window && 'yoroi' in (window as any).cardano,
    getApi: () => (window as any).cardano?.yoroi,
  },
  {
    id: 'nami',
    name: 'Nami',
    icon: '🦊',
    checkAvailable: () => typeof window !== 'undefined' && 'cardano' in window && 'nami' in (window as any).cardano,
    getApi: () => (window as any).cardano?.nami,
  },
  {
    id: 'flint',
    name: 'Flint',
    icon: '🔥',
    checkAvailable: () => typeof window !== 'undefined' && 'cardano' in window && 'flint' in (window as any).cardano,
    getApi: () => (window as any).cardano?.flint,
  },
]

/**
 * Get list of available wallets installed in the browser
 */
export function getAvailableWallets(): SupportedWallet[] {
  return SUPPORTED_WALLETS.filter(wallet => wallet.checkAvailable())
}

/**
 * Connect to a specific wallet
 */
export async function connectWallet(walletId: string): Promise<{
  api: CardanoWalletApi
  address: string
  stakeAddress: string | null
}> {
  const wallet = SUPPORTED_WALLETS.find(w => w.id === walletId)

  if (!wallet) {
    throw new Error(`Wallet ${walletId} is not supported`)
  }

  if (!wallet.checkAvailable()) {
    throw new Error(`${wallet.name} is not installed. Please install the extension.`)
  }

  const walletApi = wallet.getApi()
  if (!walletApi) {
    throw new Error(`Failed to get ${wallet.name} API`)
  }

  // Enable wallet (requests user permission)
  const api = await walletApi.enable()

  // Get wallet address
  const usedAddresses = await api.getUsedAddresses()
  const unusedAddresses = await api.getUnusedAddresses()

  const addressHex = usedAddresses[0] || unusedAddresses[0]
  if (!addressHex) {
    throw new Error('No addresses found in wallet')
  }

  // Convert hex address to bech32
  const address = hexToBech32(addressHex)

  // Get stake address (optional)
  let stakeAddress: string | null = null
  try {
    const rewardAddresses = await api.getRewardAddresses()
    if (rewardAddresses && rewardAddresses.length > 0) {
      stakeAddress = hexToBech32(rewardAddresses[0])
    }
  } catch (error) {
    console.warn('Failed to get stake address:', error)
  }

  return { api, address, stakeAddress }
}

/**
 * Sign data with wallet (CIP-30 signData)
 */
export async function signData(
  api: CardanoWalletApi,
  address: string,
  message: string
): Promise<DataSignature> {
  // Most modern Cardano wallets (Eternl, Lace, etc.) support bech32 addresses directly
  // per CIP-30 spec, so we pass the address as-is

  // Convert message to hex
  const messageHex = Buffer.from(message, 'utf-8').toString('hex')

  // Sign with wallet - use bech32 address directly
  const signature = await api.signData(address, messageHex)

  return signature
}

/**
 * Convert hex address to bech32 format
 */
function hexToBech32(hex: string): string {
  // This is a simplified version - in production use @emurgo/cardano-serialization-lib-browser
  try {
    const bytes = Buffer.from(hex, 'hex')
    // For now, return the hex with prefix (frontend will handle it)
    // In real implementation, use proper bech32 encoding
    return `addr1${bytes.toString('hex').slice(0, 50)}`
  } catch {
    return hex
  }
}

/**
 * Convert bech32 address to hex format
 */
function bech32ToHex(address: string): string {
  // This is a simplified version - in production use @emurgo/cardano-serialization-lib-browser
  try {
    // For now, just return the part after the prefix
    return address.replace(/^addr[_0-9a-z]+/, '')
  } catch {
    return address
  }
}

/**
 * Check if user has a wallet extension installed
 */
export function hasAnyWallet(): boolean {
  return getAvailableWallets().length > 0
}
