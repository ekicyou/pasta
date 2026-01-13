use crate::error::*;
use pest;
use pest::Parser as PestParser;
use pest::iterators::FlatPairs;
use std::collections::HashMap;

pub use super::req_parser::Rule;
pub use super::req_parser::ShioriRequestParser as Parser;

pub type ParseError = pest::error::Error<Rule>;

/// SHIORI3リクエストの解析結果を格納します。
#[derive(PartialEq, Eq, Debug)]
pub struct ShioriRequest<'a> {
    pub text: &'a str,
    pub version: i32,
    pub method: Rule,
    pub id: Option<&'a str>,
    pub sender: Option<&'a str>,
    pub security_level: Option<&'a str>,
    pub charset: Option<&'a str>,
    pub status: Option<&'a str>,
    pub base_id: Option<&'a str>,
    pub reference: Vec<(i32, &'a str)>,
    pub dic: HashMap<String, &'a str>,
    pub key_values: Vec<(Rule, &'a str, &'a str)>,
}

impl<'a> ShioriRequest<'a> {
    #[allow(dead_code)]
    pub fn parse(text: &'a str) -> MyResult<ShioriRequest<'a>> {
        let rc = ShioriRequest::new(text);
        let it = Parser::parse(Rule::req, text)?.flatten();
        rc.parse1(it)
    }

    #[allow(dead_code)]
    fn new(text: &'a str) -> ShioriRequest<'a> {
        ShioriRequest {
            text,
            version: 0,
            method: Rule::req,
            id: None,
            sender: None,
            security_level: None,
            charset: None,
            status: None,
            base_id: None,
            dic: HashMap::new(),
            key_values: Vec::new(),
            reference: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn parse1(mut self, mut it: FlatPairs<'a, Rule>) -> MyResult<ShioriRequest<'a>> {
        let pair = match it.next() {
            Some(a) => a,
            None => return Ok(self),
        };
        let rule = pair.as_rule();
        match rule {
            Rule::key_value => self.parse_key_value(&mut it)?,
            Rule::get => self.method = rule,
            Rule::notify => self.method = rule,
            Rule::header3 => self.version = 30,
            Rule::shiori2_id => self.id = Some(pair.as_str()),
            Rule::shiori2_ver => {
                self.version = {
                    let nums: i32 = pair.as_str().parse().unwrap();
                    if nums < 0 {
                        20
                    } else if nums > 9 {
                        29
                    } else {
                        nums + 20
                    }
                };
            }
            _ => (),
        };
        self.parse1(it)
    }

    #[allow(dead_code)]
    fn parse_key_value(&mut self, it: &mut FlatPairs<'a, Rule>) -> MyResult<()> {
        let pair = it.next().unwrap();
        let rule = pair.as_rule();
        let key = pair.as_str();
        let value = match rule {
            Rule::key_ref => {
                let nums = it.next().unwrap().as_str().parse().unwrap();
                let value = it.next().unwrap().as_str();
                self.reference.push((nums, value));
                value
            }
            _ => {
                let value = it.next().unwrap().as_str();
                match rule {
                    Rule::key_charset => self.charset = Some(value),
                    Rule::key_id => self.id = Some(value),
                    Rule::key_base_id => self.base_id = Some(value),
                    Rule::key_status => self.status = Some(value),
                    Rule::key_security_level => self.security_level = Some(value),
                    Rule::key_sender => self.sender = Some(value),
                    Rule::key_other => (),
                    _ => panic!(),
                };
                value
            }
        };
        self.dic.entry(key.into()).or_insert(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn req_1() {
        let src = include_str!("test_data/shiori3-1.txt")
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n", "\r\n");
        let grammar = src.as_str();

        let req = ShioriRequest::parse(grammar).unwrap_or_else(|e| panic!("{}", e));
        assert_eq!(req.version, 30);
        assert_eq!(req.charset.unwrap(), "UTF-8");
        assert_eq!(req.sender.unwrap(), "SSP");
        assert_eq!(req.security_level.unwrap(), "local");
        assert_eq!(req.method, Rule::get);
        assert_eq!(req.id.unwrap(), "version");
        assert_eq!(req.status, None);
        assert_eq!(req.base_id, None);

        assert_eq!(req.dic.len(), 4);
        assert_eq!(req.dic["Charset"], "UTF-8");
        assert_eq!(req.dic["ID"], "version");
        assert_eq!(req.dic["SecurityLevel"], "local");
        assert_eq!(req.dic["Sender"], "SSP");

        assert_eq!(req.key_values.len(), 0);

        assert_eq!(req.reference.len(), 0);
    }

    #[test]
    fn req_2() {
        let src = include_str!("test_data/shiori3-2.txt")
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n", "\r\n");
        let grammar = src.as_str();

        let req = ShioriRequest::parse(grammar).unwrap_or_else(|e| panic!("{}", e));

        assert_eq!(req.version, 30);
        assert_eq!(req.charset.unwrap(), "UTF-8");
        assert_eq!(req.sender.unwrap(), "SSP");
        assert_eq!(req.security_level.unwrap(), "local");
        assert_eq!(req.method, Rule::notify);
        assert_eq!(req.id.unwrap(), "ownerghostname");
        assert_eq!(req.status, None);
        assert_eq!(req.base_id, None);

        assert_eq!(req.dic.len(), 5);
        assert_eq!(req.dic["Reference0"], "セキュリティボール");

        assert_eq!(req.key_values.len(), 0);

        assert_eq!(req.reference.len(), 1);
        let mut it = req.reference.into_iter();
        assert_eq!(it.next().unwrap(), (0i32, "セキュリティボール"));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn req_3() {
        let src = include_str!("test_data/shiori2-1.txt")
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n", "\r\n");
        let grammar = src.as_str();

        let req = ShioriRequest::parse(grammar).unwrap_or_else(|e| panic!("{}", e));

        assert_eq!(req.version, 26);
        assert_eq!(req.charset.unwrap(), "UTF-8");
        assert_eq!(req.sender.unwrap(), "SSP");
        assert_eq!(req.method, Rule::get);
        assert_eq!(req.id.unwrap(), "Version");
        assert_eq!(req.security_level, None);
        assert_eq!(req.status, None);
        assert_eq!(req.base_id, None);

        assert_eq!(req.dic.len(), 2);
        assert_eq!(req.key_values.len(), 0);
        assert_eq!(req.reference.len(), 0);
    }
}
