# Cardano Blockchain Viewer - Docker-Based Deployment Guide

This comprehensive guide covers deploying your Cardano Blockchain Viewer using Docker containers on Railways platform.

## 🚀 Overview

Docker-based deployment uses containerization to package your application with all its dependencies, ensuring consistent deployment across different environments. This method is recommended for developers who want:

- Consistent development and production environments
- Easy scaling and management
- Simple deployment process
- Better resource utilization

## Project Structure Overview

- **Backend**: Rust/Axum server running on port 8080
- **Frontend**: Next.js application with TypeScript
- **Real-time**: WebSocket support for live blockchain data
- **Authentication**: JWT-based authentication system

## Prerequisites

- [Docker](https://www.docker.com/get-started) (for Docker-based deployment)
- [Node.js 18+](https://nodejs.org/) (for Docker-free deployment)
- [Rust](https://rustup.rs/) (for local development)
- [Git](https://git-scm.com/)

---

## Method 1: Docker-Based Deployment (Railways)

This method uses Docker containers deployed on Railways platform.

### Step 1: Prepare Docker Configuration

#### Backend Dockerfile

Create `cardano_blockchain_viewer/Dockerfile`:

```dockerfile
# Backend Dockerfile
FROM rust:1.70 as builder

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY src ./src
COPY daemon.toml ./

# Build the application
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/cardano_blockchain_viewer ./cardano_blockchain_viewer
COPY --from=builder /app/daemon.toml ./daemon.toml

# Create non-root user
RUN useradd -r -s /bin/false appuser
RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

CMD ["./cardano_blockchain_viewer"]
```

#### Frontend Dockerfile

Create `cardano_blockchain_viewer_frontend/Dockerfile`:

```dockerfile
# Frontend Dockerfile
FROM node:18-alpine as builder

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY next.config.ts ./
COPY tsconfig.json ./
COPY tailwind.config.mjs ./
COPY postcss.config.mjs ./
COPY components.json ./

# Install dependencies
RUN npm ci --only=production

# Copy source code
COPY . .

# Build the application
RUN npm run build

# Production stage
FROM node:18-alpine as runner

WORKDIR /app

# Install dumb-init for proper signal handling
RUN apk add --no-cache dumb-init

# Copy built application
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

# Create non-root user
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

USER nextjs

EXPOSE 3000

ENV PORT 3000
ENV HOSTNAME "0.0.0.0"

# Use dumb-init for proper signal handling
ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "server.js"]
```

#### Docker Compose for Local Development

Create `docker-compose.yml` in the root directory:

```yaml
version: "3.8"

services:
  backend:
    build:
      context: ./cardano_blockchain_viewer
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - BLOCKFROST_API_KEY=${BLOCKFROST_API_KEY}
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=info
    volumes:
      - ./cardano_blockchain_viewer/daemon.toml:/app/daemon.toml
    restart: unless-stopped
    networks:
      - cardano-network

  frontend:
    build:
      context: ./cardano_blockchain_viewer_frontend
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://localhost:8080
      - NEXT_PUBLIC_WS_URL=ws://localhost:8080
    depends_on:
      - backend
    restart: unless-stopped
    networks:
      - cardano-network

networks:
  cardano-network:
    driver: bridge
```

#### .dockerignore files

Create `.dockerignore` in backend directory:

```
target
.git
.gitignore
README.md
.env
.env.local
.env.production.local
.env.development.local
.env.test.local
```

Create `.dockerignore` in frontend directory:

```
node_modules
.next
.git
.gitignore
.env
.env.local
.env.production.local
.env.development.local
.env.test.local
npm-debug.log*
yarn-debug.log*
yarn-error.log*
```

### Step 2: Test Docker Setup Locally

1. **Build and run with Docker Compose**:

   ```bash
   # From the root directory
   docker-compose up --build

   # Or run in detached mode
   docker-compose up --build -d
   ```

2. **Test the applications**:

   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8080
   - WebSocket: ws://localhost:8080/ws

3. **Stop the services**:
   ```bash
   docker-compose down
   ```

### Step 3: Deploy to Railways

#### 3.1 Create Railway Account and Install CLI

1. Sign up at [Railway.app](https://railway.app)
2. Install Railway CLI:
   ```bash
   npm install -g @railway/cli
   ```

#### 3.2 Initialize Railway Projects

**Backend Deployment**:

1. Navigate to backend directory:

   ```bash
   cd cardano_blockchain_viewer
   ```

2. Initialize Railway project:

   ```bash
   railway login
   railway init
   ```

3. Create `railway.json`:

   ```json
   {
     "build": {
       "builder": "DOCKERFILE",
       "dockerfilePath": "Dockerfile"
     },
     "deploy": {
       "startCommand": "./cardano_blockchain_viewer",
       "healthcheckPath": "/health",
       "healthcheckTimeout": 300,
       "restartPolicyType": "ON_FAILURE"
     }
   }
   ```

4. Add environment variables:

   ```bash
   railway variables set BLOCKFROST_API_KEY=your_blockfrost_api_key
   railway variables set JWT_SECRET=your_jwt_secret_key
   railway variables set RUST_LOG=info
   ```

5. Deploy:
   ```bash
   railway up
   ```

**Frontend Deployment**:

1. Navigate to frontend directory:

   ```bash
   cd ../cardano_blockchain_viewer_frontend
   ```

2. Initialize Railway project:

   ```bash
   railway init
   ```

3. Create `railway.json`:

   ```json
   {
     "build": {
       "builder": "DOCKERFILE",
       "dockerfilePath": "Dockerfile"
     },
     "deploy": {
       "startCommand": "node server.js",
       "healthcheckPath": "/",
       "healthcheckTimeout": 300,
       "restartPolicyType": "ON_FAILURE"
     }
   }
   ```

4. Update Next.js config for Railway:

   ```javascript
   // next.config.ts
   const nextConfig = {
     output: "standalone",
     experimental: {
       outputFileTracingRoot: path.join(__dirname, "../../"),
     },
     env: {
       NEXT_PUBLIC_API_URL:
         process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080",
       NEXT_PUBLIC_WS_URL:
         process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:8080",
     },
   };
   ```

5. Add environment variables:

   ```bash
   railway variables set NEXT_PUBLIC_API_URL=https://your-backend-url.railway.app
   railway variables set NEXT_PUBLIC_WS_URL=wss://your-backend-url.railway.app
   ```

6. Deploy:
   ```bash
   railway up
   ```

#### 3.3 Configure Domain and SSL

1. In Railway dashboard, add custom domains for both services
2. Enable automatic HTTPS certificates
3. Update environment variables with new domain URLs

### Step 4: Monitor and Maintain

1. **Monitor logs**:

   ```bash
   railway logs
   ```

2. **Check service status**:

   ```bash
   railway status
   ```

3. **Update services**:
   ```bash
   git add .
   git commit -m "Update deployment"
   railway up
   ```

---

## Method 2: Docker-Free Deployment (Vercel + Render)

This method deploys the frontend to Vercel and backend to Render without Docker.

### Step 1: Prepare Frontend for Vercel

#### 1.1 Configure Next.js for Vercel

Update `next.config.ts`:

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

#### 1.2 Create Vercel Configuration

Create `vercel.json` in frontend directory:

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

#### 1.3 Prepare Build Scripts

Update `package.json` scripts:

```json
{
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "eslint .",
    "export": "next export"
  }
}
```

### Step 2: Prepare Backend for Render

#### 2.1 Create Render Configuration

Create `render.yaml` in backend directory:

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

#### 2.2 Create Health Check Endpoint

Add to `src/main.rs`:

```rust
use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};

// Add health check route
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... existing code ...

    let api_router = api::create_router(jwt_manager, blockfrost, ws_state)
        .route("/health", get(health_check));

    // ... rest of the code ...
}
```

### Step 3: Deploy Backend to Render

#### 3.1 Create Render Account

1. Sign up at [Render.com](https://render.com)
2. Connect your GitHub repository

#### 3.2 Deploy Backend Service

1. **Create New Web Service**:

   - Connect your GitHub repository
   - Select the `cardano_blockchain_viewer` folder
   - Choose "Web Service"

2. **Configure Build and Deploy**:

   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/cardano_blockchain_viewer`
   - **Environment**: `Rust`

3. **Environment Variables**:

   ```
   BLOCKFROST_API_KEY=your_blockfrost_api_key
   JWT_SECRET=your_jwt_secret_key
   RUST_LOG=info
   PORT=10000
   ```

4. **Deploy**:
   - Click "Create Web Service"
   - Wait for deployment to complete
   - Note the service URL (e.g., `https://cardano-backend.onrender.com`)

### Step 4: Deploy Frontend to Vercel

#### 4.1 Create Vercel Account

1. Sign up at [Vercel.com](https://vercel.com)
2. Connect your GitHub repository

#### 4.2 Deploy Frontend Service

1. **Import Project**:

   - Click "New Project"
   - Import your GitHub repository
   - Select the `cardano_blockchain_viewer_frontend` folder

2. **Configure Build Settings**:

   - **Framework Preset**: Next.js
   - **Build Command**: `npm run build`
   - **Output Directory**: `out` (for static export)
   - **Install Command**: `npm install`

3. **Environment Variables**:

   ```
   NEXT_PUBLIC_API_URL=https://your-render-backend-url.onrender.com
   NEXT_PUBLIC_WS_URL=wss://your-render-backend-url.onrender.com
   ```

4. **Deploy**:
   - Click "Deploy"
   - Wait for deployment to complete
   - Note the frontend URL

### Step 5: Test and Configure

#### 5.1 Update CORS Configuration

Update backend CORS settings in `src/main.rs`:

```rust
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... existing code ...

    // Add CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_router = api::create_router(jwt_manager, blockfrost, ws_state)
        .route("/health", get(health_check))
        .layer(cors);

    // ... rest of the code ...
}
```

#### 5.2 Test the Deployment

1. **Backend Health Check**:

   ```
   https://your-backend-url.onrender.com/health
   ```

2. **Frontend Application**:

   ```
   https://your-frontend-url.vercel.app
   ```

3. **Test WebSocket Connection**:
   - Open browser console
   - Connect to WebSocket endpoint
   - Verify real-time data flow

### Step 6: Monitor and Maintain

#### 6.1 Backend Monitoring (Render)

1. **View Logs**: Dashboard → Your Service → Logs
2. **Metrics**: Dashboard → Your Service → Metrics
3. **Auto-scaling**: Configure in service settings

#### 6.2 Frontend Monitoring (Vercel)

1. **View Analytics**: Dashboard → Your Project → Analytics
2. **Function Logs**: Dashboard → Your Project → Functions
3. **Deployment History**: Dashboard → Your Project → Deployments

#### 6.3 Update Deployment

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

## Environment Variables Reference

### Backend Environment Variables

| Variable             | Description                         | Required | Example               |
| -------------------- | ----------------------------------- | -------- | --------------------- |
| `BLOCKFROST_API_KEY` | Blockfrost API key for Cardano data | Yes      | `your_blockfrost_key` |
| `JWT_SECRET`         | Secret key for JWT token signing    | Yes      | `your-256-bit-secret` |
| `RUST_LOG`           | Logging level                       | No       | `info`                |
| `PORT`               | Server port (Render auto-sets)      | No       | `10000`               |

### Frontend Environment Variables

| Variable              | Description     | Required | Example                            |
| --------------------- | --------------- | -------- | ---------------------------------- |
| `NEXT_PUBLIC_API_URL` | Backend API URL | Yes      | `https://backend-url.onrender.com` |
| `NEXT_PUBLIC_WS_URL`  | WebSocket URL   | Yes      | `wss://backend-url.onrender.com`   |

## Troubleshooting

### Common Issues

1. **CORS Errors**:

   - Ensure backend CORS is configured to allow your frontend domain
   - Check environment variables for correct URLs

2. **WebSocket Connection Failed**:

   - Verify WebSocket URL uses `wss://` for HTTPS
   - Check backend WebSocket endpoint is accessible

3. **Build Failures**:

   - Ensure all dependencies are properly specified
   - Check environment variables are set
   - Verify build commands are correct

4. **Environment Variable Issues**:
   - Double-check variable names (case-sensitive)
   - Ensure quotes are properly escaped
   - Verify variables are set in correct deployment environments

### Getting Help

- **Railways**: Check [Railways Documentation](https://docs.railway.app)
- **Vercel**: Check [Vercel Documentation](https://vercel.com/docs)
- **Render**: Check [Render Documentation](https://render.com/docs)

---

## Security Considerations

1. **Environment Variables**:

   - Never commit sensitive keys to version control
   - Use strong, unique secrets for JWT
   - Rotate API keys regularly

2. **CORS Configuration**:

   - Restrict CORS origins in production
   - Use specific domain instead of `Any`

3. **HTTPS**:

   - Ensure all services use HTTPS in production
   - WebSocket connections should use `wss://`

4. **Rate Limiting**:
   - Implement rate limiting for API endpoints
   - Monitor for abuse patterns

---

_Last updated: 2024_
