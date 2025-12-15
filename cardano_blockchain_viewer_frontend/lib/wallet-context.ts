'use client'

import React, { createContext, useContext, useState, useEffect } from 'react'

declare global {
  interface Window {
    cardano?: {
      eternl?: WalletAPI
      nami?: WalletAPI
      lace?: WalletAPI
    }
  }
}
