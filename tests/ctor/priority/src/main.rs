#![allow(unexpected_cfgs)]
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]
//! Edition 2024 test.

use ctor::ctor;

#[ctor(unsafe, priority = 1)]
unsafe fn priority_1() {
    println!("1");
}

#[ctor(unsafe, priority = 2)]
unsafe fn priority_b() {
    println!("2");
}

#[ctor(unsafe, priority = 3)]
unsafe fn priority_three() {
    println!("3");
}

#[ctor(unsafe, priority = 7)]
unsafe fn z_priority_7() {
    println!("7");
}

#[ctor(unsafe, priority = 4)]
unsafe fn priority_four() {
    println!("4");
}

#[ctor(unsafe, priority = late, anonymous)]
unsafe fn priority_late() {
    println!("late");
}

#[ctor(unsafe, priority = 10)]
unsafe fn priority_10() {
    println!("10");
}

#[ctor(unsafe, priority = 5)]
unsafe fn priority_five() {
    println!("5");
}

#[ctor(unsafe, priority = early, anonymous)]
unsafe fn priority_early() {
    println!("early");
}

#[ctor(unsafe, priority = 0)]
unsafe fn priority_zero() {
    println!("0");
}

#[ctor(unsafe, priority = late, anonymous)]
unsafe fn priority_late() {
    println!("late");
}

#[ctor(unsafe, priority = 6)]
unsafe fn a_priority_6() {
    println!("6");
}

#[ctor(unsafe, anonymous)]
unsafe fn priority_none() {
    println!("no priority");
}

#[ctor(unsafe, priority = 8)]
unsafe fn priority_eight() {
    println!("8");
}

#[ctor(unsafe, priority = 9)]
unsafe fn priority_nine() {
    println!("9");
}

#[ctor(unsafe, anonymous)]
unsafe fn priority_none() {
    println!("no priority");
}

#[ctor(unsafe, priority = early, anonymous)]
unsafe fn priority_early() {
    println!("early");
}

fn main() {
    println!("main");
}
