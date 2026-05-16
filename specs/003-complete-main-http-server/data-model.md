# Data Model: Complete Main Function with HTTP Server

**Branch**: `003-complete-main-http-server` | **Date**: 2026-04-12

## Overview

This feature introduces **no new database entities**. It is a transport/adapter layer that wires existing domain logic to an HTTP server. All entities listed below are existing and unchanged.

## Existing Entities (unchanged)

| Entity | Purpose | Key Fields |
|--------|---------|-----------|
| `admin_users` | Single admin account | `id`, `username`, `password_hash` |
| `sessions` | Server-side session tokens | `id`, `user_id`, `token_hash`, `expires_at`, `revoked_at` |
| `auth_state` | Login attempt tracking / lockout | `id`, `failed_attempt_count`, `lockout_until` |
| `groups` | Expense groups | `id`, `name`, `target_currency` |
| `members` | Group members | `id`, `group_id`, `display_name`, `is_active` |
| `expenses` | Individual expenses | `id`, `group_id`, `payer_member_id`, `amount`, `currency` |
| `expense_shares` | How expenses split among members | `id`, `expense_id`, `member_id`, `share_mode`, `share_value` |
| `exchange_rates` | Cached currency exchange rates | `id`, `from_currency`, `to_currency`, `rate` |

## New Runtime Structures (not persisted)

These are in-memory structures introduced by the adapter layer:

### AppConfig Extensions

Two new fields added to the existing `AppConfig` struct:

| Field | Type | Env Var | Default | Purpose |
|-------|------|---------|---------|---------|
| `host` | `String` | `APP_HOST` | `"127.0.0.1"` | Network interface to bind |
| `port` | `u16` | `APP_PORT` | `3000` | TCP port to listen on |

### Template Context Structs

Serializable structs passed to Handlebars templates during rendering. These map existing domain models to template variables:

| Context Struct | Template | Key Fields |
|---------------|----------|-----------|
| `LoginPageContext` | `auth/login.html` | `csrf_token`, `error_state` (none/invalid/lockout/csrf), `lockout_until` |
| `GroupDetailContext` | `groups/detail.html` | `group_id`, `group_name`, `target_currency`, `members` |
| `MemberListContext` | `groups/partials/member_list.html` | `group_id`, `members` (vec of id, display_name, is_active) |
| `ExpenseListContext` | `expenses/list.html` | `group_id`, `expenses` (vec), `members` (for form) |
| `ExpenseFormContext` | `expenses/partials/expense_form.html` | `group_id`, `members` (vec of id, display_name) |
| `DebtSummaryContext` | `debts/summary.html` | `group_id`, `no_outstanding`, `transfers`, `rates_stale_warning`, `conversion_blocked_no_cache` |
| `LayoutContext` | `layout.html` | `content` (rendered inner HTML) |

No database migrations required.
