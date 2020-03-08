use log::debug;
use pest::{iterators::Pair, Parser};

use crate::risp::{
    result::{RispResult, Result},
    val::*,
};

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
        Rule::num => Ok(val_num(parsed.as_str().parse::<i64>()?)),
        Rule::symbol => Ok(val_sym(parsed.as_str())),
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

    #[test]
    fn parsing_nonsense_results_in_error() {
        init();
        assert!(parse("/|garbage|/").is_err(), "garbage should not parse");
    }

    #[test]
    fn parse_single_number() {
        init();
        let res = parse("1");
        assert!(res.is_ok(), "single number should parse");

        match *res.unwrap() {
            Val::Risp(children) => {
                assert_eq!(1, children.len(), "Should have had one child");
                let child = (&children[0]).clone();
                assert_eq!(val_num(1), child, "Should have been Val::Num(1)");
            },
            _ => assert!(false, "should have been a Val::Risp")
        }
    }

    #[test]
    fn parse_multiple_numbers() {
        init();
        let res = parse("1 2 3");
        assert!(res.is_ok(), "list of numbers should parse");

        match *res.unwrap() {
            Val::Risp(children) => {
                assert_eq!(3, children.len(), "Should have had three children");
                assert_eq!(val_num(1), (&children[0]).clone(), "Should have been Val::Num(1)");
                assert_eq!(val_num(2), (&children[1]).clone(), "Should have been Val::Num(1)");
                assert_eq!(val_num(3), (&children[2]).clone(), "Should have been Val::Num(1)");
            },
            _ => assert!(false, "should have been a Val::Risp")
        }
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
}