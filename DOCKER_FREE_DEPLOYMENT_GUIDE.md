# Cardano Blockchain Viewer - Docker-Free Deployment Guide

This comprehensive guide covers deploying your Cardano Blockchain Viewer using Docker-free methods: **Vercel** for frontend and **Render** for backend.

## 🚀 Overview

Docker-free deployment uses platform-native build systems to deploy your application without Docker containers. This method is recommended for developers who want:

- Faster deployment times
- Lower resource usage
- Automatic scaling and management
- Simplified maintenance

## Project Structure Overview

- **Backend**: Rust/Axum server running on port 8080
- **Frontend**: Next.js application with TypeScript
- **Real-time**: WebSocket support for live blockchain data
- **Authentication**: JWT-based authentication system

## Prerequisites

- [Node.js 18+](https://nodejs.org/) (for frontend development)
- [Rust](https://rustup.rs/) (for backend development)
- [Git](https://git-scm.com/)
- Accounts on [Vercel](https://vercel.com) and [Render](https://render.com)

---

## 🛠️ Configuration Steps

### Step 1: Backend Configuration for Render

#### 1.1 Create Render Configuration

The `render.yaml` file has been created with the following configuration:

```yaml
services:
  - type: web
    name: cardano-backend
    env: rust
    buildCommand: cargo build --release
    startCommand: ./target/release/cardano_blockchain_viewer
    envVars:
      - key: BLOCKFROST_API_KEY
        sync: false
      - key: JWT_SECRET
        sync: false
      - key: RUST_LOG
        value: info
      - key: PORT
        value: 10000
    healthCheckPath: /health
```

#### 1.2 Health Check Endpoint

A health check endpoint has been added to the backend at `/health` for Render deployment monitoring.

#### 1.3 Environment Variables Template

Create `.env.example` in the backend directory:

```bash
# Blockfrost API Key (Required)
# Get your API key from: https://blockfrost.io/
BLOCKFROST_API_KEY=your_blockfrost_api_key_here

# JWT Secret (Required)
# Use a strong, unique secret for production (256-bit recommended)
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production

# Rust Log Level (Optional)
# Options: trace, debug, info, warn, error
RUST_LOG=info

# Server Port (Optional - auto-set by hosting platforms)
# PORT=8080
```

### Step 2: Frontend Configuration for Vercel

#### 2.1 Update Next.js Configuration

The `next.config.ts` has been updated for Vercel deployment:

```typescript
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
  env: {
    NEXT_PUBLIC_API_URL:
      process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080",
    NEXT_PUBLIC_WS_URL: process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:8080",
  },
};

export default nextConfig;
```

#### 2.2 Create Vercel Configuration

The `vercel.json` file has been created with the following configuration:

```json
{
  "version": 2,
  "builds": [
    {
      "src": "package.json",
      "use": "@vercel/next"
    }
  ],
  "routes": [
    {
      "src": "/(.*)",
      "dest": "/$1"
    }
  ],
  "env": {
    "NEXT_PUBLIC_API_URL": "@next_public_api_url",
    "NEXT_PUBLIC_WS_URL": "@next_public_ws_url"
  },
  "build": {
    "env": {
      "NEXT_PUBLIC_API_URL": "@next_public_api_url",
      "NEXT_PUBLIC_WS_URL": "@next_public_ws_url"
    }
  }
}
```

#### 2.3 Environment Variables Template

Create `.env.example` in the frontend directory:

```bash
# Backend API URL (Required)
# Replace with your deployed backend URL
NEXT_PUBLIC_API_URL=http://localhost:8080

# WebSocket URL (Required)
# Replace with your deployed backend WebSocket URL
NEXT_PUBLIC_WS_URL=ws://localhost:8080
```

---

## 🚀 Deployment Steps

### Step 1: Deploy Backend to Render

#### 1.1 Create Render Account

1. Sign up at [Render.com](https://render.com)
2. Connect your GitHub repository

#### 1.2 Deploy Backend Service

1. **Create New Web Service**:

   - Click "New" → "Web Service"
   - Connect your GitHub repository
   - Select the `cardano_blockchain_viewer` folder
   - Choose "Web Service"

2. **Configure Build and Deploy**:

   - **Environment**: `Rust`
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/cardano_blockchain_viewer`
   - **Root Directory**: Leave empty (or specify `cardano_blockchain_viewer`)

3. **Environment Variables**:

   ```
   BLOCKFROST_API_KEY=your_blockfrost_api_key
   JWT_SECRET=your_jwt_secret_key
   RUST_LOG=info
   PORT=10000
   ```

4. **Deploy**:

   - Click "Create Web Service"
   - Wait for deployment to complete (usually 5-10 minutes)
   - Note the service URL (e.g., `https://cardano-backend.onrender.com`)

5. **Test Health Check**:
   ```
   https://your-backend-url.onrender.com/health
   ```

### Step 2: Deploy Frontend to Vercel

#### 2.1 Create Vercel Account

1. Sign up at [Vercel.com](https://vercel.com)
2. Connect your GitHub repository

#### 2.2 Deploy Frontend Service

1. **Import Project**:

   - Click "New Project"
   - Import your GitHub repository
   - Select the `cardano_blockchain_viewer_frontend` folder

2. **Configure Build Settings**:

   - **Framework Preset**: `Next.js`
   - **Root Directory**: `cardano_blockchain_viewer_frontend`
   - **Build Command**: `npm run build` (auto-detected)
   - **Output Directory**: Leave empty (defaults to `out` for static export)
   - **Install Command**: `npm install` (auto-detected)

3. **Environment Variables**:

   ```
   NEXT_PUBLIC_API_URL=https://your-backend-url.onrender.com
   NEXT_PUBLIC_WS_URL=wss://your-backend-url.onrender.com
   ```

4. **Deploy**:
   - Click "Deploy"
   - Wait for deployment to complete (usually 2-5 minutes)
   - Note the frontend URL (e.g., `https://your-frontend.vercel.app`)

### Step 3: Test and Configure

#### 3.1 Update CORS Configuration

The backend is configured to allow all origins for development. For production, consider restricting CORS:

```rust
use tower_http::cors::{Any, CorsLayer};

// In main.rs, replace the current CORS configuration with:
let cors = CorsLayer::new()
    .allow_origin("https://your-frontend.vercel.app".parse()?)
    .allow_methods(Any)
    .allow_headers(Any);
```

#### 3.2 Test the Deployment

1. **Backend Health Check**:

   ```
   https://your-backend-url.onrender.com/health
   ```

2. **Frontend Application**:

   ```
   https://your-frontend.vercel.app
   ```

3. **Test API Endpoints**:

   ```
   https://your-backend-url.onrender.com/api/auth/challenge
   ```

4. **Test WebSocket Connection**:
   - Open browser console on the frontend
   - Check for WebSocket connection errors
   - Verify real-time data flow

#### 3.3 Update Environment Variables

After successful deployment, update frontend environment variables with the actual URLs:

1. **In Vercel Dashboard**:

   - Go to your project settings
   - Update `NEXT_PUBLIC_API_URL` to your Render backend URL
   - Update `NEXT_PUBLIC_WS_URL` to use `wss://` instead of `https://`

2. **Redeploy Frontend**:
   - Vercel will automatically redeploy after environment variable changes

---

## 🔧 Local Development Testing

### Test Docker Compose Setup

Before deploying, test locally with Docker Compose:

```bash
# From the root directory
docker-compose up --build

# Test the applications:
# Frontend: http://localhost:3000
# Backend API: http://localhost:8080
# WebSocket: ws://localhost:8080/ws
# Health Check: http://localhost:8080/health

# Stop the services
docker-compose down
```

### Test Without Docker

Test individual components locally:

**Backend Testing**:

```bash
cd cardano_blockchain_viewer
cargo run

# Test health endpoint
curl http://localhost:8080/health
```

**Frontend Testing**:

```bash
cd cardano_blockchain_viewer_frontend
npm install
npm run dev

# Test the application
# Frontend: http://localhost:3000
```

---

## 🔍 Monitoring and Maintenance

### Backend Monitoring (Render)

1. **View Logs**: Dashboard → Your Service → Logs
2. **Metrics**: Dashboard → Your Service → Metrics
3. **Auto-scaling**: Configure in service settings
4. **Environment Variables**: Manage in service settings

### Frontend Monitoring (Vercel)

1. **View Analytics**: Dashboard → Your Project → Analytics
2. **Function Logs**: Dashboard → Your Project → Functions
3. **Deployment History**: Dashboard → Your Project → Deployments
4. **Environment Variables**: Manage in project settings

### Regular Updates

**Backend Updates**:

```bash
git add .
git commit -m "Update backend"
git push origin main
# Render will automatically redeploy
```

**Frontend Updates**:

```bash
git add .
git commit -m "Update frontend"
git push origin main
# Vercel will automatically redeploy
```

---

## 🛡️ Security Considerations

### Environment Variables

- Never commit sensitive keys to version control
- Use strong, unique secrets for JWT tokens
- Rotate API keys regularly
- Use different secrets for development and production

### CORS Configuration

- Restrict CORS origins in production to your frontend domain
- Avoid using `Any` origin in production environments
- Consider implementing rate limiting

### HTTPS and WebSockets

- Ensure all services use HTTPS in production
- WebSocket connections should use `wss://` for secure connections
- Vercel and Render automatically provide HTTPS

---

## 🐛 Troubleshooting

### Common Issues

1. **Build Failures**:

   - Ensure all dependencies are properly specified
   - Check environment variables are set correctly
   - Verify build commands are appropriate for the platform

2. **CORS Errors**:

   - Check that backend CORS allows your frontend domain
   - Verify environment variables for correct URLs
   - Ensure WebSocket URLs use `wss://` for HTTPS

3. **WebSocket Connection Issues**:

   - Verify WebSocket endpoint is accessible
   - Check that firewall/security groups allow WebSocket traffic
   - Ensure backend WebSocket server is properly configured

4. **Environment Variable Issues**:
   - Double-check variable names (case-sensitive)
   - Ensure variables are set in the correct deployment environment
   - Verify quotes are properly escaped

### Getting Help

- **Render Documentation**: [render.com/docs](https://render.com/docs)
- **Vercel Documentation**: [vercel.com/docs](https://vercel.com/docs)
- **Next.js Documentation**: [nextjs.org/docs](https://nextjs.org/docs)
- **Axum Documentation**: [docs.rs/axum](https://docs.rs/axum)

---

## 📊 Performance Optimization

### Backend Optimization

1. **Rust Optimizations**:

   - Use release builds (`cargo build --release`)
   - Enable compiler optimizations
   - Consider using `tokio` with appropriate features

2. **Database and Caching**:
   - Implement caching for frequently accessed data
   - Consider using Redis for session storage
   - Optimize database queries

### Frontend Optimization

1. **Next.js Optimizations**:

   - Enable static generation where possible
   - Implement proper image optimization
   - Use Next.js built-in performance features

2. **Bundle Optimization**:
   - Use dynamic imports for large components
   - Optimize WebSocket connection handling
   - Implement proper error boundaries

---

## 🎯 Next Steps

1. **Set up custom domains** for both services
2. **Configure SSL certificates** (automatic with Vercel and Render)
3. **Implement monitoring and alerting**
4. **Set up automated testing** in CI/CD
5. **Consider database integration** for user data
6. **Implement proper logging and metrics collection**

---

_Last updated: 2024_
