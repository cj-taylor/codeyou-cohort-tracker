# Frontend Testing

## Setup

```bash
cd static
npm install
```

## Running Tests

```bash
npm test              # Run all tests
npm run test:watch    # Watch mode
npm run test:coverage # With coverage
```

## Current Modules

- **utils.js** - Formatting and masking functions
- **state.js** - Global application state  
- **api.js** - API wrapper functions

Each module has corresponding tests in `js/__tests__/`.

## Next Steps

The main `app.js` (1692 lines) will be gradually refactored into smaller modules with tests for each.
