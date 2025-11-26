use std::marker::PhantomPinned;

fn main() {
    move_unpin_type();
    move_not_unpin_type();
}

fn move_unpin_type() {
    let mut i = 42;
    let pinned = Box::pin(&mut i);
    drop(pinned);
    let _ = std::mem::replace(&mut i, 10);
    println!("{}", i);
}

struct UnpinType {
    value: String,
    _pin: PhantomPinned,
}

fn move_not_unpin_type() {
    let mut s = UnpinType {
        value: String::from("old value"),
        _pin: PhantomPinned,
    };

    let pinned = Box::pin(&mut s);
    println!("{}", pinned.as_ref().value);
    drop(pinned);

    let _ = std::mem::replace(
        &mut s,
        UnpinType {
            value: String::from("new value"),
            _pin: PhantomPinned,
        },
    );

    println!("{}", s.value);
}
