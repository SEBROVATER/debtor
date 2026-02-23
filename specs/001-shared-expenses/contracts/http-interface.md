# HTTP Interface Contract

Base URL: `http://<host>:<port>`  
Rendering mode: server-rendered HTML + HTMX fragments  
Auth: server-side session cookie (`HttpOnly`, `Secure` in HTTPS deployments)

## Global Rules

- All routes except `/login` and `/health` require valid session.
- Unauthenticated requests get `303 See Other` redirect to `/login`.
- Form submissions use `application/x-www-form-urlencoded`.
- HTMX requests return partial HTML fragments; non-HTMX requests return full pages.

## Authentication Routes

### `GET /login`

- Response `200 text/html`
- Returns login page with username/password form.

### `POST /login`

Request fields:
- `username` (required)
- `password` (required)

Responses:
- `303` redirect to `/dashboard` on success and sets session cookie.
- `401` login HTML fragment/page with invalid-credentials message.
- `423` login HTML fragment/page with lockout-until message.

### `POST /logout`

- Requires authenticated session.
- Invalidates session immediately.
- Response `303` redirect to `/login`.

## Dashboard and Group Routes

### `GET /dashboard`

- Response `200 text/html`
- Contains group list and create-group form.

### `POST /groups`

Request fields:
- `name` (required, 1..80)
- `target_currency` (required, ISO-4217)

Responses:
- `201` group row fragment (HTMX) or `303` to `/groups/{id}`.
- `422` validation error fragment/page.

### `GET /groups/{group_id}`

- Response `200 text/html`
- Shows group detail, members, expenses, and debt summary containers.

### `PATCH /groups/{group_id}`

Request fields:
- `name` (optional)
- `target_currency` (optional)

Responses:
- `200` updated group fragment/page.
- `422` validation error fragment/page.

### `DELETE /groups/{group_id}`

Responses:
- `204` (HTMX trigger to remove row) or `303` to `/dashboard`.

## Member Routes

### `POST /groups/{group_id}/members`

Request fields:
- `display_name` (required, 1..80)

Responses:
- `201` member list fragment.
- `422` validation error.

### `PATCH /groups/{group_id}/members/{member_id}`

Request fields:
- `display_name` (required)

Responses:
- `200` updated member fragment.
- `404` if member not found.

### `DELETE /groups/{group_id}/members/{member_id}`

Behavior:
- Member marked inactive, not physically deleted.

Responses:
- `200` refreshed member list fragment.

## Expense Routes

### `POST /groups/{group_id}/expenses`

Request fields:
- `payer_member_id` (required)
- `amount` (required decimal > 0)
- `currency` (required ISO-4217)
- `expense_date` (required date)
- `note` (optional)
- `shares[]` (required participant share payload)

`shares[]` item fields:
- `member_id`
- `mode` (`equal|percent|amount`)
- `value` (required for `percent|amount`)

Responses:
- `201` expense list fragment plus refreshed debt summary fragment.
- `422` share validation or field validation errors.

### `PATCH /groups/{group_id}/expenses/{expense_id}`

- Same payload contract as create.
- Response `200` updated fragments or `422`.

### `DELETE /groups/{group_id}/expenses/{expense_id}`

- Response `200` refreshed expense list + debt summary fragments.

## Debt Summary Route

### `GET /groups/{group_id}/debts`

Query fields:
- `format` optional (`fragment|page`, default inferred by HTMX headers)

Response `200 text/html`:
- Minimal directed debt transactions in group target currency.
- `no outstanding debts` indicator when net balances are all zero.
- Optional warning banner when stale exchange rates were used.

Error responses:
- `503` conversion blocked when no rate cache exists and provider call fails.

## Health Route

### `GET /health`

- Response `200 application/json`
- Body: `{ "status": "ok" }`
