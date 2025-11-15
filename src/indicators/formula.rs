use std::collections::{BTreeSet, HashMap};
use std::iter::Peekable;
use std::str::Chars;
use std::sync::Arc;

use crate::indicators::types::{IndicatorError, OHLCData};

#[derive(Clone, Debug)]
pub struct FormulaDefinition {
    expression: String,
    root: FormulaNode,
    dependencies: BTreeSet<String>,
}

impl FormulaDefinition {
    pub fn parse(expression: &str) -> Result<Self, IndicatorError> {
        let tokens = tokenize(expression)?;
        let mut parser = FormulaParser::new(tokens);
        let root = parser.parse_expression(1)?;
        if parser.has_remaining() {
            return Err(IndicatorError::FormulaError("unexpected token".to_string()));
        }
        let mut dependencies = BTreeSet::new();
        collect_dependencies(&root, &mut dependencies);
        Ok(Self {
            expression: expression.trim().to_string(),
            root,
            dependencies,
        })
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }

    pub fn dependencies(&self) -> &BTreeSet<String> {
        &self.dependencies
    }

    pub fn data_dependencies(&self) -> impl Iterator<Item = &String> {
        self.dependencies
            .iter()
            .filter(|name| !is_builtin_identifier(name))
    }

    pub fn evaluate(
        &self,
        context: &FormulaEvaluationContext<'_>,
    ) -> Result<Vec<f32>, IndicatorError> {
        let length = context.length_for(self)?;
        let mut result = Vec::with_capacity(length);
        for idx in 0..length {
            let point = FormulaPointContext::new(context, idx);
            let value = self.root.evaluate(&point)?;
            result.push(value.as_number()?);
        }
        Ok(result)
    }
}

pub struct FormulaEvaluationContext<'a> {
    ohlc: &'a OHLCData,
    indicators: &'a HashMap<String, Arc<Vec<f32>>>,
}

impl<'a> FormulaEvaluationContext<'a> {
    pub fn new(ohlc: &'a OHLCData, indicators: &'a HashMap<String, Arc<Vec<f32>>>) -> Self {
        Self { ohlc, indicators }
    }

    pub fn length_for(&self, definition: &FormulaDefinition) -> Result<usize, IndicatorError> {
        let mut length = self.ohlc.len();
        for dependency in definition.dependencies() {
            if is_builtin_identifier(dependency) {
                continue;
            }
            let Some(series) = self.indicators.get(dependency) else {
                return Err(IndicatorError::FormulaError(format!(
                    "missing dependency {}",
                    dependency
                )));
            };
            length = length.min(series.len());
        }
        if length == 0 {
            return Err(IndicatorError::FormulaError(
                "no data available".to_string(),
            ));
        }
        Ok(length)
    }

    pub fn value(&self, name: &str, index: usize) -> Option<f32> {
        if let Some(series) = self.indicators.get(name) {
            return series.get(index).copied();
        }
        match name.to_ascii_lowercase().as_str() {
            "open" => self.ohlc.open.get(index).copied(),
            "high" => self.ohlc.high.get(index).copied(),
            "low" => self.ohlc.low.get(index).copied(),
            "close" => self.ohlc.close.get(index).copied(),
            "volume" => self
                .ohlc
                .volume
                .as_ref()
                .and_then(|values| values.get(index).copied()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
enum FormulaNode {
    Number(f32),
    Identifier(String),
    Unary(UnaryOp, Box<FormulaNode>),
    Binary(BinaryOp, Box<FormulaNode>, Box<FormulaNode>),
    Function(String, Vec<FormulaNode>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnaryOp {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
    And,
    Or,
}

#[derive(Clone, Copy)]
struct FormulaPointContext<'ctx, 'data> {
    context: &'ctx FormulaEvaluationContext<'data>,
    index: usize,
}

impl<'ctx, 'data> FormulaPointContext<'ctx, 'data> {
    fn new(context: &'ctx FormulaEvaluationContext<'data>, index: usize) -> Self {
        Self { context, index }
    }

    fn resolve(&self, name: &str) -> Result<FormulaScalar, IndicatorError> {
        if let Some(value) = self.context.value(name, self.index) {
            return Ok(FormulaScalar::Number(value));
        }
        match name.to_ascii_lowercase().as_str() {
            "true" => Ok(FormulaScalar::Bool(true)),
            "false" => Ok(FormulaScalar::Bool(false)),
            _ => Err(IndicatorError::FormulaError(format!(
                "unknown identifier {}",
                name
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum FormulaScalar {
    Number(f32),
    Bool(bool),
}

impl FormulaScalar {
    fn as_number(self) -> Result<f32, IndicatorError> {
        Ok(match self {
            Self::Number(value) => value,
            Self::Bool(flag) => {
                if flag {
                    1.0
                } else {
                    0.0
                }
            }
        })
    }

    fn as_bool(self) -> Result<bool, IndicatorError> {
        Ok(match self {
            Self::Number(value) => value != 0.0,
            Self::Bool(flag) => flag,
        })
    }
}

impl FormulaNode {
    fn evaluate(
        &self,
        point: &FormulaPointContext<'_, '_>,
    ) -> Result<FormulaScalar, IndicatorError> {
        match self {
            Self::Number(value) => Ok(FormulaScalar::Number(*value)),
            Self::Identifier(name) => point.resolve(name),
            Self::Unary(op, expr) => {
                let value = expr.evaluate(point)?;
                match op {
                    UnaryOp::Negate => Ok(FormulaScalar::Number(-value.as_number()?)),
                    UnaryOp::Not => Ok(FormulaScalar::Bool(!value.as_bool()?)),
                }
            }
            Self::Binary(op, left, right) => {
                let lhs = left.evaluate(point)?;
                let rhs = right.evaluate(point)?;
                match op {
                    BinaryOp::Add => Ok(FormulaScalar::Number(lhs.as_number()? + rhs.as_number()?)),
                    BinaryOp::Sub => Ok(FormulaScalar::Number(lhs.as_number()? - rhs.as_number()?)),
                    BinaryOp::Mul => Ok(FormulaScalar::Number(lhs.as_number()? * rhs.as_number()?)),
                    BinaryOp::Div => {
                        let divisor = rhs.as_number()?;
                        if divisor == 0.0 {
                            return Err(IndicatorError::FormulaError(
                                "division by zero".to_string(),
                            ));
                        }
                        Ok(FormulaScalar::Number(lhs.as_number()? / divisor))
                    }
                    BinaryOp::Pow => Ok(FormulaScalar::Number(
                        lhs.as_number()?.powf(rhs.as_number()?),
                    )),
                    BinaryOp::Lt => Ok(FormulaScalar::Bool(lhs.as_number()? < rhs.as_number()?)),
                    BinaryOp::Lte => Ok(FormulaScalar::Bool(lhs.as_number()? <= rhs.as_number()?)),
                    BinaryOp::Gt => Ok(FormulaScalar::Bool(lhs.as_number()? > rhs.as_number()?)),
                    BinaryOp::Gte => Ok(FormulaScalar::Bool(lhs.as_number()? >= rhs.as_number()?)),
                    BinaryOp::Eq => Ok(FormulaScalar::Bool(lhs.as_number()? == rhs.as_number()?)),
                    BinaryOp::Neq => Ok(FormulaScalar::Bool(lhs.as_number()? != rhs.as_number()?)),
                    BinaryOp::And => Ok(FormulaScalar::Bool(lhs.as_bool()? && rhs.as_bool()?)),
                    BinaryOp::Or => Ok(FormulaScalar::Bool(lhs.as_bool()? || rhs.as_bool()?)),
                }
            }
            Self::Function(name, args) => evaluate_function(name, args, point),
        }
    }
}

fn evaluate_function(
    name: &str,
    args: &[FormulaNode],
    point: &FormulaPointContext<'_, '_>,
) -> Result<FormulaScalar, IndicatorError> {
    let key = name.to_ascii_lowercase();
    match key.as_str() {
        "abs" => {
            if args.len() != 1 {
                return Err(IndicatorError::FormulaError(format!(
                    "abs expects 1 argument, got {}",
                    args.len()
                )));
            }
            Ok(FormulaScalar::Number(
                args[0].evaluate(point)?.as_number()?.abs(),
            ))
        }
        "sum" => {
            if args.is_empty() {
                return Err(IndicatorError::FormulaError(
                    "sum expects arguments".to_string(),
                ));
            }
            let mut total = 0.0;
            for arg in args {
                total += arg.evaluate(point)?.as_number()?;
            }
            Ok(FormulaScalar::Number(total))
        }
        "avg" => {
            if args.is_empty() {
                return Err(IndicatorError::FormulaError(
                    "avg expects arguments".to_string(),
                ));
            }
            let mut total = 0.0;
            for arg in args {
                total += arg.evaluate(point)?.as_number()?;
            }
            Ok(FormulaScalar::Number(total / args.len() as f32))
        }
        "min" => {
            if args.is_empty() {
                return Err(IndicatorError::FormulaError(
                    "min expects arguments".to_string(),
                ));
            }
            let mut value = args[0].evaluate(point)?.as_number()?;
            for arg in &args[1..] {
                value = value.min(arg.evaluate(point)?.as_number()?);
            }
            Ok(FormulaScalar::Number(value))
        }
        "max" => {
            if args.is_empty() {
                return Err(IndicatorError::FormulaError(
                    "max expects arguments".to_string(),
                ));
            }
            let mut value = args[0].evaluate(point)?.as_number()?;
            for arg in &args[1..] {
                value = value.max(arg.evaluate(point)?.as_number()?);
            }
            Ok(FormulaScalar::Number(value))
        }
        "if" => {
            if args.len() != 3 {
                return Err(IndicatorError::FormulaError(format!(
                    "if expects 3 arguments, got {}",
                    args.len()
                )));
            }
            let condition = args[0].evaluate(point)?.as_bool()?;
            if condition {
                Ok(FormulaScalar::Number(args[1].evaluate(point)?.as_number()?))
            } else {
                Ok(FormulaScalar::Number(args[2].evaluate(point)?.as_number()?))
            }
        }
        _ => Err(IndicatorError::FormulaError(format!(
            "unknown function {}",
            name
        ))),
    }
}

fn collect_dependencies(node: &FormulaNode, target: &mut BTreeSet<String>) {
    match node {
        FormulaNode::Number(_) => {}
        FormulaNode::Identifier(name) => {
            target.insert(name.clone());
        }
        FormulaNode::Unary(_, expr) => collect_dependencies(expr, target),
        FormulaNode::Binary(_, left, right) => {
            collect_dependencies(left, target);
            collect_dependencies(right, target);
        }
        FormulaNode::Function(_, args) => {
            for arg in args {
                collect_dependencies(arg, target);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f32),
    Identifier(String),
    Operator(String),
    LParen,
    RParen,
    Comma,
}

struct FormulaParser {
    tokens: Vec<Token>,
    index: usize,
}

impl FormulaParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    fn has_remaining(&self) -> bool {
        self.index < self.tokens.len()
    }

    fn parse_expression(&mut self, min_precedence: u8) -> Result<FormulaNode, IndicatorError> {
        let mut left = self.parse_prefix()?;
        loop {
            let op = match self.peek_binary_op() {
                Some(op) => op,
                None => break,
            };
            let precedence = op.precedence();
            if precedence < min_precedence {
                break;
            }
            self.index += 1;
            let next_min = if op.associativity() == Assoc::Left {
                precedence + 1
            } else {
                precedence
            };
            let right = self.parse_expression(next_min)?;
            left = FormulaNode::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<FormulaNode, IndicatorError> {
        let token = self.next_token().ok_or_else(|| {
            IndicatorError::FormulaError("unexpected end of expression".to_string())
        })?;
        match token {
            Token::Number(value) => Ok(FormulaNode::Number(value)),
            Token::Identifier(name) => {
                if self
                    .peek_token()
                    .is_some_and(|tok| matches!(tok, Token::LParen))
                {
                    self.index += 1;
                    let args = self.parse_arguments()?;
                    Ok(FormulaNode::Function(name, args))
                } else {
                    Ok(FormulaNode::Identifier(name))
                }
            }
            Token::Operator(op) if op == "-" => {
                let expr = self.parse_expression(7)?;
                Ok(FormulaNode::Unary(UnaryOp::Negate, Box::new(expr)))
            }
            Token::Operator(op) if op == "!" => {
                let expr = self.parse_expression(7)?;
                Ok(FormulaNode::Unary(UnaryOp::Not, Box::new(expr)))
            }
            Token::LParen => {
                let expr = self.parse_expression(1)?;
                self.expect_rparen()?;
                Ok(expr)
            }
            other => Err(IndicatorError::FormulaError(format!(
                "unsupported token {:?}",
                other
            ))),
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<FormulaNode>, IndicatorError> {
        let mut args = Vec::new();
        if self
            .peek_token()
            .is_some_and(|tok| matches!(tok, Token::RParen))
        {
            self.index += 1;
            return Ok(args);
        }
        loop {
            let expr = self.parse_expression(1)?;
            args.push(expr);
            match self.next_token() {
                Some(Token::Comma) => continue,
                Some(Token::RParen) => break,
                Some(token) => {
                    return Err(IndicatorError::FormulaError(format!(
                        "unexpected token {:?}",
                        token
                    )))
                }
                None => {
                    return Err(IndicatorError::FormulaError(
                        "unexpected end of expression".to_string(),
                    ))
                }
            }
        }
        Ok(args)
    }

    fn expect_rparen(&mut self) -> Result<(), IndicatorError> {
        match self.next_token() {
            Some(Token::RParen) => Ok(()),
            other => Err(IndicatorError::FormulaError(format!(
                "expected ')', got {:?}",
                other
            ))),
        }
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.index >= self.tokens.len() {
            None
        } else {
            let token = self.tokens[self.index].clone();
            self.index += 1;
            Some(token)
        }
    }

    fn peek_binary_op(&self) -> Option<BinaryOp> {
        match self.tokens.get(self.index) {
            Some(Token::Operator(value)) => BinaryOp::from_symbol(value),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assoc {
    Left,
    Right,
}

impl BinaryOp {
    fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "+" => Some(Self::Add),
            "-" => Some(Self::Sub),
            "*" => Some(Self::Mul),
            "/" => Some(Self::Div),
            "^" => Some(Self::Pow),
            "<" => Some(Self::Lt),
            "<=" => Some(Self::Lte),
            ">" => Some(Self::Gt),
            ">=" => Some(Self::Gte),
            "==" => Some(Self::Eq),
            "!=" => Some(Self::Neq),
            "&&" => Some(Self::And),
            "||" => Some(Self::Or),
            _ => None,
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Eq | Self::Neq => 3,
            Self::Lt | Self::Lte | Self::Gt | Self::Gte => 4,
            Self::Add | Self::Sub => 5,
            Self::Mul | Self::Div => 6,
            Self::Pow => 7,
        }
    }

    fn associativity(&self) -> Assoc {
        match self {
            Self::Pow => Assoc::Right,
            _ => Assoc::Left,
        }
    }
}

fn tokenize(expression: &str) -> Result<Vec<Token>, IndicatorError> {
    let mut tokens = Vec::new();
    let mut chars = expression.chars().peekable();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        match ch {
            '0'..='9' | '.' => tokens.push(parse_number(&mut chars)?),
            'a'..='z' | 'A'..='Z' | '_' => tokens.push(parse_identifier(&mut chars)),
            '+' | '-' | '*' | '/' | '^' | '>' | '<' | '=' | '!' => {
                tokens.push(parse_operator(&mut chars)?);
            }
            '&' | '|' => tokens.push(parse_operator(&mut chars)?),
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            _ => {
                return Err(IndicatorError::FormulaError(format!(
                    "invalid character {}",
                    ch
                )))
            }
        }
    }
    Ok(tokens)
}

fn parse_number(chars: &mut Peekable<Chars<'_>>) -> Result<Token, IndicatorError> {
    let mut buffer = String::new();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_digit() || ch == '.' {
            buffer.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    let value = buffer
        .parse::<f32>()
        .map_err(|_| IndicatorError::FormulaError(format!("invalid number {}", buffer)))?;
    Ok(Token::Number(value))
}

fn parse_identifier(chars: &mut Peekable<Chars<'_>>) -> Token {
    let mut buffer = String::new();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_alphanumeric() || ch == '_' {
            buffer.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    Token::Identifier(buffer)
}

fn parse_operator(chars: &mut Peekable<Chars<'_>>) -> Result<Token, IndicatorError> {
    let first = chars.next().unwrap();
    let mut symbol = String::new();
    symbol.push(first);
    if let Some(next) = chars.peek().copied() {
        if matches!(
            (first, next),
            ('>', '=') | ('<', '=') | ('=', '=') | ('!', '=') | ('&', '&') | ('|', '|')
        ) {
            symbol.push(next);
            chars.next();
        }
    }
    match symbol.as_str() {
        "+" | "-" | "*" | "/" | "^" | ">" | ">=" | "<" | "<=" | "==" | "!=" | "&&" | "||" | "!" => {
            Ok(Token::Operator(symbol))
        }
        _ => Err(IndicatorError::FormulaError(format!(
            "unsupported operator {}",
            symbol
        ))),
    }
}

pub fn is_builtin_identifier(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "open" | "high" | "low" | "close" | "volume" | "true" | "false"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ohlc() -> OHLCData {
        OHLCData::new(
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
            vec![0.5, 1.5, 2.5],
            vec![1.5, 2.5, 3.5],
        )
    }

    #[test]
    fn parses_dependencies() {
        let definition = FormulaDefinition::parse("fast - slow").unwrap();
        let deps: Vec<_> = definition.data_dependencies().cloned().collect();
        assert_eq!(deps, vec!["fast".to_string(), "slow".to_string()]);
    }

    #[test]
    fn evaluates_difference() {
        let mut indicators = HashMap::new();
        indicators.insert("fast".to_string(), Arc::new(vec![3.0, 4.0, 5.0]));
        indicators.insert("slow".to_string(), Arc::new(vec![1.0, 2.0, 3.0]));
        let ohlc = sample_ohlc();
        let ctx = FormulaEvaluationContext::new(&ohlc, &indicators);
        let definition = FormulaDefinition::parse("fast - slow").unwrap();
        let result = definition.evaluate(&ctx).unwrap();
        assert_eq!(result, vec![2.0, 2.0, 2.0]);
    }

    #[test]
    fn evaluates_if_function() {
        let mut indicators = HashMap::new();
        indicators.insert("fast".to_string(), Arc::new(vec![3.0, 1.0, 5.0]));
        indicators.insert("slow".to_string(), Arc::new(vec![2.0, 2.0, 2.0]));
        let ohlc = sample_ohlc();
        let ctx = FormulaEvaluationContext::new(&ohlc, &indicators);
        let definition = FormulaDefinition::parse("if(fast > slow, fast, slow)").unwrap();
        let result = definition.evaluate(&ctx).unwrap();
        assert_eq!(result, vec![3.0, 2.0, 5.0]);
    }
}
