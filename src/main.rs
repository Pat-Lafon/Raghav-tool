use egg::{rewrite as rw, *};
use ordered_float::NotNan;
use std::ops::Index;

pub type EGraph = egg::EGraph<Math, NoAnalysis>;
pub type EClass = egg::EClass<Math, NoAnalysis>;
pub type Rewrite = egg::Rewrite<Math, NoAnalysis>;

pub type Constant = NotNan<f64>;

define_language! {
    pub enum Math {
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),

        Constant(Constant),
        Symbol(Symbol),
    }
}

#[derive(Default)]
pub struct NoAnalysis;

impl Analysis<Math> for NoAnalysis {
    type Data = ();

    fn make(egraph: &EGraph, enode: &Math) -> Self::Data {
        ()
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        DidMerge(false, false)
    }

    fn modify(egraph: &mut EGraph, id: Id) {
        ()
    }
}

#[rustfmt::skip]
pub fn rules() -> Vec<Rewrite> { vec![
    rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
    rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
    rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
    rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),

    rw!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),

    rw!("zero-add"; "(+ ?a 0)" => "?a"),
    rw!("zero-mul"; "(* ?a 0)" => "0"),
    rw!("one-mul";  "(* ?a 1)" => "?a"),

    rw!("add-zero"; "?a" => "(+ ?a 0)"),
    rw!("mul-one";  "?a" => "(* ?a 1)"),

    rw!("cancel-sub"; "(- ?a ?a)" => "0"),

    rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
    rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
]}

fn check_match(egraph: &EGraph, eclass: &Id, target_nodes: &egg::RecExpr<Math>, index: &Id) -> bool {
    let target_enode = target_nodes.index(*index);
    egraph.index(*eclass).nodes.iter().fold(false, |acc, e| {
        acc || match (target_enode, e) {
            (Math::Add(x1), Math::Add(x2))
            | (Math::Sub(x1), Math::Sub(x2))
            | (Math::Mul(x1), Math::Mul(x2))
            | (Math::Div(x1), Math::Div(x2)) => {
                x1.iter().zip(x2.iter()).fold(true, |acc2, (i1, i2)| {
                    acc2 && check_match(egraph, i2, target_nodes, i1)
                })
            }
            (Math::Constant(c1), Math::Constant(c2)) => c1 == c2,
            (Math::Symbol(s1), Math::Symbol(s2)) => {
                s2.as_str() == "??" || s1.as_str() == s2.as_str()
            }
            (_, _) => false,
        }
    })
}

fn main() {
/*     let source_expr: RecExpr<Math> = "(+ (* A B) (* A ??))".parse().unwrap();
    let target_expr: RecExpr<Math> = "(* A (+ B C))".parse().unwrap(); */

/*     let source_expr: RecExpr<Math> = "(+ (* D B) (* D ??))".parse().unwrap();
    let target_expr: RecExpr<Math> = "(* A (+ B C))".parse().unwrap(); */

    let source_expr: RecExpr<Math> = "(+ (* A B) (* C ??))".parse().unwrap();
    let target_expr: RecExpr<Math> = "(* A (+ B C))".parse().unwrap();

    let runner: Runner<Math, NoAnalysis> = Runner::default()
        .with_iter_limit(5)
        .with_expr(&source_expr)
        .run(&rules());

    /*
        // Do some extraction for fun
        let extractor = Extractor::new(graph, AstSize);
        let (best_cost, best) = extractor.find_best(runner.roots[0]);
        println!("{best_cost:?} {best}");
    */

    println!(
        "{:?}",
        check_match(
            &runner.egraph,
            &runner.roots[0],
            &target_expr,
            &Id::from(target_expr.as_ref().len() - 1)
        )
    )
}
