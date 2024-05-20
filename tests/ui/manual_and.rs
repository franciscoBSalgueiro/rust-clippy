#![warn(clippy::manual_and)]
#[allow(irrefutable_let_patterns)]

fn main() {
    let a = true;
    let b = false;
    let c = true;

    let _ = if a { b } else { false };
    //~^ ERROR: this `if`-then-`else` expression can be simplified with `&&`

    let _ = if a && c { b } else { false };
    //~^ ERROR: this `if`-then-`else` expression can be simplified with `&&`

    let _ = if a || c { b } else { false };
    //~^ ERROR: this `if`-then-`else` expression can be simplified with `&&`

    // Should not lint

    // with if-let
    let _ = if let x = a { x } else { false };

    // when the then-branch is a block
    let _ = if let x = a {
        println!("hi");
        x
    } else {
        false
    };

    // when part of a chain of if-elses
    let _ = if a {
        b
    } else if b {
        a
    } else {
        false
    };
}
