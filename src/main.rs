use std::{
    collections::HashMap, env, fs, io, path::PathBuf,
};

use glob::glob;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct WorkspacePackageJson {
    workspaces: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    name: String,
    dependencies: HashMap<String, String>,
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    args.next();

    let project_dir =
        args.next().expect("requires a project_dir");
    let workspace_name =
        args.next().expect("requires a workspace_name");
    let dependency_name =
        args.next().expect("requires a dependency");

    let root_package_path =
        PathBuf::from(&project_dir).join("package.json");
    let root_package_json =
        fs::read_to_string(root_package_path)?;

    let json: WorkspacePackageJson =
        serde_json::from_str(&root_package_json)?;

    let packages = json
        .workspaces
        .iter()
        .flat_map(|workspace_glob| {
            glob(
                &PathBuf::from(&project_dir)
                    .join(&workspace_glob)
                    .display()
                    .to_string(),
            )
            .expect("glob failed")
            .filter_map(|entry| match entry {
                Ok(path) => {
                    let package_json =
                        path.join("package.json");

                    let json: Option<PackageJson> =
                        fs::read_to_string(package_json)
                            .ok()
                            .and_then(|contents| {
                                serde_json::from_str(
                                    &contents,
                                )
                                .ok()
                            });
                    json
                }
                Err(e) => None,
            })
            .collect::<Vec<PackageJson>>()
        })
        .collect::<Vec<PackageJson>>();
    let semver = packages
        .iter()
        .find(|package| package.name == workspace_name)
        .and_then(|package_json| {
            package_json.dependencies.iter().find_map(
                |(package_name, version)| {
                    if **package_name == dependency_name {
                        Some(version)
                    } else {
                        None
                    }
                },
            )
        });
    match semver {
        Some(version) => {
            println!(
                "dep `{}` in workspace `{}` is `{}`",
                dependency_name, workspace_name, version
            )
        }
        None => println!(
            "could not find dep `{}` in workspace `{}`",
            dependency_name, workspace_name
        ),
    }
    Ok(())
}
