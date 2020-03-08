use log::debug;
use pest::{iterators::Pair, Parser};

use crate::risp::{
    error::RispError,
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
        Rule::num => Ok(val_num(parsed.as_str().parse::<i64>()?)),
        _ => unreachable!(),
    }
}

pub fn parse(s: &str) -> RispResult {
    let parsed = RispParser::parse(Rule::risp, s)?.next().unwrap();
    debug!("{}", parsed);
    let val_ptr = val_read(parsed)?;
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
    fn parsing_single_num() {
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
}