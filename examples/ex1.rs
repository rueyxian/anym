use anym::anym;
fn main() {
    let v1 = {
        let (x, y) = coor();
        anym!({ tag: tag(), x, y })
    };

    let v2 = {
        let (x, y) = coor();
        anym!({ tag: tag(), x, y })
    };

    println!("{}: ({}, {})", v1.tag, v1.x, v1.y);
    println!("{}: ({}, {})", v2.tag, v2.x, v2.y);
}

fn coor() -> (u32, u32) {
    let gen = || {
        std::thread::sleep(std::time::Duration::from_millis(70));
        ((epoch() % 100000_u128) / 1000) as u32
    };
    (gen(), gen())
}

fn tag() -> String {
    let gen = || {
        std::thread::sleep(std::time::Duration::from_millis(70));
        ((epoch() % 25000_u128) / 1000) as u8 + 65_u8
    };
    String::from_utf8([gen(), gen(), gen()].to_vec()).unwrap()
}

fn epoch() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
