# Contracts: Dotenvy Environment Variable Support

**Status**: N/A — No external interface contracts for this feature.

This feature is an internal configuration change. It does not expose new APIs, modify existing endpoints, or change any external interfaces. The integration is limited to:

- **Cargo.toml**: New dependency declarations (`dotenvy`, `dotenvy_macro`)
- **main.rs**: Startup call to `dotenvy::dotenv()`
- **.gitignore**: Negation rule for `.env.example`
- **.env.example**: New template file (static content)

No contract artifacts are needed.
