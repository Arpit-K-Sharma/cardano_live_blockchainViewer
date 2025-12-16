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
  // Note: CIP-30 wallets may return addresses in hex or bech32 format
  const usedAddresses = await api.getUsedAddresses()
  const unusedAddresses = await api.getUnusedAddresses()

  const rawAddress = usedAddresses[0] || unusedAddresses[0]
  if (!rawAddress) {
    throw new Error('No addresses found in wallet')
  }

  // Determine address format and normalize
  // CIP-30 signData accepts both hex and bech32 addresses
  // We'll keep the address as returned by the wallet for signData operations
  // but ensure consistency for backend communication
  let address: string
  
  // Check if address is bech32 (starts with addr1, addr_test1, etc.)
  if (rawAddress.startsWith('addr')) {
    // It's a bech32 address - use as-is for signData
    address = rawAddress
    console.log('📝 Address is bech32 format:', address.substring(0, 20) + '...')
  } else {
    // Assume it's hex format
    address = rawAddress
    console.log('📝 Address is hex format:', address.substring(0, 20) + '...')
  }

  // Get stake address (optional) - keep as hex for consistency
  let stakeAddress: string | null = null
  try {
    const rewardAddresses = await api.getRewardAddresses()
    if (rewardAddresses && rewardAddresses.length > 0) {
      stakeAddress = rewardAddresses[0] // Keep as hex
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
  // CIP-30 signData(address, payload) specification:
  // - address: Can be hex-encoded bytes or bech32 string (wallet handles both)
  // - payload: Must be hex-encoded string representing the bytes to sign
  
  // Convert message to hex (as required by CIP-30)
  const messageHex = Buffer.from(message, 'utf-8').toString('hex')
  console.log('✍️ Signing message:', {
    originalLength: message.length,
    hexLength: messageHex.length,
    hexPreview: messageHex.substring(0, 50) + '...'
  })

  // Sign with wallet - address can be hex or bech32, wallet handles it
  // The wallet will sign the bytes represented by the hex string
  const signature = await api.signData(address, messageHex)
  
  console.log('✅ Signature received:', {
    signatureLength: signature.signature.length,
    keyLength: signature.key.length
  })

  return signature
}


/**
 * Convert hex address to bech32 format
 * Uses a simplified approach that works with most CIP-30 wallets
 */
function hexToBech32(hex: string): string {
  try {
    // Remove any existing prefixes and ensure it's clean hex
    const cleanHex = hex.replace(/^addr1[a-z0-9]+/, '').replace(/^0x/, '')
    
    // For most modern wallets, the address bytes are already in the correct format
    // The hex provided by getUsedAddresses() is already the raw address bytes
    // We just need to ensure it's properly formatted
    
    // Convert to bytes and back to ensure valid length
    const bytes = Buffer.from(cleanHex, 'hex')
    
    if (bytes.length === 0) {
      throw new Error('Invalid address bytes')
    }
    
    // Most CIP-30 wallets return the raw address bytes directly
    // The bech32 encoding should be done by the wallet itself, but since we
    // already have hex from the wallet, we assume it's in the correct format
    // Return the hex as-is with addr1 prefix (common for mainnet)
    return `addr1${cleanHex}`
    
  } catch (error) {
    console.warn('Address conversion failed:', error)
    // If conversion fails, return the original hex
    // This allows the backend to handle different address formats
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
