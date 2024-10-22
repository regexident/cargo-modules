// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_syntax::ast;

use anyhow::bail;

pub fn sanitized_use_tree(
    focus_on: Option<&str>,
    crate_name: &str,
) -> anyhow::Result<ast::UseTree> {
    let mut path_expr = focus_on.unwrap_or(crate_name).to_owned();

    // Trim leading `::` from `use` expression,
    // expressions are implied to be absolute:
    let double_colon_prefix = "::";
    if path_expr.starts_with(double_colon_prefix) {
        let range = 0..(double_colon_prefix.len() - 2);
        path_expr.replace_range(range, "");
    }

    let crate_prefix = "crate::";
    if path_expr.starts_with(crate_prefix) {
        let range = 0..(crate_prefix.len() - 2);
        path_expr.replace_range(range, crate_name);
    }

    for keyword in ["super", "self", "$crate"] {
        let keyword_prefix = format!("{keyword}::");

        if path_expr == keyword || path_expr.starts_with(&keyword_prefix) {
            bail!("unexpected keyword `{keyword}` in `--focus-on` option");
        }
    }

    Ok(crate::analyzer::parse_use_tree(&path_expr))
}
