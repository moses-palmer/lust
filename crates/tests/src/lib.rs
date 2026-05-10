#[cfg(test)]
mod tests {
    const _AST: lust::ast::Node = lust::ast! {r#"
        (do (
            (+ 1 2 3)
            (list )))
    "#};
}
