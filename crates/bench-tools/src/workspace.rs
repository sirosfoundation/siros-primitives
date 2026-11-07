//! The workspace files that a measured library and its bench executable are built from, read from
//! `cargo metadata`.

use std::collections::VecDeque;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::run::{Paths, output};
use crate::tool_error::ToolError;

/// The files of the workspace root that every build reads.
const ROOT_FILES: [&str; 4] = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    ".cargo/config.toml",
];

/// The git pathspecs of a measured package, relative to the workspace root.
pub(crate) struct Sources {
    /// The files that the library's code is compiled from: `src`, `Cargo.toml` and `build.rs`
    /// of the package and of every workspace package it depends on for its build.
    library: Vec<String>,
    /// The files that the bench executable is built from: the root files, the library's files,
    /// the package's `benches`, and the files of the workspace packages its benches depend on.
    executable: Vec<String>,
}

impl Sources {
    pub(crate) fn library(&self) -> Vec<&str> {
        self.library.iter().map(String::as_str).collect()
    }

    pub(crate) fn executable(&self) -> Vec<&str> {
        self.executable.iter().map(String::as_str).collect()
    }
}

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<String>,
    workspace_root: String,
    resolve: Resolve,
}

#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    manifest_path: String,
}

#[derive(Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Deserialize)]
struct Node {
    id: String,
    deps: Vec<Dependency>,
}

#[derive(Deserialize)]
struct Dependency {
    pkg: String,
    dep_kinds: Vec<Kind>,
}

#[derive(Deserialize)]
struct Kind {
    /// `None` for a normal dependency, `"dev"` or `"build"` otherwise.
    kind: Option<String>,
}

impl Dependency {
    /// Whether the dependency is compiled into its dependent's library.
    fn builds_library(&self) -> bool {
        self.dep_kinds
            .iter()
            .any(|kind| kind.kind.as_deref() != Some("dev"))
    }
}

impl Sources {
    /// The sources of `package`, from `cargo metadata` at the workspace root with every feature
    /// resolved.
    pub(crate) fn of(paths: &Paths, package: &str) -> Result<Self, ToolError> {
        let text = output(
            Command::new("cargo")
                .args([
                    "metadata",
                    "--format-version",
                    "1",
                    "--locked",
                    "--all-features",
                ])
                .current_dir(&paths.root),
        )?;
        let metadata: Metadata = serde_json::from_str(&text).map_err(ToolError::Metadata)?;
        metadata.sources(package)
    }
}

impl Metadata {
    fn sources(&self, package: &str) -> Result<Sources, ToolError> {
        let root = self
            .packages
            .iter()
            .find(|candidate| candidate.name == package && self.is_member(&candidate.id))
            .ok_or_else(|| {
                ToolError::missing(&format!("workspace package {package}"), "cargo metadata")
            })?;
        let node = self.node(&root.id)?;
        let library = self.closure(
            node.deps
                .iter()
                .filter(|dependency| dependency.builds_library())
                .map(|dependency| dependency.pkg.as_str()),
            &root.id,
        )?;
        let benched = self.closure(
            node.deps.iter().map(|dependency| dependency.pkg.as_str()),
            &root.id,
        )?;
        let mut library_files = Vec::new();
        let mut executable_files: Vec<String> =
            ROOT_FILES.iter().map(|file| (*file).to_owned()).collect();
        for id in &library {
            library_files.extend(self.code(id)?);
        }
        for id in &benched {
            executable_files.extend(self.code(id)?);
        }
        executable_files.push(format!("{}/benches", self.directory(&root.id)?));
        Ok(Sources {
            library: library_files,
            executable: executable_files,
        })
    }

    /// `start` and the workspace packages that `first` and their library dependencies reach, in
    /// breadth-first order.
    fn closure<'a>(
        &'a self,
        first: impl Iterator<Item = &'a str>,
        start: &'a str,
    ) -> Result<Vec<&'a str>, ToolError> {
        let mut order = vec![start];
        let mut queue: VecDeque<&str> = first.filter(|id| self.is_member(id)).collect();
        while let Some(id) = queue.pop_front() {
            if order.contains(&id) {
                continue;
            }
            order.push(id);
            queue.extend(
                self.node(id)?
                    .deps
                    .iter()
                    .filter(|dependency| dependency.builds_library())
                    .map(|dependency| dependency.pkg.as_str())
                    .filter(|dependency| self.is_member(dependency)),
            );
        }
        Ok(order)
    }

    fn is_member(&self, id: &str) -> bool {
        self.workspace_members.iter().any(|member| member == id)
    }

    fn node(&self, id: &str) -> Result<&Node, ToolError> {
        self.resolve
            .nodes
            .iter()
            .find(|node| node.id == id)
            .ok_or_else(|| ToolError::missing(&format!("resolve node {id}"), "cargo metadata"))
    }

    /// The directory of package `id`, relative to the workspace root.
    fn directory(&self, id: &str) -> Result<String, ToolError> {
        let package = self
            .packages
            .iter()
            .find(|package| package.id == id)
            .ok_or_else(|| ToolError::missing(&format!("package {id}"), "cargo metadata"))?;
        let directory = Path::new(&package.manifest_path)
            .parent()
            .and_then(|directory| directory.strip_prefix(&self.workspace_root).ok())
            .ok_or_else(|| {
                ToolError::missing(
                    &format!("a directory under the workspace root for {}", package.name),
                    "cargo metadata",
                )
            })?;
        Ok(directory.to_string_lossy().into_owned())
    }

    /// The pathspecs of the code of package `id`.
    fn code(&self, id: &str) -> Result<[String; 3], ToolError> {
        let directory = self.directory(id)?;
        Ok([
            format!("{directory}/src"),
            format!("{directory}/Cargo.toml"),
            format!("{directory}/build.rs"),
        ])
    }
}
