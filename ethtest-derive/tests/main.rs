#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use ethtest::{ethtest, ethstate};

pub struct State0;
pub struct State1;
pub trait State { type Prev: State; }
impl State for State0 { type Prev = State0; }


#[test]
pub fn tests() {
    // #[ethtest]
    // pub async fn foo(ctx: State1) {}
    #[ethstate(init)]
    impl State for State1 { type Prev = State0; }
}
