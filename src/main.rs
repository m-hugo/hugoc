use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
enum Expr<'a> {
    Num(usize),
    String(&'a str),
    Ident(&'a str),
    Exrange(&'a str),
    Op(String, Box<(Expr<'a>, Expr<'a>)>),
}
impl<'a> Expr<'a> {
    fn parse() -> impl Parser<
        'a,
        &'a str,
        Expr<'a>,
        chumsky::extra::Full<Rich<'a, char, SimpleSpan, &'a str>, (), ()>,
    > {
        let ident = any::<&str, extra::Err<Rich<char>>>()
            .filter(|c: &char| c.is_ascii_alphabetic())
            .repeated()
            .at_least(1)
            .map_slice(Expr::Ident)
            .boxed();
        let num = any::<&str, extra::Err<Rich<char>>>()
            .filter(|c: &char| c.is_digit(10))
            .repeated()
            .at_least(1)
            .collect::<String>()
            .from_str()
            .unwrapped()
            .map(Expr::Num)
            .boxed();
        let string = just('"')
            .then(any().filter(|c: &char| *c != '"').repeated())
            .then(just('"'))
            .map_slice(Expr::String)
            .boxed();
        let exrange = just("[1;100]").map_slice(Expr::Exrange).boxed();
        //let addlit = num.clone() .foldl(just('+').ignore_then(num).repeated(), |a, b| a + b);

        recursive::<_, _, extra::Err<Rich<'a, char, SimpleSpan, &'a str>>, _, _>(|other_expr| {
            let op = other_expr
                .clone()
                .then(one_of("@#$%^&*+-<>,;:").repeated().at_least(1).collect())
                .then(other_expr.clone() )
                .map(|((e1, s), e2): ((_, String /*&str*/), _)| Expr::Op(s, Box::new((e1, e2)))) ;
            choice((
                other_expr.delimited_by(just('('), just(')')) ,
                op,
                exrange.clone(),
                string,
                ident,
                num.clone(),
            )).memoised() //memoised memoized
            .boxed()
        })
        .padded()
        .boxed()
    }
}

#[derive(Clone, PartialEq, Debug)]
struct HugoIR<'a> {
    fnmap: HashMap<String, Expr<'a>>,
}
impl<'a> HugoIR<'a> {
    fn parse(
        src: &'a str,
    ) -> (
        Option<Self>,
        Vec<chumsky::error::Rich<'a, char, chumsky::span::SimpleSpan, &'a str>>,
    ) {
        let ident = any()
            .filter(|c: &char| c.is_alphabetic())
            .repeated()
            .at_least(1)
            .collect()
            .boxed();

        ident
            .padded()
            .then(Expr::parse())
            .repeated()
            .collect::<HashMap<String, _>>()
            .map(|i| Self { fnmap: i })
            .parse(src.trim())
            .into_output_errors()
    }
    fn optimise(&mut self) {}
    fn interpret(&self) {}
    fn output_x86_elf(&self) {}
}

fn main() {
    let src = std::fs::read_to_string("test.hl").expect("Failed to read source test.hl");
    let (mut r, errs) = HugoIR::parse(&src);

    errs.iter().for_each(|e /*: Rich<char>*/| {
        Report::build(ReportKind::Error, (), e.span().start)
            .with_message(e.to_string())
            .with_label(
                Label::new(e.span().into_range())
                    .with_message(e.reason().to_string())
                    .with_color(Color::RGB(255, 50, 127)),
            )
            .finish()
            .print(Source::from(&src))
            .unwrap()
    });

    if let Some(ref mut res) = r {
        println!("continuing compilation with: {res:#?}");
        res.optimise();
        println!("optimised form: {res:#?}");
        println!("Interpreter Started");
        res.interpret();
        println!("Interpretation Done");
    } else {
        println!("Fatal Error, Cannot  Continue");
    }

    drop(r); //prevents src from
    drop(errs); // being static
}
