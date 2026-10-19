//! The mockups of the design of the window, in the gallery of `showcase`: a
//! page served by trunk, or a window on the desktop. `make mockups` serves
//! them.

mod mock;

fn main() {
    showcase::run(showcase::Options {
        title: "obusina mockups",
        canvas: "mockups",
        mocks: mock::all,
    });
}
