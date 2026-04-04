use std::mem::ManuallyDrop;

use futures::executor;
use janetrs::{
    IsJanetAbstract, Janet, TaggedJanet, declare_janet_mod, janet_fn, jpanic,
    lowlevel::JanetAbstractType,
};

struct LibsqlDatabaseConnection {
    db: libsql::Database,
    conn: libsql::Connection,
}

unsafe impl IsJanetAbstract for LibsqlDatabaseConnection {
    type Get = ManuallyDrop<Self>;
    const SIZE: usize = size_of::<Self>();
    fn type_info() -> &'static janetrs::lowlevel::JanetAbstractType {
        let name_ptr = b"LibdqlDatabaseConnection\0".as_ptr();
        let output_ptr = name_ptr as *const i8;
        let abstract_type = JanetAbstractType {
            name: output_ptr,
            gc: None,
            gcmark: None,
            get: None,
            put: None,
            marshal: None,
            unmarshal: None,
            tostring: None,
            compare: None,
            hash: None,
            next: None,
            call: None,
            length: None,
            bytes: None,
            gcperthread: None,
        };
        let boxed = Box::new(abstract_type);
        Box::leak(boxed)
    }
}

#[janet_fn(arity(fix(1)))]
fn open_local_db(args: &mut [Janet]) -> Janet {
    let db_url = match args.get(0) {
        Some(item) => match item.unwrap() {
            TaggedJanet::String(url) => url.to_string(),
            _ => jpanic!("First argument must be a string"),
        },
        None => jpanic!("open-local-db takes one argument. Must be a valid url or :memory:"),
    };
    let db_fut = libsql::Builder::new_local(&db_url).build();
    match executor::block_on(db_fut) {
        Ok(db) => {
            let _db_msg = db.connect();
            let msg = format!("Database created at {}", &db_url);
            println!("{}", msg);
            Janet::nil()
        }
        Err(_) => jpanic!("Error while accessing database"),
    }
}

#[janet_fn(arity(fix(1)))]
fn open_sync_db(_args: &mut [Janet]) -> Janet {
    Janet::nil()
}

#[janet_fn(arity(fix(2)))]
fn open_remote_db(_args: &mut [Janet]) -> Janet {
    Janet::nil()
}

#[janet_fn(arity(fix(1)))]
fn close_db(_args: &mut [Janet]) -> Janet {
    Janet::nil()
}

#[janet_fn(arity(fix(3)))]
fn execute(_args: &mut [Janet]) -> Janet {
    Janet::nil()
}

#[janet_fn(arity(fix(3)))]
fn query(_args: &mut [Janet]) -> Janet {
    Janet::nil()
}

declare_janet_mod!("hydra";
    {"open-local-db", open_local_db},
    {"open-sync-db", open_sync_db},
    {"open-remote-db", open_remote_db},
    {"close-db", close_db},
    {"execute", execute},
    {"query", query}
);
