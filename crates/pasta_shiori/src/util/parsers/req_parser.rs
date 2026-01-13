#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("req_parser.pest");

use pest_derive::*;

#[allow(dead_code)]
#[derive(Parser)]
#[grammar_inline = r#"
req         = ${ SOI ~ header ~ key_values ~ _eol ~ EOI }
key_values  = ${ key_value* }

key_value   = ${ key ~ _tag ~ value ~ _eol }
value       = @{ remain }
key         = _{      ( key_ref ~ !_id2 )
                    | ( key_charset ~ !_id2 )
                    | ( key_id ~ !_id2 )
                    | ( key_base_id ~ !_id2 )
                    | ( key_status ~ !_id2 )
                    | ( key_security_level ~ !_id2 )
                    | ( key_sender ~ !_id2 )
                    | ( key_other )
                }
key_ref     = ${ "Reference" ~ nums }
key_charset = @{ "Charset" }
key_id      = @{ "ID" }
key_base_id = @{ "BaseID" }
key_status  = @{ "Status" }
key_security_level = @{ "SecurityLevel" }
key_sender  = @{ "Sender" }
key_other   = @{ id }

header      = ${ method ~ _sp ~ ( header3 | header2 ) ~ _eol }

method      = ${ get | notify }
header2     = ${ shiori2_id ~ _sp ~ _shiori2 ~ shiori2_ver }
shiori2_id  = @{ id }
shiori2_ver = @{ ver }
header3     = ${ _shiori3 }

id          = @{ XID_START ~ _id2* }
_id2        = _{ XID_CONTINUE | _key_sep }

_shiori2    = _{ "SHIORI/2." }
_shiori3    = _{ "SHIORI/3.0" }
nums        = @{ ( ('1'..'9') ~ ('0'..'9')* ) | "0" }
ver         = @{ ASCII_DIGIT }
get         =  { "GET" }
notify      =  { "NOTIFY" }
_tag        = _{ ": " }
remain      = @{ ( !"\r" ~ !"\n" ~ ANY )* }

sp          = @{ _sp }
eol         = @{ _eol }
key_sep     = @{ _key_sep }
_sp         = _{ " " }
_eol        = _{ "\r\n" | "\n" | "\r" }
_key_sep    = _{ "-" | "." }
"#]
pub struct ShioriRequestParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn id_1() {
        let items = ShioriRequestParser::parse(Rule::id, "a1")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        {
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::id);
            let span = pair.clone().as_span();
            assert_eq!(span.as_str(), "a1");
            assert_eq!(span.start(), 0);
            assert_eq!(span.end(), 2);
        }
    }

    #[test]
    fn id_2() {
        let items = ShioriRequestParser::parse(Rule::id, "感じの良いID")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        {
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::id);
            let span = pair.clone().as_span();
            assert_eq!(span.as_str(), "感じの良いID");
            assert_eq!(span.start(), 0);
            assert_eq!(span.end(), 17);
        }
    }

    #[test]
    fn id_3() {
        let mut it = ShioriRequestParser::parse(Rule::id, "Ref-123.a23")
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        let pair = it.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::id);
        assert_eq!(pair.as_str(), "Ref-123.a23");

        assert_eq!(it.next(), None);
    }

    #[test]
    fn remain_1() {
        let mut items = ShioriRequestParser::parse(Rule::remain, "ABC\r\n")
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::remain);
        assert_eq!(pair.as_str(), "ABC");

        assert_eq!(items.next(), None);
    }

    #[test]
    fn remain_2() {
        let mut items = ShioriRequestParser::parse(Rule::remain, "ABC\rABCD\r\n")
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::remain);
        assert_eq!(pair.as_str(), "ABC");

        assert_eq!(items.next(), None);
    }

    #[test]
    fn method_1() {
        let items = ShioriRequestParser::parse(Rule::method, "GET")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        {
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::method);
            let span = pair.clone().as_span();
            assert_eq!(span.as_str(), "GET");
            assert_eq!(span.start(), 0);
            assert_eq!(span.end(), 3);
            let items = pair.clone().into_inner().collect::<Vec<_>>();
            assert_eq!(items.len(), 1);
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::get);
        }
    }

    #[test]
    fn method_2() {
        let items = ShioriRequestParser::parse(Rule::method, "NOTIFY")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        {
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::method);
            let span = pair.clone().as_span();
            assert_eq!(span.as_str(), "NOTIFY");
            assert_eq!(span.start(), 0);
            assert_eq!(span.end(), 6);
            let items = pair.clone().into_inner().collect::<Vec<_>>();
            assert_eq!(items.len(), 1);
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::notify);
        }
    }

    #[test]
    fn header_1() {
        let items = ShioriRequestParser::parse(Rule::header, "GET OPEN SHIORI/2.6\r\n")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        {
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::header);
            let span = pair.clone().as_span();
            assert_eq!(span.as_str(), "GET OPEN SHIORI/2.6\r\n");
            assert_eq!(span.start(), 0);
            assert_eq!(span.end(), 21);
            let items = pair.clone().into_inner().collect::<Vec<_>>();
            assert_eq!(items.len(), 2);
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::method);
            {
                let items = pair.clone().into_inner().collect::<Vec<_>>();
                assert_eq!(items.len(), 1);
                let pair = &items[0];
                assert_eq!(pair.as_rule(), Rule::get);
            }
            let pair = &items[1];
            assert_eq!(pair.as_rule(), Rule::header2);
            {
                let items = pair.clone().into_inner().collect::<Vec<_>>();
                assert_eq!(items.len(), 2);
                let pair = &items[0];
                assert_eq!(pair.as_rule(), Rule::shiori2_id);
                assert_eq!(pair.as_str(), "OPEN");
                let pair = &items[1];
                assert_eq!(pair.as_rule(), Rule::shiori2_ver);
                assert_eq!(pair.as_str(), "6");
            }
        }
    }

    #[test]
    fn header_2() {
        let items = ShioriRequestParser::parse(Rule::header, "NOTIFY SHIORI/3.0\r\n")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        {
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::header);
            let span = pair.clone().as_span();
            assert_eq!(span.as_str(), "NOTIFY SHIORI/3.0\r\n");
            assert_eq!(span.start(), 0);
            assert_eq!(span.end(), 19);
            let items = pair.clone().into_inner().collect::<Vec<_>>();
            assert_eq!(items.len(), 2);
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::method);
            {
                let items = pair.clone().into_inner().collect::<Vec<_>>();
                assert_eq!(items.len(), 1);
                let pair = &items[0];
                assert_eq!(pair.as_rule(), Rule::notify);
            }
            let pair = &items[1];
            assert_eq!(pair.as_rule(), Rule::header3);
        }
    }

    #[test]
    fn key_1() {
        let items = ShioriRequestParser::parse(Rule::key, "感じの良いID")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        {
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::key_other);
            let span = pair.clone().as_span();
            assert_eq!(span.as_str(), "感じの良いID");
            assert_eq!(span.start(), 0);
            assert_eq!(span.end(), 17);
        }
    }

    #[test]
    fn key_2() {
        let items = ShioriRequestParser::parse(Rule::key, "IDの感じ")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        {
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::key_other);
            let span = pair.clone().as_span();
            assert_eq!(span.as_str(), "IDの感じ");
            assert_eq!(span.start(), 0);
            assert_eq!(span.end(), 11);
        }
    }

    #[test]
    fn key_3() {
        let mut it = ShioriRequestParser::parse(Rule::key, "Reference123:")
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        let pair = it.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::key_ref);
        assert_eq!(pair.as_str(), "Reference123");

        let pair = it.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::nums);
        assert_eq!(pair.as_str(), "123");

        assert_eq!(it.next(), None);
    }

    #[test]
    fn key_4() {
        let mut it = ShioriRequestParser::parse(Rule::key, "Reference123の感じ")
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        let pair = it.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::key_other);
        assert_eq!(pair.as_str(), "Reference123の感じ");

        assert_eq!(it.next(), None);
    }

    #[test]
    fn key_value_1() {
        let items = ShioriRequestParser::parse(Rule::key_value, "IDは: XYZですよ\r\n")
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);

        let pair = &items[0];
        assert_eq!(pair.as_rule(), Rule::key_value);
        let span = pair.clone().as_span();
        assert_eq!(span.as_str(), "IDは: XYZですよ\r\n");
        assert_eq!(span.start(), 0);
        assert_eq!(span.end(), 21);

        let items = pair.clone().into_inner().collect::<Vec<_>>();
        assert_eq!(items.len(), 2);

        let pair = &items[0];
        assert_eq!(pair.as_rule(), Rule::key_other);
        assert_eq!(pair.as_str(), "IDは");

        let pair = &items[1];
        assert_eq!(pair.as_rule(), Rule::value);
        assert_eq!(pair.as_str(), "XYZですよ");
    }

    #[test]
    fn req_1() {
        let src = include_str!("test_data/shiori3-1.txt")
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n", "\r\n");
        let grammar = src.as_str();
        let items = ShioriRequestParser::parse(Rule::req, grammar)
            .unwrap_or_else(|e| panic!("{}", e))
            .collect::<Vec<_>>();
        assert_eq!(items.len(), 1);

        let pair = &items[0];
        assert_eq!(pair.as_rule(), Rule::req);
        let span = pair.clone().as_span();
        assert_eq!(span.as_str(), grammar);
        assert_eq!(span.start(), 0);
        assert_eq!(span.end(), 82);

        let items = pair.clone().into_inner().collect::<Vec<_>>();
        assert_eq!(items.len(), 3);

        let pair = &items[0];
        assert_eq!(pair.as_rule(), Rule::header);
        assert_eq!(pair.as_str(), "GET SHIORI/3.0\r\n");

        let pair = &items[1];
        assert_eq!(pair.as_rule(), Rule::key_values);
        {
            let items = pair.clone().into_inner().collect::<Vec<_>>();
            assert_eq!(items.len(), 4);
            let pair = &items[0];
            assert_eq!(pair.as_rule(), Rule::key_value);
            assert_eq!(pair.as_str(), "Charset: UTF-8\r\n");
            let pair = &items[1];
            assert_eq!(pair.as_rule(), Rule::key_value);
            assert_eq!(pair.as_str(), "ID: version\r\n");
            let pair = &items[2];
            assert_eq!(pair.as_rule(), Rule::key_value);
            assert_eq!(pair.as_str(), "SecurityLevel: local\r\n");
            let pair = &items[3];
            assert_eq!(pair.as_rule(), Rule::key_value);
            assert_eq!(pair.as_str(), "Sender: SSP\r\n");
        }

        let pair = &items[2];
        assert_eq!(pair.as_rule(), Rule::EOI);
    }

    #[test]
    fn req_2() {
        let src = include_str!("test_data/shiori3-2.txt")
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n", "\r\n");
        let grammar = src.as_str();
        let mut items = ShioriRequestParser::parse(Rule::req, grammar)
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::req);
        let span = pair.clone().as_span();
        assert_eq!(span.as_str(), grammar);
        assert_eq!(span.start(), 0);
        assert_eq!(span.end(), 133);

        assert_eq!(items.next().unwrap().as_rule(), Rule::header);
        assert_eq!(items.next().unwrap().as_rule(), Rule::method);
        assert_eq!(items.next().unwrap().as_rule(), Rule::notify);
        assert_eq!(items.next().unwrap().as_rule(), Rule::header3);
        assert_eq!(items.next().unwrap().as_rule(), Rule::key_values);

        assert_eq!(items.next().unwrap().as_rule(), Rule::key_value);
        assert_eq!(items.next().unwrap().as_rule(), Rule::key_charset);
        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::value);
        assert_eq!(pair.as_str(), "UTF-8");

        assert_eq!(items.next().unwrap().as_rule(), Rule::key_value);
        assert_eq!(items.next().unwrap().as_rule(), Rule::key_sender);
        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::value);
        assert_eq!(pair.as_str(), "SSP");

        assert_eq!(items.next().unwrap().as_rule(), Rule::key_value);
        assert_eq!(items.next().unwrap().as_rule(), Rule::key_security_level);
        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::value);
        assert_eq!(pair.as_str(), "local");

        assert_eq!(items.next().unwrap().as_rule(), Rule::key_value);
        assert_eq!(items.next().unwrap().as_rule(), Rule::key_id);
        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::value);
        assert_eq!(pair.as_str(), "ownerghostname");

        assert_eq!(items.next().unwrap().as_rule(), Rule::key_value);
        assert_eq!(items.next().unwrap().as_rule(), Rule::key_ref);
        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::nums);
        assert_eq!(pair.as_str(), "0");
        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::value);
        assert_eq!(pair.as_str(), "セキュリティボール");

        assert_eq!(items.next().unwrap().as_rule(), Rule::EOI);
        assert_eq!(items.next(), None);
    }

    #[test]
    fn req_3() {
        let src = include_str!("test_data/shiori2-1.txt")
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n", "\r\n");
        let grammar = src.as_str();
        let mut items = ShioriRequestParser::parse(Rule::req, grammar)
            .unwrap_or_else(|e| panic!("{}", e))
            .flatten();

        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::req);
        let span = pair.clone().as_span();
        assert_eq!(span.as_str(), grammar);

        assert_eq!(items.next().unwrap().as_rule(), Rule::header);
        assert_eq!(items.next().unwrap().as_rule(), Rule::method);
        assert_eq!(items.next().unwrap().as_rule(), Rule::get);
        assert_eq!(items.next().unwrap().as_rule(), Rule::header2);

        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::shiori2_id);
        assert_eq!(pair.as_str(), "Version");

        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::shiori2_ver);
        assert_eq!(pair.as_str(), "6");

        assert_eq!(items.next().unwrap().as_rule(), Rule::key_values);

        assert_eq!(items.next().unwrap().as_rule(), Rule::key_value);
        assert_eq!(items.next().unwrap().as_rule(), Rule::key_charset);
        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::value);
        assert_eq!(pair.as_str(), "UTF-8");

        assert_eq!(items.next().unwrap().as_rule(), Rule::key_value);
        assert_eq!(items.next().unwrap().as_rule(), Rule::key_sender);
        let pair = items.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::value);
        assert_eq!(pair.as_str(), "SSP");

        assert_eq!(items.next().unwrap().as_rule(), Rule::EOI);
        assert_eq!(items.next(), None);
    }

}
