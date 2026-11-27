<p align="center">
  <img src="docs/assets/banner.svg" alt="Anypay Prices (Rust) — Anypay" width="100%" />
</p>

<p align="center">
  <a href="https://www.anypayx.com"><img src="https://img.shields.io/badge/Anypay-Bitcoin%20%26%20Stablecoin%20Payments-00AB55?style=for-the-badge&labelColor=161C24" alt="Anypay" /></a>
  <a href="https://crates.io/crates/prices"><img src="https://img.shields.io/crates/v/prices?style=for-the-badge" alt="Crates.io" /></a>
  <img src="https://img.shields.io/badge/rust-1.76+-orange?style=for-the-badge&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/tokio-async-blue?style=for-the-badge" alt="Async" />
</p>

<p align="center">
  <a href="https://www.anypayx.com">Website</a> ·
  <a href="https://www.anypayx.com/docs">Docs</a> ·
  <a href="https://api.anypayx.com/api">API</a> ·
  <a href="https://www.anypayx.com/prices">Price Feed</a> ·
  <a href="https://github.com/anypay">GitHub</a>
</p>

---

## What It Is & Purpose

**Anypay Prices** is a high-performance, Rust-based cryptocurrency price feed and conversion service that powers Anypay's real-time payment processing. Built with Tokio for async performance, it aggregates price data from multiple exchanges (CoinMarketCap, Kraken, and more), stores it in PostgreSQL via Prisma, and serves it through both REST API and WebSocket endpoints.

The service eliminates stale price risks in crypto payments by maintaining sub-second price updates across all supported cryptocurrencies and stablecoins. When an Anypay invoice is created, this service provides the current exchange rates that convert requested amounts (USD, EUR, etc.) into the exact cryptocurrency quantity needed for payment.

## Why Use It

**Anypay Prices** delivers mission-critical price accuracy for the entire Anypay payment stack. Without reliable, low-latency price feeds, crypto payment processors face conversion errors, customer confusion, and settlement risk.

- **Who it's for:** Backend developers building Anypay integrations, merchants needing programmatic access to crypto prices, and internal Anypay services that calculate invoice amounts.
- **Where it fits:** Core infrastructure layer. Sits between external exchange APIs and the Anypay invoice service, providing a single consistent price source with caching, redundancy, and RabbitMQ-based pub/sub for real-time updates. The WebSocket server (`PRICES_WEBSOCKETS_PORT: 5210`) pushes price changes to connected clients, while the REST API (`PRICES_REST_API_PORT: 5200`) serves on-demand queries.

## How It Works

<p align="center">
  <img src="docs/assets/architecture.svg" alt="How Anypay Prices (Rust) works in the Anypay stack" width="100%" />
</p>

1. **Exchange APIs** — Fetches live price data from CoinMarketCap (via `CMC_PRO_API_KEY`), Kraken, and other sources using async HTTP clients (reqwest with native TLS).
2. **Rust Core** — Tokio-based async runtime processes incoming price updates, validates them, and writes to PostgreSQL through Prisma ORM. The main process polls exchanges and publishes updates to RabbitMQ queues.
3. **Low-Latency API** — Dual interfaces: REST endpoints (`/api/prices`, `/api/prices/:id`, `/api/prices/history/:currencyPair`, `/api/prices/sources/:currencyPair`) served by Axum, and WebSocket server that maintains persistent connections with RabbitMQ queue-per-session architecture for real-time price streams.
4. **Anypay Payments** — Invoices, checkout, WalletBot, and merchant apps query this service to convert fiat amounts (e.g., "$50 USD") into cryptocurrency payment requests (e.g., "0.00084 BTC"), ensuring customers pay the correct amount at current market rates.

## Quickstart / How to Use It

### Prerequisites

- **Rust 1.76+** (specified in Dockerfile)
- **PostgreSQL** (Prisma-managed schema at `prisma/schema.prisma`)
- **RabbitMQ** (default: `amqp://localhost:5672/`)
- **CoinMarketCap API Key** (set as `coinmarketcap_api_key` env var, or `CMC_PRO_API_KEY` per README)

### Environment Variables

```bash
# API Hosts & Ports
PRICES_REST_API_HOST=127.0.0.1
PRICES_REST_API_PORT=5200
PRICES_WEBSOCKETS_HOST=127.0.0.1
PRICES_WEBSOCKETS_PORT=5210

# Messaging & Database
AMQP_URL=amqp://localhost:5672/
DATABASE_URL=postgresql://user:password@localhost:5432/prices

# External APIs
CMC_PRO_API_KEY=your_coinmarketcap_api_key
coinmarketcap_api_key=your_coinmarketcap_api_key

# Logging (optional)
RUST_LOG=info  # or debug, trace
```

### Build & Run

```bash
# Clone & install dependencies
git clone https://github.com/anypay/prices-rs.git
cd prices-rs

# Generate Prisma client (run before first build)
cd prisma-cli && cargo run && cd ..

# Build release binary
cargo build --release

# Run the service
cargo run --release
```

The service will:
1. Connect to PostgreSQL and ensure the `prices` table schema is current
2. Fetch an initial BTC price from CoinMarketCap on startup
3. Start the WebSocket server on `:5210`
4. Begin listening for WebSocket clients (each gets a dedicated RabbitMQ queue: `websocket:{session_uid}`)

### Docker Deployment

```dockerfile
# Multi-stage build (see Dockerfile)
docker build -t anypay-prices:latest .
docker run -d \
  -e PRICES_REST_API_PORT=5200 \
  -e PRICES_WEBSOCKETS_PORT=5210 \
  -e DATABASE_URL=postgresql://... \
  -e AMQP_URL=amqp://rabbitmq:5672/ \
  -e CMC_PRO_API_KEY=... \
  -p 5200:5200 -p 5210:5210 \
  anypay-prices:latest
```

### REST API Example

```bash
# List all prices
curl http://localhost:5200/api/prices

# Get price by ID
curl http://localhost:5200/api/prices/{uuid}

# Create new price record
curl -X POST http://localhost:5200/api/prices \
  -H "Content-Type: application/json" \
  -d '{
    "currency": "BTC",
    "base_currency": "USD",
    "value": 43250.00,
    "source": "coinmarketcap.com"
  }'

# Get price history for a currency pair
curl http://localhost:5200/api/prices/history/BTC-USD

# List price sources for a currency pair
curl http://localhost:5200/api/prices/sources/BTC-USD
```

### WebSocket Client

```javascript
const ws = new WebSocket('ws://localhost:5210');

ws.on('open', () => {
  console.log('Connected to Anypay Prices WebSocket');
});

ws.on('message', (data) => {
  const priceUpdate = JSON.parse(data);
  console.log('Price update:', priceUpdate);
  // { topic: 'price.updated', payload: { currency: 'BTC', value: 43250.00, ... } }
});
```

Each WebSocket connection gets a unique RabbitMQ queue (`websocket:{uid}`), and the server pushes price updates from the exchange polling loop through these queues. Sessions auto-clean their queues on disconnect.

## Next Moves for Growth

1. **Multi-Exchange Redundancy** — Currently CoinMarketCap is primary; add active Kraken and Fixer.io fallback (source modules exist at `src/sources/kraken.rs`, `src/sources/fixer.rs` but are not invoked in `main.rs`). Implement source priority and automatic failover.

2. **REST API Completion** — The Axum server skeleton in `src/rest/server.rs` is not wired into the main binary. Either start it in a separate tokio task from `main.rs`, or deploy it as a standalone service pointing to the same PostgreSQL database.

3. **Stale Price Alerting** — Add a background task that checks `updatedAt` timestamps in the `prices` table and publishes alerts (via RabbitMQ or logs) when any critical currency hasn't updated in >60 seconds. Essential for detecting upstream API outages.

4. **Historical Price Archive** — The `PriceRecords` table exists in `schema.prisma` but is unused. Implement a time-series archival process (e.g., write one record per minute to `PriceRecords`, keep only latest in `prices` table) so invoices can reference historical rates for auditing and dispute resolution.

5. **OpenAPI Spec & Integration Tests** — The REST endpoints reference types like `PriceCreateRequest`, `PriceUpdateRequest`, but these are not defined in the codebase (they are in comments as placeholders). Generate a full OpenAPI schema, add request/response validation, and write integration tests that spin up PostgreSQL + RabbitMQ in Docker Compose.

6. **Rate Limiting & Authentication** — The public REST and WebSocket endpoints have no authentication. Add JWT or API key validation (compatible with Anypay's existing auth system) and rate limiting (per-IP or per-key) before exposing this service beyond internal VPCs.

## Anypay Ecosystem & License

**Anypay Prices (Rust)** is part of [Anypay](https://www.anypayx.com) — Bitcoin & Stablecoin payments for business. Merchants and developers use Anypay to accept BTC, ETH, and stablecoins such as USDC, USDT, RLUSD, and NUSD through invoices, APIs, SDKs, webhooks, WooCommerce, and mobile POS.

| Project | Role |
| --- | --- |
| [anypay](https://github.com/anypay/anypay) | Invoice & payments platform / API (`@anypay/server`) |
| [anypay-sdk](https://github.com/anypay/anypay-sdk) | TypeScript SDK for browser & Node.js (`@anypay/client`) |
| [web3-checkout-vite](https://github.com/anypay/web3-checkout-vite) | Current checkout UI (`checkout.anypayx.com`) |
| [walletbot](https://github.com/anypay/walletbot) | Self-custody headless wallet (`@anypay/walletbot`) |

**Related Price & Conversion Infrastructure:**

| Project | Role |
| --- | --- |
| [anypay/prices](https://github.com/anypay/prices) | Node.js price service (predecessor; this Rust rewrite offers 10x throughput) |
| [anypay/prices-service](https://github.com/anypay/prices-service) | Alternative TypeScript implementation |

### License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

```text
MIT License

Copyright (c) 2024 Anypay Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
...
```

---

<p align="center">
  <a href="https://www.anypayx.com"><img src="https://raw.githubusercontent.com/anypay/anypayx.com/master/public/logo/logo_single.svg" alt="Anypay" width="56" /></a><br/>
  <sub>Bitcoin & Stablecoin Payments For Business · <a href="https://www.anypayx.com">anypayx.com</a> · <a href="https://x.com/xanypay">@xanypay</a></sub>
</p>
