# Exchange Rate Provider Contract

## Provider

- Name: Frankfurter
- Endpoint: `https://api.frankfurter.app/latest`
- Method: `GET`
- Authentication: none
- Transport: HTTPS

## Request Contract

Query parameters:
- `from` (required): base currency code, ISO-4217 uppercase
- `to` (required): comma-separated target currency codes

Example:

```http
GET /latest?from=USD&to=EUR,PLN HTTP/1.1
Host: api.frankfurter.app
Accept: application/json
```

## Response Contract

Success (`200 application/json`) shape:

```json
{
  "amount": 1.0,
  "base": "USD",
  "date": "2026-02-23",
  "rates": {
    "EUR": 0.92,
    "PLN": 3.95
  }
}
```

Required mapping rules:
- `base` -> `from_currency`
- each key in `rates` -> `to_currency`
- each numeric value in `rates` -> `rate`
- `date` -> `rate_date`
- local fetch timestamp -> `fetched_at`

## Caching Contract

- Cache key: (`from_currency`, `to_currency`, `rate_date`)
- Freshness target: no more than one provider fetch per pair per day.
- Refresh trigger: lazy, on debt summary read if pair missing/stale.
- Fallback: if provider fetch fails and prior cache exists, use latest cached
  rate and surface stale-data warning.
- Hard failure: if provider fetch fails and no cached rate exists, debt
  conversion response is blocked with user-facing error.

## Timeout and Retry Policy

- Request timeout: 3 seconds.
- Retries: 0 immediate retries (fail fast, use cache fallback).
- Circuit behavior: none initially; keep simple per constitution principle V.
