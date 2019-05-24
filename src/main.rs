#[macro_use]
mod script;

fn main() {
    use script::*;

    let expr = dbg!(expr!((|x| {x x}) {} {}));

    let linked = dbg!(expr.link());

    dbg!(linked.eval());
}
