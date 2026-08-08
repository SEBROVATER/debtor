//! Validate the production workspace dependency boundaries from `cargo metadata`.

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    process::Command,
};

use serde_json::Value;

const DOMAIN_ALLOWED: &[&str] = &["chrono", "rust_decimal", "serde", "thiserror"];
const APPLICATION_ALLOWED: &[&str] = &[
    "async-trait",
    "chrono",
    "debtor-domain",
    "futures",
    "rust_decimal",
    "thiserror",
];

const REQUIRED_PACKAGES: &[&str] = &[
    "debtor",
    "debtor-domain",
    "debtor-application",
    "debtor-infra",
    "debtor-web",
];

struct PackageDependencies {
    normal: BTreeSet<String>,
    build: BTreeSet<String>,
}

fn main() -> Result<(), String> {
    let metadata = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing manifest directory")?)
        .output()
        .map_err(|error| format!("unable to run cargo metadata: {error}"))?;
    if !metadata.status.success() {
        return Err(String::from_utf8_lossy(&metadata.stderr).into_owned());
    }

    let document: Value = serde_json::from_slice(&metadata.stdout)
        .map_err(|error| format!("invalid cargo metadata JSON: {error}"))?;
    let violations = evaluate(&document)?;
    if violations.is_empty() {
        println!("architecture fitness checks passed");
        Ok(())
    } else {
        Err(violations.join("\n"))
    }
}

fn evaluate(document: &Value) -> Result<Vec<String>, String> {
    let packages = document
        .get("packages")
        .and_then(Value::as_array)
        .ok_or("cargo metadata did not contain packages")?;
    let mut dependencies = BTreeMap::new();
    for package in packages {
        let name = package
            .get("name")
            .and_then(Value::as_str)
            .ok_or("cargo metadata package did not contain a name")?;
        dependencies.insert(
            name.to_owned(),
            PackageDependencies {
                normal: dependencies_of_kind(package, None)?,
                build: dependencies_of_kind(package, Some("build"))?,
            },
        );
    }

    let mut violations = Vec::new();
    for required in REQUIRED_PACKAGES {
        if !dependencies.contains_key(*required) {
            violations.push(format!(
                "required production package is missing: {required}"
            ));
        }
    }
    if let Some(domain) = dependencies.get("debtor-domain") {
        for dependency in domain.normal.union(&domain.build) {
            if !DOMAIN_ALLOWED.contains(&dependency.as_str()) {
                violations.push(format!(
                    "debtor-domain may only depend on pure domain libraries; found {dependency}"
                ));
            }
        }
    }
    if let Some(application) = dependencies.get("debtor-application") {
        for dependency in application.normal.union(&application.build) {
            if !APPLICATION_ALLOWED.contains(&dependency.as_str()) {
                violations.push(format!(
                    "debtor-application has an outward or unapproved dependency: {dependency}"
                ));
            }
        }
    }
    if let Some(web) = dependencies.get("debtor-web") {
        for dependency in web.normal.union(&web.build) {
            if dependency == "debtor-infra" || dependency == "debtor" {
                violations.push(format!(
                    "debtor-web must not depend outward on {dependency}"
                ));
            }
        }
    }
    if let Some(infra) = dependencies.get("debtor-infra") {
        for dependency in infra.normal.union(&infra.build) {
            if dependency == "debtor-web" || dependency == "debtor" {
                violations.push(format!(
                    "debtor-infra must not depend outward on {dependency}"
                ));
            }
        }
    }
    if let Some(root) = dependencies.get("debtor") {
        for dependency in ["debtor-application", "debtor-infra", "debtor-web"] {
            if !root.normal.contains(dependency) {
                violations.push(format!("root composition crate is missing {dependency}"));
            }
        }
        if root.normal.contains("debtor-domain") || root.build.contains("debtor-domain") {
            violations
                .push("root composition crate must not depend directly on debtor-domain".into());
        }
    }

    Ok(violations)
}

fn dependencies_of_kind(
    package: &Value,
    requested_kind: Option<&str>,
) -> Result<BTreeSet<String>, String> {
    Ok(package
        .get("dependencies")
        .and_then(Value::as_array)
        .ok_or("cargo metadata package did not contain dependencies")?
        .iter()
        .filter(|dependency| match requested_kind {
            Some(kind) => dependency.get("kind").and_then(Value::as_str) == Some(kind),
            None => dependency.get("kind").is_none_or(Value::is_null),
        })
        .filter_map(|dependency| dependency.get("name").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::needless_pass_by_value)]
mod tests {
    use serde_json::{Value, json};

    use super::evaluate;

    fn package(name: &str, dependencies: &[&str]) -> Value {
        json!({
            "name": name,
            "dependencies": dependencies
                .iter()
                .map(|dependency| json!({"name": dependency, "kind": null}))
                .collect::<Vec<_>>()
        })
    }

    fn graph(packages: Vec<Value>) -> Value {
        json!({"packages": packages})
    }

    fn allowed_graph() -> Value {
        graph(vec![
            package(
                "debtor-domain",
                &["chrono", "rust_decimal", "serde", "thiserror"],
            ),
            package(
                "debtor-application",
                &[
                    "async-trait",
                    "chrono",
                    "debtor-domain",
                    "futures",
                    "rust_decimal",
                    "thiserror",
                ],
            ),
            package(
                "debtor-infra",
                &["debtor-domain", "debtor-application", "sqlx"],
            ),
            package(
                "debtor-web",
                &["debtor-domain", "debtor-application", "axum"],
            ),
            package(
                "debtor",
                &["debtor-application", "debtor-infra", "debtor-web", "axum"],
            ),
        ])
    }

    #[test]
    fn accepts_the_supported_inward_dependency_graph() {
        assert!(
            evaluate(&allowed_graph())
                .expect("fixture metadata")
                .is_empty()
        );
    }

    #[test]
    fn rejects_domain_io_and_outward_edges() {
        let document = graph(vec![
            package("debtor-domain", &["sqlx"]),
            package("debtor-application", &["debtor-web"]),
            package("debtor-infra", &["debtor-web"]),
            package("debtor-web", &["debtor-infra"]),
            package(
                "debtor",
                &[
                    "debtor-application",
                    "debtor-infra",
                    "debtor-web",
                    "debtor-domain",
                ],
            ),
        ]);
        let violations = evaluate(&document).expect("fixture metadata");
        assert_eq!(violations.len(), 5);
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("debtor-domain"))
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("debtor-application"))
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("debtor-infra"))
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("debtor-web"))
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("root composition"))
        );
    }

    #[test]
    fn ignores_dev_dependencies_in_architecture_edges() {
        let mut document = allowed_graph();
        document["packages"][0]["dependencies"]
            .as_array_mut()
            .expect("dependency array")
            .push(json!({"name": "tokio", "kind": "dev"}));
        assert!(evaluate(&document).expect("fixture metadata").is_empty());
    }

    #[test]
    fn rejects_missing_packages_and_build_edges() {
        let document = graph(vec![
            package(
                "debtor-domain",
                &["chrono", "rust_decimal", "serde", "thiserror"],
            ),
            package("debtor-application", &["debtor-domain"]),
        ]);
        let violations = evaluate(&document).expect("fixture metadata");
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("missing"))
        );

        let mut document = allowed_graph();
        document["packages"][1]["dependencies"]
            .as_array_mut()
            .expect("dependency array")
            .push(json!({"name": "sqlx", "kind": "build"}));
        let violations = evaluate(&document).expect("fixture metadata");
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("debtor-application"))
        );
    }

    #[test]
    fn checks_the_package_name_even_when_a_dependency_is_renamed() {
        let mut document = allowed_graph();
        document["packages"][1]["dependencies"]
            .as_array_mut()
            .expect("dependency array")
            .push(json!({
                "name": "debtor-web",
                "rename": "application_web_facade",
                "kind": null
            }));
        let violations = evaluate(&document).expect("fixture metadata");
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("debtor-application"))
        );
    }
}
