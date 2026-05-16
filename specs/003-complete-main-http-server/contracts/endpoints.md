# HTTP Endpoint Contracts: Complete Main Function with HTTP Server

**Branch**: `003-complete-main-http-server` | **Date**: 2026-04-12

## Overview

All 16 routes from the existing `route_specs()` table, plus static file serving. Each endpoint is documented with its HTTP contract: method, path, authentication requirement, CSRF protection, request format, and response format.

---

## Health

### `GET /health`

- **Auth**: No
- **CSRF**: No
- **Request**: No body
- **Response**: `200 OK` with plain text body `"ok"`
- **Purpose**: Liveness check — confirms the server is running and accepting requests

---

## Authentication

### `GET /login`

- **Auth**: No
- **CSRF**: No
- **Request**: No body
- **Response**: `200 OK` with HTML page (`auth/login.html`) containing:
  - CSRF token in hidden form field
  - Error banners (hidden by default; shown via `data-state` attribute matching on re-render)
- **Template variables**: `csrf_token`, `lockout_until`, error state indicators

### `POST /login`

- **Auth**: No
- **CSRF**: Yes (form field `csrf_token`)
- **Request**: `application/x-www-form-urlencoded` — fields: `username`, `password`, `csrf_token`
- **Response**:
  - Success: `303 See Other` redirect to `/dashboard` + `Set-Cookie` session header
  - Invalid credentials: `200 OK` re-render of login page with `data-state="invalid"` error visible
  - Lockout active: `200 OK` re-render of login page with `data-state="lockout"` error visible and `lockout_until` populated
  - CSRF failure: `200 OK` re-render of login page with `data-state="csrf"` error visible

### `POST /logout`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` — field: `csrf_token`
- **Response**: `303 See Other` redirect to `/login` + `Set-Cookie` clearing session cookie

---

## Dashboard

### `GET /dashboard`

- **Auth**: Yes
- **CSRF**: No
- **Request**: No body
- **Response**: `200 OK` with HTML page (layout + dashboard placeholder content)

---

## Groups

### `POST /groups`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` — fields: `name`, `target_currency`
- **Response**: `303 See Other` redirect to `/groups/{new_group_id}`

### `GET /groups/{group_id}`

- **Auth**: Yes
- **CSRF**: No
- **Request**: Path parameter `group_id`
- **Response**: `200 OK` with HTML page (`groups/detail.html`) containing group settings, member list, expense list, debt summary
- **Template variables**: `group_id`, `group_name`, `target_currency`, `members`

### `PATCH /groups/{group_id}`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` (via POST with `_method=PATCH`) — fields: `name` (optional), `target_currency` (optional)
- **Response**: `303 See Other` redirect to `/groups/{group_id}`

### `DELETE /groups/{group_id}`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` (via POST with `_method=DELETE`)
- **Response**: `303 See Other` redirect to `/dashboard`

---

## Members

### `POST /groups/{group_id}/members`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` — field: `display_name`
- **Response**: `303 See Other` redirect to `/groups/{group_id}`

### `PATCH /groups/{group_id}/members/{member_id}`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` (via POST with `_method=PATCH`) — field: `display_name`
- **Response**: `303 See Other` redirect to `/groups/{group_id}`

### `DELETE /groups/{group_id}/members/{member_id}`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` (via POST with `_method=DELETE`)
- **Response**: `303 See Other` redirect to `/groups/{group_id}`

---

## Expenses

### `POST /groups/{group_id}/expenses`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` — fields: `payer_member_id`, `amount`, `currency`, `expense_date`, `note` (optional), `shares[{member_id}][mode]`, `shares[{member_id}][value]`
- **Response**: `303 See Other` redirect to `/groups/{group_id}`

### `PATCH /groups/{group_id}/expenses/{expense_id}`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` (via POST with `_method=PATCH`) — same fields as create
- **Response**: `303 See Other` redirect to `/groups/{group_id}`

### `DELETE /groups/{group_id}/expenses/{expense_id}`

- **Auth**: Yes
- **CSRF**: Yes
- **Request**: `application/x-www-form-urlencoded` (via POST with `_method=DELETE`)
- **Response**: `303 See Other` redirect to `/groups/{group_id}`

---

## Debts

### `GET /groups/{group_id}/debts`

- **Auth**: Yes
- **CSRF**: No
- **Request**: Path parameter `group_id`
- **Response**: `200 OK` with HTML fragment (`debts/summary.html`)
- **Template variables**: `group_id`, `no_outstanding`, `transfers` (vec of from_member_name, to_member_name, amount), `rates_stale_warning`, `conversion_blocked_no_cache`

---

## Static Assets

### `GET /static/**`

- **Auth**: No
- **CSRF**: No
- **Request**: File path within `static/` directory
- **Response**: File contents with appropriate `Content-Type` header (e.g., `text/css` for `.css` files)
- **404**: Returns `404 Not Found` for non-existent files

---

## Common Behaviors

### Method Override

HTML forms only support GET and POST. For PATCH and DELETE operations, the templates include `<input type="hidden" name="_method" value="PATCH|DELETE">` inside a POST form. The server extracts this field and routes the request as the overridden method.

### Authentication Redirect

Requests to routes with `requires_auth: true` that lack a valid session cookie are redirected to `GET /login` via `303 See Other`.

### CSRF Validation

Routes with `csrf_protected: true` require a `csrf_token` field in the form body that matches the token stored in the session. Mismatches result in a `403 Forbidden` response.

### Error Responses

| HTTP Status | Condition |
|-------------|-----------|
| 200 | Successful page render |
| 303 | Successful state-changing operation (redirect) |
| 401 | Missing or expired session on protected route |
| 403 | CSRF validation failure or forbidden access |
| 404 | Unknown route or missing resource |
| 422 | Validation error (invalid form data) |
| 500 | Internal server error (database failure, etc.) |
| 503 | Exchange rate conversion unavailable |
