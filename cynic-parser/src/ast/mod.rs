use crate::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

mod generated {
    mod tokens;
}

trait AstToken: Sized {
    fn can_cast(kind: SyntaxKind) -> bool;
    fn cast(syntax: SyntaxToken) -> Option<Self>;
    fn syntax(&self) -> &SyntaxToken;
}
