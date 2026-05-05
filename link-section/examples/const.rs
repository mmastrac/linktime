//! Example usage of the `link-section` crate.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

use link_section::{in_section, section};

struct Driver {
    name: &'static str,
    f: fn (),
}

impl Driver {
    pub const fn new(name: &'static str, f: fn ()) -> Self {
        Self { name, f }
    }
}

#[section]
static DATA_SECTION: link_section::TypedSection<Driver>;

#[in_section(DATA_SECTION)]
pub const POSTGRES_DRIVER: Driver = Driver::new("postgres", || println!("postgres"));

#[in_section(DATA_SECTION)]
pub const MYSQL_DRIVER: Driver = Driver::new("mysql", || println!("mysql"));

#[in_section(DATA_SECTION)]
pub const SQLITE_DRIVER: Driver = Driver::new("sqlite", || println!("sqlite"));

fn main() {
    for driver in DATA_SECTION {
        println!("Connecting to {}...", driver.name);
        (driver.f)();
    }
}
