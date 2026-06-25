//! Example for `ScatteredMap`: registering named commands at link-time.
use scattered_collect::{ScatteredMap, gather, scatter};

trait Command: Send + Sync {
    fn execute(&self);
}

#[gather]
static COMMANDS: ScatteredMap<&'static str, &'static dyn Command>;

/// Macro for registering commands at link-time.
#[macro_export]
macro_rules! register_commands {
    ($($command:ident: $type:ident),* $(,)?) => {
        $(
            #[allow(non_upper_case_globals)]
            #[scatter(COMMANDS)]
            static $command: (&'static str, $type) = (stringify!($command), &$type);
        )*
    };
}

struct Command1;
impl Command for Command1 {
    fn execute(&self) {
        println!("Command1");
    }
}

struct Command2;
impl Command for Command2 {
    fn execute(&self) {
        println!("Command2");
    }
}

register_commands!(
    command_1: Command1,
    command_2: Command2,
);

fn main() {
    for command in ["command_1", "command_2"] {
        let command = COMMANDS.get(command).unwrap();
        command.execute();
    }
}
