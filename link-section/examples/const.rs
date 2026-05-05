//! Example usage of the `link-section` crate.
#![cfg_attr(linktime_used_linker, feature(used_with_arg))]

use link_section::{in_section, section};

struct Driver {
    name: &'static str,
    f: fn(),
}

impl Driver {
    /// Create a new driver.
    const fn new(name: &'static str, f: fn()) -> Self {
        Self { name, f }
    }
}

/// Drivers.
#[section]
static DATA_SECTION: link_section::TypedSection<Driver>;

/// A driver for PostgreSQL.
#[in_section(DATA_SECTION)]
pub const POSTGRES_DRIVER: Driver = Driver::new("postgres", || println!("connected to postgres!"));

/// A driver for MySQL.
#[in_section(DATA_SECTION)]
pub const MYSQL_DRIVER: Driver = Driver::new("mysql", || println!("connected to mysql!"));

/// A driver for SQLite.
#[in_section(DATA_SECTION)]
pub const SQLITE_DRIVER: Driver = Driver::new("sqlite", || println!("connected to sqlite!"));

/// Databases.
#[section]
static DATABASES: link_section::TypedSection<(&'static str, &'static Driver)>;

/// A database for PostgreSQL.
#[in_section(DATABASES)]
pub const POSTGRES_DATABASE: (&'static str, &'static Driver) =
    ("postgres://localhost:5432", &POSTGRES_DRIVER);

/// A database for MySQL.
#[in_section(DATABASES)]
pub const MYSQL_DATABASE: (&'static str, &'static Driver) =
    ("mysql://localhost:3306", &MYSQL_DRIVER);

/// A database for SQLite.
#[in_section(DATABASES)]
pub const SQLITE_DATABASE: (&'static str, &'static Driver) =
    ("sqlite://localhost:1433", &SQLITE_DRIVER);

fn main() {
    for (url, driver) in DATABASES {
        println!("Connecting to {url} ({})...", driver.name);
        (driver.f)();
    }
}
