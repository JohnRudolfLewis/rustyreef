use log::debug;
use pest::{iterators::Pair, Parser};

use crate::risp::{
    error::RispError,
    result::{RispResult, Result},
    val::*,
};

use chrono::NaiveTime;

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("risp.pest");

#[derive(Parser)]
#[grammar = "risp/risp.pest"]
pub struct RispParser;

fn is_bracket_or_eoi(parsed: &Pair<Rule>) -> bool {
    if parsed.as_rule() == Rule::EOI {
        return true;
    }
    let c = parsed.as_str();
    c == "(" || c == ")"
}

// Read a rule with children into the given containing Val
fn read_to_val(mut v: &mut Val, parsed: Pair<Rule>) -> Result<()> {
    for child in parsed.into_inner() {
        if is_bracket_or_eoi(&child) {
            continue;
        }
        val_add(&mut v, &*val_read(child)?)?;
    }
    Ok(())
}

fn val_read(parsed: Pair<Rule>) -> RispResult {
    match parsed.as_rule() {
        Rule::risp => {
            let mut ret = val_risp();
            read_to_val(&mut ret, parsed)?;
            Ok(ret)
        }
        Rule::expr => val_read(parsed.into_inner().next().unwrap()),
        Rule::list => {
            let mut ret = val_list();
            read_to_val(&mut ret, parsed)?;
            Ok(ret)
        }
        Rule::num => {
            let s = parsed.as_str();

            let s_int = s.parse::<i64>();
            if s_int.is_ok() {
                return Ok(val_num(s_int?));
            }

            let s_f64 = s.parse::<f64>();
            if s_f64.is_ok() {
                return Ok(val_float(s_f64?));
            }

            return Err(RispError::NotANumber);
            //Ok(val_num(parsed.as_str().parse::<i64>()?))
        },
        Rule::operator => Ok(val_sym(parsed.as_str())),
        Rule::symbol => Ok(val_sym(parsed.as_str())),
        Rule::time => {
            let s = parsed.as_str();
            let t = NaiveTime::parse_from_str(s, "%H:%M:%S")?;
            Ok(val_time(t))
        }
        _ => unreachable!(),
    }
}

pub fn parse(s: &str) -> RispResult {
    let parsed = RispParser::parse(Rule::risp, s)?.next().unwrap();
    debug!("{}", parsed);
    let val_ptr = val_read(parsed)?;
    debug!("Parsed: {:?}", *val_ptr);
    Ok(val_ptr)
}

#[cfg(test)]
mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn assert_parse_risp(input: &str, expected: &str) {
        let parsed = match parse(input) {
            Ok(p) => format!("{:?}", p),
            Err(e) => return assert!(false, format!("Parse failed: {:?}", e))
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parsing_nonsense_results_in_error() {
        init();
        let parsed = match parse("/|garbage|/") {
            Ok(p) => return assert!(false, format!("Should not have parsed: {:?}", p)),
            Err(e) => {}
        };
    }

    #[test]
    fn parse_single_number() {
        init();
        assert_parse_risp("1", "Risp([Num(1)])");
    }

    #[test]
    fn parse_multiple_numbers() {
        init();
        assert_parse_risp("1 2 3", "Risp([Num(1), Num(2), Num(3)])");
    }

    #[test]
    fn parse_single_symbol() {
        init();
        let res = parse("a");
        assert!(res.is_ok(), "single symbol should parse");

        match *res.unwrap() {
            Val::Risp(children) => {
                assert_eq!(1, children.len(), "Should have had one child");
                let child = (&children[0]).clone();
                assert_eq!(val_sym("a"), child, "Should have been Val::Sym(a)");
            },
            _ => assert!(false, "should have been a Val::Risp")
        }
    }

    #[test]
    fn symbol_can_have_numbers() {
        init();
        let res = parse("a1");
        assert!(res.is_ok(), "single symbol with number should parse");

        match *res.unwrap() {
            Val::Risp(children) => {
                assert_eq!(1, children.len(), "Should have had one child");
                let child = (&children[0]).clone();
                assert_eq!(val_sym("a1"), child, "Should have been Val::Sym(a1)");
            },
            _ => assert!(false, "should have been a Val::Risp")
        }
    }

    #[test]
    fn symbol_can_not_start_with_a_number() {
        init();
        let res = parse("1a");
        assert!(res.is_err(), "symbol cant start with a number");
    }

    #[test]
    fn parse_multiple_symbols() {
        init();
        let res = parse("a b c");
        assert!(res.is_ok(), "list of symbols should parse");

        match *res.unwrap() {
            Val::Risp(children) => {
                assert_eq!(3, children.len(), "Should have had three children");
                assert_eq!(val_sym("a"), (&children[0]).clone(), "Should have been Val::Sym(a)");
                assert_eq!(val_sym("b"), (&children[1]).clone(), "Should have been Val::Sym(b)");
                assert_eq!(val_sym("c"), (&children[2]).clone(), "Should have been Val::Sym(c)");
            },
            _ => assert!(false, "should have been a Val::Risp")
        }
    }

    

    #[test]
    fn parse_single_char_operator() {
        init();
        assert_parse_risp("<", "Risp([Sym(\"<\")])");
    }

    #[test]
    fn parse_double_char_operator() {
        init();
        assert_parse_risp("<=", "Risp([Sym(\"<=\")])");
    }

    #[test]
    fn parse_list_of_numbers_and_symbols() {
        init();

        let res = match parse("(+ 1 a b)") {
            Ok(p) => *p,
            Err(err) => {
                debug!("{}", err);
                return assert!(false, err)
            }
        };

        let risp_children = match res {
            Val::Risp(children) => children,
            _ => return assert!(false, "should have been a Val::Risp")
        };
        assert_eq!(1, risp_children.len(), "Risp should have had one child");

        let list = match &*risp_children[0] {
            Val::List(l) => l,
            _ => return assert!(false, "Risp should have had a List as its one child")
        };
        assert_eq!(4, list.len(), "List should have had four children");

        assert_eq!(val_sym("+"), list[0], "First element in list should have been Val::Sym(a)");
        assert_eq!(val_num(1), list[1], "Second element in list should have been Val::Num(1)");
        assert_eq!(val_sym("a"), list[2], "Third element in list should have been Val::Sym(a)");
        assert_eq!(val_sym("b"), list[3], "Fourth element in list should have been Val::Sym(b)");
    }

    #[test]
    fn parse_single_float() {
        init();
        let res = parse("-3.1415");
        assert!(res.is_ok(), "single float should parse");

        match *res.unwrap() {
            Val::Risp(children) => {
                assert_eq!(1, children.len(), "Should have had one child");
                let child = (&children[0]).clone();
                assert_eq!(val_float(-3.1415), child, "Should have been Val::Num(1)");
            },
            _ => assert!(false, "should have been a Val::Risp")
        }
    }

    #[test]
    fn parse_time() {
        init();
        assert_parse_risp("00:00:00", "Risp([Time(00:00:00)])");
    }
    
}