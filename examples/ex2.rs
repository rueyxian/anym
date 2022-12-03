use anym::anym;

fn main() {
    // cargo expand --example ex2
    unit_struct();
    tuple_struct();
    c_struct();
}

fn unit_struct() {
    let _v = anym!();
    let _v = anym!(Foo);
}

fn tuple_struct() {
    let v = anym!((
        Plushie::Ferris,
        Bfg {
            deriv: "9000".to_string()
        }
    ));
    let _plush = v.0;
    let _bfg = &v.1;
    let _deriv = &v.1.deriv;

    let v = anym!(Foo(
        1.168,
        Bfg {
            deriv: "over 9000!".to_string()
        }
    ));
    let _huh = v.0;
    let _bfg = v.1;
}

fn c_struct() {
    let plush = Plushie::Gopher;
    let bfg = Bfg {
        deriv: "10K".to_string(),
    };
    let _v = anym!({
        epsilon: f64::EPSILON,
        n0: 42_usize,
        n1: 42_usize,
        n2: 42_usize,
        n3: 42_usize,
        n4: 42_usize,
        n5: 42_usize,
        plush,
        n6: 42_usize,
        n7: 42_usize,
        n8: 42_usize,
        n9: 42_usize,
        bfg,
    });

    let v = anym!(Coor {
        x: 42,
        y: 4896,
        z: 666
    });
    let _x = v.x;
    let _y = v.y;
    let _z = v.z;
}

enum Plushie {
    Gopher,
    Ferris,
}

struct Bfg {
    deriv: String,
}
