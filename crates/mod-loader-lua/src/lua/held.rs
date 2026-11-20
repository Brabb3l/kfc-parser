//! Lua values kept by Rust without an mlua reference each.
//!
//! An `mlua::Value` that names a string, table, function or userdata occupies one slot of mlua's
//! reference thread for as long as it lives. That thread is a Lua stack, and a Lua stack holds at
//! most `LUAI_MAXSTACK` (1,000,000) values; past it mlua panics ("cannot create a Lua reference,
//! out of auxiliary stack space"). The value wrappers of this crate cache every field and element a
//! mod looks at or assigns, and those caches live until the run ends, so a set of mods could touch
//! at most a million nodes between them.
//!
//! [`Held`] keeps such a value in ONE Lua table (the store) under an integer key instead. The store
//! is the only reference; a `Held` is that key. Numbers, booleans and nil are kept inline. The
//! value is strongly held until the `Held` is dropped, exactly as an `mlua::Value` would hold it.

use std::cell::{Cell, RefCell};

use mlua::Table;

use super::LuaValue;

/// Slots freed but still holding their old value, cleared (set to nil) in one go past this count.
const SWEEP_AT: usize = 4096;

thread_local! {
    static STORE: RefCell<Option<Table>> = const { RefCell::new(None) };
    static NEXT: Cell<i64> = const { Cell::new(1) };
    /// Freed slots whose old value is still in the store.
    static FREED: RefCell<Vec<i64>> = const { RefCell::new(Vec::new()) };
    /// Freed slots already set to nil.
    static CLEARED: RefCell<Vec<i64>> = const { RefCell::new(Vec::new()) };
}

/// Creates the store for this thread's Lua state. Call once, right after the state is created.
pub fn init(lua: &mlua::Lua) -> mlua::Result<()> {
    let table = lua.create_table()?;

    STORE.with(|s| s.replace(Some(table)));
    NEXT.with(|n| n.set(1));
    FREED.with(|f| f.borrow_mut().clear());
    CLEARED.with(|f| f.borrow_mut().clear());

    Ok(())
}

/// Lets go of the store. Call before the Lua state is closed.
pub fn shutdown() {
    STORE.with(|s| s.replace(None));
    FREED.with(|f| f.borrow_mut().clear());
    CLEARED.with(|f| f.borrow_mut().clear());
}

/// How many slots the store has handed out so far (for logging).
pub fn slots_used() -> i64 {
    NEXT.with(|n| n.get()) - 1
}

fn with_store<R>(f: impl FnOnce(&Table) -> R) -> R {
    STORE.with(|s| {
        let store = s.borrow();
        f(store.as_ref().expect("the held-value store is not initialised"))
    })
}

/// Sets every freed slot to nil, so the values they held can be collected.
fn sweep() {
    // taken out first: storing can run the collector, which can drop wrappers, which push to FREED
    let freed = FREED.with(|f| std::mem::take(&mut *f.borrow_mut()));

    if freed.is_empty() {
        return;
    }

    with_store(|store| {
        for id in &freed {
            let _ = store.raw_set(*id, LuaValue::Nil);
        }
    });

    CLEARED.with(|c| c.borrow_mut().extend(freed));
}

/// A Lua value kept alive by Rust. See the module documentation.
pub enum Held {
    Nil,
    Boolean(bool),
    Integer(mlua::Integer),
    Number(mlua::Number),
    /// A value in the store, under this key.
    Slot(i64),
}

impl Held {

    pub fn new(value: LuaValue) -> Self {
        match value {
            LuaValue::Nil => Held::Nil,
            LuaValue::Boolean(b) => Held::Boolean(b),
            LuaValue::Integer(i) => Held::Integer(i),
            LuaValue::Number(n) => Held::Number(n),
            other => {
                if FREED.with(|f| f.borrow().len()) >= SWEEP_AT {
                    sweep();
                }

                // no borrow is held while the store is written (see `sweep`)
                let id = FREED.with(|f| f.borrow_mut().pop())
                    .or_else(|| CLEARED.with(|c| c.borrow_mut().pop()))
                    .unwrap_or_else(|| NEXT.with(|n| {
                        let id = n.get();
                        n.set(id + 1);
                        id
                    }));

                with_store(|store| store.raw_set(id, other))
                    .expect("failed to store a held Lua value");

                Held::Slot(id)
            }
        }
    }

    pub fn get(&self) -> LuaValue {
        match self {
            Held::Nil => LuaValue::Nil,
            Held::Boolean(b) => LuaValue::Boolean(*b),
            Held::Integer(i) => LuaValue::Integer(*i),
            Held::Number(n) => LuaValue::Number(*n),
            Held::Slot(id) => with_store(|store| store.raw_get::<LuaValue>(*id))
                .expect("failed to read a held Lua value"),
        }
    }

}

impl From<LuaValue> for Held {
    fn from(value: LuaValue) -> Self {
        Held::new(value)
    }
}

impl Clone for Held {
    fn clone(&self) -> Self {
        match self {
            Held::Nil => Held::Nil,
            Held::Boolean(b) => Held::Boolean(*b),
            Held::Integer(i) => Held::Integer(*i),
            Held::Number(n) => Held::Number(*n),
            Held::Slot(_) => Held::new(self.get()),
        }
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        if let Held::Slot(id) = self {
            // Only recorded here: this can run inside Lua's collector (a wrapper being finalised),
            // where the store must not be written. The slot is reused or cleared later.
            let id = *id;
            let _ = FREED.try_with(|f| {
                if let Ok(mut f) = f.try_borrow_mut() {
                    f.push(id);
                }
            });
        }
    }
}

impl std::fmt::Debug for Held {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Held::Nil => write!(f, "Held::Nil"),
            Held::Boolean(b) => write!(f, "Held::Boolean({b})"),
            Held::Integer(i) => write!(f, "Held::Integer({i})"),
            Held::Number(n) => write!(f, "Held::Number({n})"),
            Held::Slot(id) => write!(f, "Held::Slot({id})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_reads_clones_and_frees() {
        let lua = mlua::Lua::new();
        init(&lua).unwrap();

        let a = Held::new(LuaValue::String(lua.create_string("hello").unwrap()));
        let b = Held::new(LuaValue::Integer(7));
        let t = lua.create_table().unwrap();
        t.set("x", 5).unwrap();
        let c = Held::new(LuaValue::Table(t));
        let c2 = c.clone();

        assert_eq!(a.get().to_string().unwrap(), "hello");
        assert!(matches!(b, Held::Integer(7)));
        lua.gc_collect().unwrap();
        lua.gc_collect().unwrap();

        // the table is alive only through the store, and the clone sees the same table
        match (c.get(), c2.get()) {
            (LuaValue::Table(x), LuaValue::Table(y)) => {
                assert_eq!(x.get::<i64>("x").unwrap(), 5);
                x.set("x", 6).unwrap();
                assert_eq!(y.get::<i64>("x").unwrap(), 6);
            }
            _ => panic!("expected tables"),
        }

        let used = slots_used();
        drop(c2);
        // a freed slot is reused
        let d = Held::new(LuaValue::String(lua.create_string("again").unwrap()));
        assert_eq!(slots_used(), used);
        assert_eq!(d.get().to_string().unwrap(), "again");

        shutdown();
    }

    /// The reason this module exists: far more than LUAI_MAXSTACK values kept at once.
    #[test]
    fn more_than_a_million() {
        let lua = mlua::Lua::new();
        init(&lua).unwrap();

        let mut kept = Vec::new();
        for i in 0..1_500_000i64 {
            let t = lua.create_table().unwrap();
            t.raw_set(1, i).unwrap();
            kept.push(Held::new(LuaValue::Table(t)));
        }

        for i in [0usize, 999_999, 1_000_000, 1_499_999] {
            match kept[i].get() {
                LuaValue::Table(t) => assert_eq!(t.raw_get::<i64>(1).unwrap(), i as i64),
                _ => panic!("expected a table"),
            }
        }

        // dropping frees the slots: after a sweep they are reused, the store does not grow
        kept.truncate(10);
        let before = slots_used();
        for i in 0..100_000i64 {
            let _ = Held::new(LuaValue::String(lua.create_string(i.to_string()).unwrap()));
        }
        assert_eq!(slots_used(), before);

        shutdown();
    }
}
