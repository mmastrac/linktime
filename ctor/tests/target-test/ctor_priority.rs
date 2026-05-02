use ctor::declarative::ctor;

ctor!(
    #[ctor(unsafe, priority = early)]
    fn early() {}
);

ctor!(
    #[ctor(unsafe, priority = 1)]
    fn priority1() {}
);

ctor!(
    #[ctor(unsafe, priority = 900)]
    fn priority900() {}
);

ctor!(
    #[ctor(unsafe, priority = late)]
    fn late() {}
);

ctor!(
    #[ctor(unsafe, naked)]
    fn naked() {}
);
