#[macro_use]
mod script;
mod mem;

fn main() {
    use script::*;

    // let expr = dbg!(expr!(
    //     (x y => {x, y}) {} {}
    // ));

    // let linked = dbg!(expr.link());

    // println!("{}", linked.eval());
}
