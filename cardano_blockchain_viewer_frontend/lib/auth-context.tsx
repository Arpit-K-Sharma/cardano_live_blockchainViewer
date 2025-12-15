"use client"

import { createContext, useContext, useState, useEffect, type ReactNode } from 'react'
import type { CardanoWalletApi } from './cardano-wallet'

interface AuthContextType {
  isAuthenticated: boolean
  isLoading: boolean
  address: string | null
  stakeAddress: string | null
  walletApi: CardanoWalletApi | null
  walletName: string | null
  token: string | null
  login: (walletApi: CardanoWalletApi, address: string, stakeAddress: string | null, token: string, walletName: string) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

interface AuthProviderProps {
  children: ReactNode
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [isAuthenticated, setIsAuthenticated] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [address, setAddress] = useState<string | null>(null)
  const [stakeAddress, setStakeAddress] = useState<string | null>(null)
  const [walletApi, setWalletApi] = useState<CardanoWalletApi | null>(null)
  const [walletName, setWalletName] = useState<string | null>(null)
  const [token, setToken] = useState<string | null>(null)

  // Load auth state from localStorage on mount
  useEffect(() => {
    try {
      const savedToken = localStorage.getItem('cardano_auth_token')
      const savedAddress = localStorage.getItem('cardano_auth_address')
      const savedStakeAddress = localStorage.getItem('cardano_auth_stake_address')
      const savedWalletName = localStorage.getItem('cardano_wallet_name')

      if (savedToken && savedAddress) {
        setToken(savedToken)
        setAddress(savedAddress)
        setStakeAddress(savedStakeAddress)
        setWalletName(savedWalletName)
        setIsAuthenticated(true)

        // Note: walletApi is not persisted, user would need to reconnect wallet
        // if they refresh the page
      }
    } catch (error) {
      console.error('Failed to load auth state:', error)
    } finally {
      setIsLoading(false)
    }
  }, [])

  const login = (
    newWalletApi: CardanoWalletApi,
    newAddress: string,
    newStakeAddress: string | null,
    newToken: string,
    newWalletName: string
  ) => {
    setWalletApi(newWalletApi)
    setAddress(newAddress)
    setStakeAddress(newStakeAddress)
    setToken(newToken)
    setWalletName(newWalletName)
    setIsAuthenticated(true)

    // Save to localStorage
    try {
      localStorage.setItem('cardano_auth_token', newToken)
      localStorage.setItem('cardano_auth_address', newAddress)
      if (newStakeAddress) {
        localStorage.setItem('cardano_auth_stake_address', newStakeAddress)
      }
      localStorage.setItem('cardano_wallet_name', newWalletName)
    } catch (error) {
      console.error('Failed to save auth state:', error)
    }
  }

  const logout = () => {
    setWalletApi(null)
    setAddress(null)
    setStakeAddress(null)
    setToken(null)
    setWalletName(null)
    setIsAuthenticated(false)

    // Clear localStorage
    try {
      localStorage.removeItem('cardano_auth_token')
      localStorage.removeItem('cardano_auth_address')
      localStorage.removeItem('cardano_auth_stake_address')
      localStorage.removeItem('cardano_wallet_name')
    } catch (error) {
      console.error('Failed to clear auth state:', error)
    }
  }

  return (
    <AuthContext.Provider
      value={{
        isAuthenticated,
        isLoading,
        address,
        stakeAddress,
        walletApi,
        walletName,
        token,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}
