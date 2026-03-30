# Feature Specification: Add Dotenvy Environment Variable Support

**Feature Branch**: `[002-add-dotenvy-env]`
**Created**: 2026-03-29
**Status**: Draft
**Input**: User description: "I need to be able running this project so it would retrieve every required data from .env file using 'dotenvy' crate. They must be read or include at compile time."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Load configuration from .env file at startup (Priority: P1)

A developer or operator places a `.env` file in the project root directory. When the application starts, all required environment variables are automatically loaded from the `.env` file without needing to set them manually in the shell or system environment.

**Why this priority**: This is the core value proposition — eliminating manual environment variable setup and enabling consistent, reproducible configuration across development and deployment environments.

**Independent Test**: Can be fully tested by creating a `.env` file with all configuration values, starting the application, and verifying it boots successfully without any pre-set shell environment variables.

**Acceptance Scenarios**:

1. **Given** a `.env` file exists in the project root with all required configuration values, **When** the application starts, **Then** it loads all values from the `.env` file and runs successfully
2. **Given** a `.env` file exists with some but not all configuration values, **When** the application starts, **Then** it loads the provided values from the file and uses sensible defaults for the missing ones
3. **Given** no `.env` file exists in the project root, **When** the application starts, **Then** it falls back to reading from the system environment or using default values, and continues to run without error
4. **Given** both a `.env` file and the system environment define the same variable, **When** the application starts, **Then** it uses the system environment value for that variable
5. **Given** a `.env` file exists with malformed entries, **When** the application starts, **Then** it exits with a clear error message indicating the file cannot be parsed
6. **Given** a `.env` file exists but is not readable due to permissions, **When** the application starts, **Then** it exits with a clear error message indicating the file cannot be read

---

### User Story 2 - Support compile-time environment variable inclusion (Priority: P2)

Configuration values can be embedded at compile time so that the built binary carries its configuration without requiring a `.env` file at runtime. This is useful for deployment scenarios where embedding configuration in the binary is preferred.

**Why this priority**: Enables deployment workflows where configuration is baked into the artifact, reducing runtime dependencies and simplifying deployment.

**Independent Test**: Can be tested by compiling the application with environment variables set during the build, then running the resulting binary without a `.env` file and verifying the embedded configuration is used.

**Acceptance Scenarios**:

1. **Given** environment variables are set during the compilation process, **When** the compiled binary runs without a `.env` file present, **Then** the application uses the compile-time embedded configuration values

---

### User Story 3 - Document required .env file format (Priority: P3)

A developer new to the project can quickly understand what configuration values are needed by looking at documentation and example files.

**Why this priority**: Reduces onboarding friction and prevents misconfiguration by making required and optional variables immediately clear.

**Independent Test**: Can be tested by having a new developer follow the documentation to set up a `.env` file and successfully start the application.

**Acceptance Scenarios**:

1. **Given** a developer reads the project documentation, **When** they look for configuration instructions, **Then** they find a clear list of all environment variables with descriptions and example values
2. **Given** an `.env.example` template file exists, **When** a developer copies it to `.env` and fills in values, **Then** the application starts successfully
3. **Given** `.env.example` is present in the repository, **When** a developer clones the project, **Then** the `.env.example` file is included in the checkout

---

### Edge Cases

- Whitespace or special characters in `.env` values are preserved as-is (standard dotenv behavior)
- If the `.env` file exists but contains malformed entries (e.g., missing `=` sign), the application MUST fail fast with a clear error message indicating the file cannot be parsed
- When both `.env` file and system environment define the same variable, system environment takes precedence (standard dotenv convention)
- If the `.env` file exists but is not readable due to file permissions, the application MUST fail fast with a clear error message indicating the file cannot be read

## Clarifications

### Session 2026-03-29

- Q: When both .env and system env define the same variable, which takes precedence? → A: System env takes precedence over .env (standard dotenv convention)
- Q: Should runtime .env loading coexist with compile-time embedding, or should one replace the other? → A: Both: runtime `.env` loading via `dotenvy::dotenv()` at startup + optional compile-time embedding via `dotenv!()` macro — existing `std::env::var` calls remain unchanged
- Q: How should `.env.example` be tracked in git given `.gitignore` has `.env*`? → A: Keep `.env*` in `.gitignore` and add a negation rule `!.env.example` to explicitly track the template file
- Q: How should the app handle malformed or unreadable `.env` files? → A: Fail fast with a clear error message if the `.env` file exists but is malformed or unreadable

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST load environment variables from a `.env` file at application startup, populating the process environment so that existing `std::env::var` calls retrieve `.env` values
- **FR-002**: System MUST support compile-time inclusion of environment variables via a macro so that configuration can be embedded in the binary as an alternative to runtime `.env` loading
- **FR-003**: System MUST continue to support all existing environment variables (APP_DATABASE_URL, APP_SESSION_COOKIE_NAME, APP_ADMIN_USERNAME, APP_ADMIN_PASSWORD_HASH, APP_SESSION_COOKIE_SECURE, APP_EXCHANGE_BASE_URL)
- **FR-004**: System MUST fall back gracefully to sensible defaults when environment variables are missing from both the `.env` file and the system environment
- **FR-005**: System MUST provide an `.env.example` file listing all supported configuration variables with placeholder values and comments
- **FR-006**: System MUST ensure `.env` files containing secrets are excluded from version control while `.env.example` remains tracked via a `.gitignore` negation rule (`!.env.example`)
- **FR-007**: System MUST fail fast with a clear error message if the `.env` file exists but cannot be read or parsed

### Key Entities

- **Environment Variable**: A key-value configuration pair used by the application. Examples: APP_DATABASE_URL (database connection string), APP_ADMIN_PASSWORD_HASH (hashed admin password)
- **.env File**: A plain-text file in the project root containing environment variable definitions in `KEY=VALUE` format, one per line

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can start the application with only a `.env` file and no prior shell environment configuration in under 2 minutes
- **SC-002**: All 6 existing configuration variables can be supplied via the `.env` file
- **SC-003**: The application starts successfully whether configuration comes from `.env` file, system environment, or a mix of both
- **SC-004**: A new developer can set up a working `.env` file using the `.env.example` template in under 5 minutes
- **SC-005**: The built binary works without a `.env` file when environment variables were set during compilation

## Assumptions

- The application continues to use the existing `AppConfig::from_env()` pattern as the configuration loading mechanism
- The `.env` file is located in the project root directory (same directory as `Cargo.toml`)
- Environment variables set in the shell or system take precedence over those in the `.env` file (standard dotenv convention)
- The `dotenvy` crate is used for `.env` file parsing and loading
- The `dotenvy_macro` crate's `dotenv!()` macro is used for compile-time variable inclusion
- The `.env` file follows standard `KEY=VALUE` format with `#` for comments
- The `.env.example` template file serves as documentation for required and optional variables
