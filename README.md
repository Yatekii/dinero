# Dinero

Personal finance management application for tracking accounts across multiple
banks and investment platforms.

## Overview

Dinero processes financial data from various bank formats and provides
portfolio tracking with foreign exchange rate handling. It consists of a Rust
backend for data processing and a React frontend for visualization.

## Supported Banks and Platforms

- Revolut: Multi-currency banking
- Neon: Swiss digital banking
- UBS: Swiss banking (private and business accounts)
- IBKR: Interactive Brokers investment platform
- Wise: Multi-currency money transfers

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Node.js and npm
- Environment variables configured (see `.env` file)

### Running the Application

Start the backend server:

```bash
cargo run -- serve
```

Start the frontend development server:

```bash
cd frontend
npm install
npm run dev
```

Access the application:

- Frontend: <http://localhost:5173>
- Backend API: <http://localhost:3000/api>

## Data Import Instructions

### Wise Transaction Data

To import your Wise transaction history:

- Go to <https://wise.com/balances/statements/balance-statement>
- Log into your Wise account
- Select the account/currency you want to export
- Choose your desired date range
- Click "Download" and select CSV format
- Save the file with a descriptive name
  (e.g., `2024-transaction-history-chf.csv`)
- Upload the CSV file through the Dinero web interface
- The system will automatically parse and process the transactions
- Transactions will appear in your Wise account ledger

### Other Banks

Similar CSV export processes are available for other supported banks. Access
your online banking platform and look for transaction export or statement
download options.

## Development

### Backend Development

```bash
# Build and run
cargo run -- serve

# Run tests
cargo test

# Review test snapshots
cargo insta review
```

### Frontend Development

```bash
cd frontend

# Development server
npm run dev

# Build for production
npm run build

# Lint code
npm run lint
```

## Architecture

- Backend: Rust with Axum web framework
- Frontend: React with TypeScript and Vite
- Data Processing: Format-specific CSV parsers
- Authentication: OAuth2/OIDC integration
- Styling: Tailwind CSS with Radix UI components
