import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'standalone', 
  env: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080' || 'https://cardanoliveblockchainviewer-production.up.railway.app',
    NEXT_PUBLIC_WS_URL: process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080' || 'wss://cardanoliveblockchainviewer-production.up.railway.app',
  },
};

export default nextConfig;