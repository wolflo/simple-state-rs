#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use ethtest::ethtest;

pub struct State1;

#[test]
pub fn tests() {
    #[ethtest]
    pub async fn foo(ctx: State1) {}
}
