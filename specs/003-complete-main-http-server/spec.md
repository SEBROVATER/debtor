# Feature Specification: Complete Main Function with HTTP Server

**Feature Branch**: `003-complete-main-http-server`  
**Created**: 2026-04-12  
**Status**: Draft  
**Input**: User description: "This project is running but exiting immediately without errors. You need to make 'main' function completed."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Application Stays Running and Accepts Requests (Priority: P1)

As the application owner, I want the application to start up, bind to a network port, and remain running so that it can serve web requests instead of exiting immediately after initialization.

**Why this priority**: This is the fundamental capability — without a running server, no other feature (authentication, expense tracking, group management) can be used. Every other user story depends on the application actually listening for and responding to requests.

**Independent Test**: Can be fully tested by starting the application and verifying it binds to the configured port, responds to a health-check request, and does not exit on its own.

**Acceptance Scenarios**:

1. **Given** the application is configured with valid environment variables, **When** the application starts, **Then** it binds to the configured host and port and begins accepting connections.
2. **Given** the application is running, **When** a client sends a request to the health endpoint, **Then** the application responds with a success status indicating it is operational.
3. **Given** the application is running, **When** no requests are being made, **Then** the application remains running and does not exit.

---

### User Story 2 - Routes Are Wired to Handlers (Priority: P2)

As the application owner, I want each declared route (login, groups, expenses, debts, dashboard) to be wired to its corresponding handler so that HTTP requests are routed to the correct domain logic.

**Why this priority**: The application already has 16 route definitions and handler functions — they just need to be connected to a live HTTP transport. Without this wiring, the server would run but could not serve any useful pages.

**Independent Test**: Can be tested by sending requests to each declared route path and verifying the server dispatches to the correct handler and returns the expected response type (HTML page, redirect, or error status).

**Acceptance Scenarios**:

1. **Given** the application is running, **When** a client sends a request matching a declared route, **Then** the application invokes the corresponding handler and returns a response.
2. **Given** the application is running, **When** a client sends a request to an undeclared path, **Then** the application returns a "not found" response.

---

### User Story 3 - Graceful Shutdown (Priority: P3)

As the application owner, I want the application to shut down cleanly when it receives a termination signal so that in-flight requests are completed and resources (database connections, open files) are released properly.

**Why this priority**: Graceful shutdown prevents data corruption and dropped requests during deployments or restarts. It is important for operational reliability but is secondary to getting the server running in the first place.

**Independent Test**: Can be tested by starting the application, sending a termination signal, and verifying that the application exits with a success status and does not leave dangling connections.

**Acceptance Scenarios**:

1. **Given** the application is running, **When** it receives a termination signal, **Then** it stops accepting new connections, completes any in-flight requests, and exits cleanly.
2. **Given** the application is running with active database connections, **When** it shuts down, **Then** all database connections are properly closed.

---

### Edge Cases

- What happens when the configured port is already in use? The application should report a clear error and exit with a non-zero status.
- What happens when required environment variables (host, port) are missing? The application should use sensible defaults (e.g., `127.0.0.1:3000`) or report a clear configuration error.
- What happens when the database is unreachable at startup? The existing initialization logic already handles this — the server startup should not mask or swallow those errors.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST bind to a configurable network address (host and port) and listen for incoming connections.
- **FR-002**: The application MUST remain running after initialization, continuously accepting and processing requests until a shutdown signal is received.
- **FR-003**: The application MUST route incoming requests to the appropriate handler based on the existing route definitions (the 16 routes declared in the route table).
- **FR-004**: The application MUST serve static assets (CSS files) so that rendered HTML pages are properly styled.
- **FR-005**: The application MUST apply authentication checks to routes marked as requiring authentication, redirecting unauthenticated users to the login page.
- **FR-006**: The application MUST apply CSRF validation to routes marked as requiring CSRF protection, rejecting requests that fail validation.
- **FR-007**: The application MUST render HTML templates for page responses, using the existing template files.
- **FR-008**: The application MUST provide a health-check endpoint that returns a success response without requiring authentication.
- **FR-009**: The application MUST use sensible default values for host (`127.0.0.1`) and port (`3000`) when not explicitly configured.
- **FR-010**: The application MUST log the address it is listening on at startup.
- **FR-011**: The application MUST shut down gracefully when it receives a termination signal, completing in-flight requests before exiting.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The application starts, binds to its configured address, and responds to a health-check request within 5 seconds of launch.
- **SC-002**: All 16 declared routes return appropriate responses (not "not found") when the correct request method and path are used.
- **SC-003**: The application remains running for at least 10 minutes under idle conditions without crashing or exiting.
- **SC-004**: The application shuts down cleanly within 30 seconds of receiving a termination signal.
- **SC-005**: Requests to protected routes without a valid session are redirected to the login page.
- **SC-006**: Static CSS assets are served correctly, and HTML pages reference them without broken links.

## Assumptions

- The single application owner is the only user — no multi-tenancy or high-concurrency requirements beyond basic responsiveness.
- The existing route table, handler functions, authentication middleware, CSRF validation, and HTML templates are functionally correct and do not need modification — this feature is solely about wiring them to an HTTP transport.
- The existing `AppConfig` structure will be extended to include host and port configuration, following the same pattern of environment-variable-based configuration.
- The `acton-dx` dependency (aliased as `acton-htmx` in the project) does not provide HTTP server functionality and will need to be supplemented or replaced with an actual HTTP framework.
- Session management (cookie-based, server-side) is already implemented and will be integrated into the HTTP middleware layer as-is.
