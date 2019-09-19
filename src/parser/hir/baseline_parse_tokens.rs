use crate::errors::ShellError;
use crate::parser::{hir, hir::syntax_shape::ExpandContext, hir::ExpandExpression, TokensIterator};
use crate::Text;
use log::trace;

pub fn baseline_parse_tokens(
    token_nodes: &mut TokensIterator<'_>,
    context: &ExpandContext,
    source: &Text,
    origin: uuid::Uuid,
    syntax_type: impl ExpandExpression,
) -> Result<Vec<hir::Expression>, ShellError> {
    let mut exprs: Vec<hir::Expression> = vec![];

    loop {
        if token_nodes.at_end() {
            break;
        }

        let expr = baseline_parse_next_expr(token_nodes, context, source, origin, syntax_type)?;
        exprs.push(expr);
    }

    Ok(exprs)
}

pub fn baseline_parse_next_expr(
    tokens: &mut TokensIterator,
    context: &ExpandContext,
    source: &Text,
    origin: uuid::Uuid,
    syntax_type: impl ExpandExpression,
) -> Result<hir::Expression, ShellError> {
    let next = tokens
        .next()
        .ok_or_else(|| ShellError::string("Expected token, found none"))?;

    trace!(target: "nu::parser::parse_one_expr", "syntax_type={:?}, token={:?}", syntax_type, next);

    syntax_type.expand(tokens, context, source, origin)

    // match (syntax_type, next) {
    //     (SyntaxShape::Path, TokenNode::Token(token)) => {
    //         return baseline_parse_token_as_path(token, context, source)
    //     }

    //     (SyntaxShape::Path, token) => {
    //         return Err(ShellError::type_error(
    //             "Path",
    //             token.type_name().tagged(token.tag()),
    //         ))
    //     }

    //     (SyntaxShape::Pattern, TokenNode::Token(token)) => {
    //         return baseline_parse_token_as_pattern(token, context, source)
    //     }

    //     (SyntaxShape::Pattern, token) => {
    //         return Err(ShellError::type_error(
    //             "Path",
    //             token.type_name().tagged(token.tag()),
    //         ))
    //     }

    //     (SyntaxShape::String, TokenNode::Token(token)) => {
    //         return baseline_parse_token_as_string(token, source);
    //     }

    //     (SyntaxShape::String, token) => {
    //         return Err(ShellError::type_error(
    //             "String",
    //             token.type_name().tagged(token.tag()),
    //         ))
    //     }

    //     (SyntaxShape::Number, TokenNode::Token(token)) => {
    //         return Ok(baseline_parse_token_as_number(token, source)?);
    //     }

    //     (SyntaxShape::Number, token) => {
    //         return Err(ShellError::type_error(
    //             "Numeric",
    //             token.type_name().tagged(token.tag()),
    //         ))
    //     }

    //     // TODO: More legit member processing
    //     (SyntaxShape::Member, TokenNode::Token(token)) => {
    //         return baseline_parse_token_as_string(token, source);
    //     }

    //     (SyntaxShape::Member, token) => {
    //         return Err(ShellError::type_error(
    //             "member",
    //             token.type_name().tagged(token.tag()),
    //         ))
    //     }

    //     (SyntaxShape::CommandHead, TokenNode::Token(token)) => {
    //         return baseline_parse_token_as_command_head(token, source)
    //     }

    //     (SyntaxShape::CommandHead, token) => {
    //         return Err(ShellError::type_error(
    //             "command",
    //             token.type_name().tagged(token.tag()),
    //         ))
    //     }

    //     (SyntaxShape::Any, _) => {}
    //     (SyntaxShape::List, _) => {}
    //     (SyntaxShape::Literal, _) => {}
    //     (SyntaxShape::Variable, _) => {}
    //     (SyntaxShape::Binary, _) => {}
    //     (SyntaxShape::Block, _) => {}
    //     (SyntaxShape::Boolean, _) => {}
    // };

    // let first = baseline_parse_semantic_token(next, context, source)?;

    // let possible_op = tokens.peek();

    // let op = match possible_op {
    //     Some(TokenNode::Operator(op)) => op.clone(),
    //     _ => return Ok(first),
    // };

    // tokens.next();

    // let second = match tokens.next() {
    //     None => {
    //         return Err(ShellError::labeled_error(
    //             "Expected something after an operator",
    //             "operator",
    //             op.tag(),
    //         ))
    //     }
    //     Some(token) => baseline_parse_semantic_token(token, context, source)?,
    // };

    // // We definitely have a binary expression here -- let's see if we should coerce it into a block

    // match syntax_type {
    //     SyntaxShape::Any => {
    //         let tag = first.tag().until(second.tag());
    //         let binary = hir::Binary::new(first, op, second);
    //         let binary = hir::RawExpression::Binary(Box::new(binary));
    //         let binary = binary.tagged(tag);

    //         Ok(binary)
    //     }

    //     SyntaxShape::Block => {
    //         let tag = first.tag().until(second.tag());

    //         let path: Tagged<hir::RawExpression> = match first {
    //             Tagged {
    //                 item: hir::RawExpression::Literal(hir::Literal::Bare),
    //                 tag,
    //             } => {
    //                 let string = tag.slice(source).to_string().tagged(tag);
    //                 let path = hir::Path::new(
    //                     // TODO: Deal with synthetic nodes that have no representation at all in source
    //                     hir::RawExpression::Variable(hir::Variable::It(Tag::unknown()))
    //                         .tagged(Tag::unknown()),
    //                     vec![string],
    //                 );
    //                 let path = hir::RawExpression::Path(Box::new(path));
    //                 path.tagged(first.tag())
    //             }
    //             Tagged {
    //                 item: hir::RawExpression::Literal(hir::Literal::String(inner)),
    //                 tag,
    //             } => {
    //                 let string = inner.slice(source).to_string().tagged(tag);
    //                 let path = hir::Path::new(
    //                     // TODO: Deal with synthetic nodes that have no representation at all in source
    //                     hir::RawExpression::Variable(hir::Variable::It(Tag::unknown()))
    //                         .tagged_unknown(),
    //                     vec![string],
    //                 );
    //                 let path = hir::RawExpression::Path(Box::new(path));
    //                 path.tagged(first.tag())
    //             }
    //             Tagged {
    //                 item: hir::RawExpression::Variable(..),
    //                 ..
    //             } => first,
    //             Tagged { tag, item } => {
    //                 return Err(ShellError::labeled_error(
    //                     "The first part of an un-braced block must be a column name",
    //                     item.type_name(),
    //                     tag,
    //                 ))
    //             }
    //         };

    //         let binary = hir::Binary::new(path, op, second);
    //         let binary = hir::RawExpression::Binary(Box::new(binary));
    //         let binary = binary.tagged(tag);

    //         let block = hir::RawExpression::Block(vec![binary]);
    //         let block = block.tagged(tag);

    //         Ok(block)
    //     }

    //     other => Err(ShellError::unimplemented(format!(
    //         "coerce hint {:?}",
    //         other
    //     ))),
    // }
}
