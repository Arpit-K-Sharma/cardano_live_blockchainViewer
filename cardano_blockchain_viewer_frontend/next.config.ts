import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'standalone', 

  env: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL || 'https://cardanoliveblockchainviewer-production.up.railway.app',
    NEXT_PUBLIC_WS_URL: process.env.NEXT_PUBLIC_WS_URL || 'wss://cardanoliveblockchainviewer-production.up.railway.app',
  },
};

export default nextConfig;