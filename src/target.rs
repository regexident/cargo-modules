// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_project_model::{CargoWorkspace, Package, Target, TargetKind};

use crate::options::project::Options;

pub fn select_package(workspace: &CargoWorkspace, options: &Options) -> anyhow::Result<Package> {
    let packages: Vec<_> = workspace
        .packages()
        .filter(|idx| workspace[*idx].is_member)
        .collect();

    let package_count = packages.len();

    // If project contains no packages, bail out:

    if package_count < 1 {
        anyhow::bail!("No packages found");
    }

    // If no (or a non-existent) package was provided via options bail out:

    let package_list_items: Vec<_> = packages
        .iter()
        .map(|package_idx| {
            let package = &workspace[*package_idx];
            format!("- {}", package.name)
        })
        .collect();

    let package_list = package_list_items.join("\n");

    // If project contains multiple packages, select the one provided via options:

    if let Some(package_name) = &options.package {
        let package_idx = packages.into_iter().find(|package_idx| {
            let package = &workspace[*package_idx];
            package.name == *package_name
        });

        return package_idx.ok_or_else(|| {
            anyhow::anyhow!(
                indoc::indoc! {
                    "No package found with name {:?}.

                        Packages present in workspace:
                        {}
                        "
                },
                package_name,
                package_list,
            )
        });
    }

    // If project contains a single packages, just pick it:

    if package_count == 1 {
        return Ok(packages[0]);
    }

    Err(anyhow::anyhow!(
        indoc::indoc! {
            "Multiple packages present in workspace,
                please explicitly select one via --package flag.

                Packages present in workspace:
                {}
                "
        },
        package_list
    ))
}

pub fn select_target(
    workspace: &CargoWorkspace,
    package_idx: Package,
    options: &Options,
) -> anyhow::Result<Target> {
    let package = &workspace[package_idx];

    // Retrieve list of indices for bin/lib targets:

    let targets: Vec<_> = package
        .targets
        .iter()
        .cloned()
        .filter(|target_idx| {
            let target = &workspace[*target_idx];
            match target.kind {
                TargetKind::Bin => true,
                TargetKind::Lib => true,
                TargetKind::Example => false,
                TargetKind::Test => false,
                TargetKind::Bench => false,
                TargetKind::Other => false,
                TargetKind::BuildScript => false,
            }
        })
        .collect();

    let target_count = targets.len();

    // If package contains no targets, bail out:

    if target_count < 1 {
        anyhow::bail!("No targets found");
    }

    // If no (or a non-existent) target was provided via options bail out:

    let target_list_items: Vec<_> = targets
        .iter()
        .map(|target_idx| {
            let target = &workspace[*target_idx];
            match target.kind {
                TargetKind::Bin => format!("- {} (--bin {})", target.name, target.name),
                TargetKind::Lib => format!("- {} (--lib)", target.name),
                TargetKind::Example => unreachable!(),
                TargetKind::Test => unreachable!(),
                TargetKind::Bench => unreachable!(),
                TargetKind::Other => unreachable!(),
                TargetKind::BuildScript => unreachable!(),
            }
        })
        .collect();

    let target_list = target_list_items.join("\n");

    // If package contains multiple targets, select the one provided via options:

    if options.lib {
        let target = targets.into_iter().find(|target_idx| {
            let target = &workspace[*target_idx];
            target.kind == TargetKind::Lib
        });

        return target.ok_or_else(|| {
            anyhow::anyhow!(
                indoc::indoc! {
                    "No library target found.

                        Targets present in package:
                        {}
                        "
                },
                target_list,
            )
        });
    }

    if let Some(bin_name) = &options.bin {
        let target = targets.into_iter().find(|target_idx| {
            let target = &workspace[*target_idx];
            (target.kind == TargetKind::Bin) && (target.name == bin_name[..])
        });

        return target.ok_or_else(|| {
            anyhow::anyhow!(
                indoc::indoc! {
                    "No binary target found with name {:?}.

                        Targets present in package:
                        {}
                        "
                },
                bin_name,
                target_list,
            )
        });
    }

    // If project contains a single target, just pick it:

    if target_count == 1 {
        return Ok(targets[0]);
    }

    Err(anyhow::anyhow!(
        indoc::indoc! {
            "Multiple targets present in package,
                please explicitly select one via --lib or --bin flag.

                Targets present in package:
                {}
                "
        },
        target_list
    ))
}
