# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dinero is a personal finance management application consisting of a Rust backend and React frontend. It processes financial data from various bank formats (UBS, Neon, Revolut, IBKR, Wise) and provides portfolio tracking with foreign exchange rate handling.

## Architecture

### Backend (Rust)
- **Web Framework**: Axum with async/await using Tokio
- **Authentication**: OAuth2/OIDC integration
- **Data Processing**: CSV parsing for bank statements with format-specific parsers
- **API Structure**: RESTful endpoints for ledgers, portfolio data, and file management
- **Key Modules**:
  - `banks/`: Format-specific parsers for different financial institutions
  - `handler/`: HTTP request handlers organized by feature (auth, ledger, portfolio)
  - `fx.rs`: Foreign exchange rate handling and currency conversion
  - `processing.rs`: Transaction processing and extended ledger record creation
  - `realms/`: Domain logic for portfolio state management

### Frontend (React + TypeScript)
- **Framework**: React with TypeScript using Vite build system
- **Routing**: React Router with typesafe routing
- **UI Components**: Radix UI, Headless UI, Tremor React for charts
- **Styling**: Tailwind CSS with custom components
- **Key Areas**:
  - `dashboard/`: Net worth tracking and spending analysis
  - `ledger/`: Account management, file uploads, transaction viewing
  - `bindings/`: TypeScript types auto-generated from Rust using ts-rs

## Common Development Commands

### Backend Development
```bash
# Build and run the server
cargo run -- serve

# Run tests
cargo test

# Run tests with snapshots (using insta)
cargo test
cargo insta review  # Review snapshot changes
```

### Frontend Development
```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Development server
npm run dev

# Build for production
npm run build

# Lint code
npm run lint

# Type checking (via build process)
npm run build
```

## Key Environment Variables

- `BASE_PATH`: API base path for the server
- `SERVER_ADDRESS`: Server binding address
- OAuth2 configuration for authentication

## Data Flow

1. Bank CSV files are uploaded via frontend
2. Backend parses files using format-specific parsers (`banks/` module)
3. Raw transactions are processed into `ExtendedLedgerRecord` format
4. FX rates are fetched and cached for multi-currency support
5. Portfolio state is calculated and cached in `realms/portfolio`
6. Dashboard aggregates data for visualization

## File Structure Notes

- `portfolio/`: Runtime data storage for ledgers and FX rates
- `src/banks/snapshots/`: Insta test snapshots for parser validation
- Frontend bindings are auto-generated from Rust types using ts-rs
- CSS classes follow Tailwind utility-first approach

## Testing

- Backend uses `insta` for snapshot testing of bank parsers
- Tests focus on CSV parsing accuracy across different bank formats
- No specific frontend test framework currently configured

## Future Improvements

### Portfolio Prediction Enhancements
*Keywords: portfolio prediction, prediction improvements, forecasting, financial modeling*

**Current Issue**: The portfolio growth prediction uses basic linear regression on 300 data points, which is overly simplistic for financial forecasting.

**Improvement Ideas**:

1. **Multiple Model Approach**
   - Linear trend (current)
   - Moving averages for trend-following
   - Mean reversion for market corrections
   - ARIMA for time series patterns

2. **Asset-Specific Modeling**
   - Stocks: Volatility models (GARCH), market correlation
   - Bonds: Interest rate sensitivity
   - Cash: Inflation adjustment
   - Crypto: High volatility with sentiment factors

3. **Enhanced Statistical Methods**
   - Monte Carlo simulation with thousands of scenarios
   - Confidence intervals (5th-95th percentiles)
   - Historical volatility patterns
   - Asset correlation matrices

4. **Feature Engineering**
   - Seasonality detection for spending/income patterns
   - Trend decomposition (long-term vs cyclical)
   - Market regime detection (bull/bear periods)
   - External economic indicators

5. **Risk-Adjusted Metrics**
   - Value at Risk (VaR)
   - Expected shortfall
   - Maximum drawdown estimation
   - Sharpe ratio calculations

6. **Machine Learning Options**
   - LSTM networks for complex patterns
   - Random Forest for non-linear relationships
   - Ensemble methods for robustness

**Location**: `src/handler/portfolio/get.rs:125-144` (current linear regression implementation)