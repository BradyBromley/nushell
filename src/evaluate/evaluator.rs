use crate::data::base::Block;
use crate::errors::{ArgumentError, Description};
use crate::parser::{
    hir::{self, Expression, RawExpression},
    CommandRegistry, Text,
};
use crate::prelude::*;
use derive_new::new;
use indexmap::IndexMap;

#[derive(new)]
pub struct Scope {
    it: Tagged<Value>,
    #[new(default)]
    vars: IndexMap<String, Tagged<Value>>,
}

impl Scope {
    pub(crate) fn empty() -> Scope {
        Scope {
            it: Value::nothing().tagged_unknown(),
            vars: IndexMap::new(),
        }
    }

    pub(crate) fn it_value(value: Tagged<Value>) -> Scope {
        Scope {
            it: value,
            vars: IndexMap::new(),
        }
    }
}

pub(crate) fn evaluate_baseline_expr(
    expr: &Expression,
    registry: &CommandRegistry,
    scope: &Scope,
    source: &Text,
) -> Result<Tagged<Value>, ShellError> {
    match &expr.item {
        RawExpression::Literal(literal) => Ok(evaluate_literal(expr.copy_tag(literal), source)),
        RawExpression::ExternalWord => Err(ShellError::argument_error(
            "Invalid external word",
            ArgumentError::InvalidExternalWord,
            expr.tag(),
        )),
        RawExpression::FilePath(path) => Ok(Value::path(path.clone()).tagged(expr.tag())),
        RawExpression::Synthetic(hir::Synthetic::String(s)) => {
            Ok(Value::string(s).tagged_unknown())
        }
        RawExpression::Variable(var) => evaluate_reference(var, scope, source, expr.tag()),
        RawExpression::Command(tag) => evaluate_command(expr.tag(), scope, source),
        RawExpression::ExternalCommand(external) => evaluate_external(external, scope, source),
        RawExpression::Binary(binary) => {
            let left = evaluate_baseline_expr(binary.left(), registry, scope, source)?;
            let right = evaluate_baseline_expr(binary.right(), registry, scope, source)?;

            match left.compare(binary.op(), &*right) {
                Ok(result) => Ok(Value::boolean(result).tagged(expr.tag())),
                Err((left_type, right_type)) => Err(ShellError::coerce_error(
                    binary.left().copy_tag(left_type),
                    binary.right().copy_tag(right_type),
                )),
            }
        }
        RawExpression::List(list) => {
            let mut exprs = vec![];

            for expr in list {
                let expr = evaluate_baseline_expr(expr, registry, scope, source)?;
                exprs.push(expr);
            }

            Ok(Value::Table(exprs).tagged(expr.tag()))
        }
        RawExpression::Block(block) => {
            Ok(
                Value::Block(Block::new(block.clone(), source.clone(), expr.tag()))
                    .tagged(expr.tag()),
            )
        }
        RawExpression::Path(path) => {
            let value = evaluate_baseline_expr(path.head(), registry, scope, source)?;
            let mut item = value;

            for name in path.tail() {
                let next = item.get_data_by_key(name);

                match next {
                    None => {
                        return Err(ShellError::missing_property(
                            Description::from(item.tagged_type_name()),
                            Description::from(name.clone()),
                        ))
                    }
                    Some(next) => {
                        item = next.clone().item.tagged(expr.tag());
                    }
                };
            }

            Ok(item.item().clone().tagged(expr.tag()))
        }
        RawExpression::Boolean(_boolean) => unimplemented!(),
    }
}

fn evaluate_literal(literal: Tagged<&hir::Literal>, source: &Text) -> Tagged<Value> {
    let result = match literal.item {
        hir::Literal::Number(int) => int.into(),
        hir::Literal::Size(int, unit) => unit.compute(int),
        hir::Literal::String(tag) => Value::string(tag.slice(source)),
        hir::Literal::GlobPattern => Value::pattern(literal.tag().slice(source)),
        hir::Literal::Bare => Value::string(literal.tag().slice(source)),
    };

    literal.map(|_| result)
}

fn evaluate_reference(
    name: &hir::Variable,
    scope: &Scope,
    source: &Text,
    tag: Tag,
) -> Result<Tagged<Value>, ShellError> {
    match name {
        hir::Variable::It(_) => Ok(scope.it.item.clone().tagged(tag)),
        hir::Variable::Other(inner) => Ok(scope
            .vars
            .get(inner.slice(source))
            .map(|v| v.clone())
            .unwrap_or_else(|| Value::nothing().tagged(tag))),
    }
}

fn evaluate_external(
    external: &hir::ExternalCommand,
    _scope: &Scope,
    _source: &Text,
) -> Result<Tagged<Value>, ShellError> {
    Err(ShellError::syntax_error(
        "Unexpected external command".tagged(*external.name()),
    ))
}

fn evaluate_command(tag: Tag, _scope: &Scope, _source: &Text) -> Result<Tagged<Value>, ShellError> {
    Err(ShellError::syntax_error("Unexpected command".tagged(tag)))
}
