pub(crate) use super::req_parser::Rule;
pub(crate) use super::req_parser::ShioriRequestParser as Parser;

pub(crate) type ParseError = pest::error::Error<Rule>;
