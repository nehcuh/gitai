/// Determine if TreeSitter should be used
fn should_use_tree_sitter(args: &ReviewArgs) -> bool {
    args.tree_sitter || args.review_ts || (!args.no_tree_sitter)
}
