use std::mem::ManuallyDrop;

use futures::executor;
use janetrs::{
    IsJanetAbstract, Janet, JanetAbstract, TaggedJanet, declare_janet_mod, janet_fn, jpanic,
    lowlevel::JanetAbstractType,
};
use libsql::params;

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

fn handle_libsql_db_connection(janet_args: &mut [Janet], db_builder: libsql::Builder) -> Janet {
    /*
    let db_url = match janet_args.get(0) {
        Some(item) => match item.unwrap() {
            TaggedJanet::String(url) => url.to_string(),
            _ => jpanic!("First argument must be a string"),
        },
        None => jpanic!("open-local-db takes one argument. Must be a valid url or :memory:"),
    };
    let db_fut = w_local(&db_url).build();
    match executor::block_on(db_fut) {
        Ok(db) => {
            let conn = match db.connect() {
                Ok(db_conn) => db_conn,
                Err(_) => jpanic!("Failed to connect to database."),
            };
            let db_struct = LibsqlDatabaseConnection { db: db, conn: conn };
            let j_abstract = JanetAbstract::new(db_struct);
            Janet::j_abstract(j_abstract)
        }
        Err(_) => jpanic!("Error while accessing database"),
    }
    */
    Janet::nil()
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
            let conn = match db.connect() {
                Ok(db_conn) => db_conn,
                Err(_) => jpanic!("Failed to connect to database."),
            };
            let db_struct = LibsqlDatabaseConnection { db: db, conn: conn };
            let j_abstract = JanetAbstract::new(db_struct);
            Janet::j_abstract(j_abstract)
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
fn execute(args: &mut [Janet]) -> Janet {
    let db_struct: LibsqlDatabaseConnection = match args.get(0) {
        Some(j_object) => {
            let j_abstract: JanetAbstract = match j_object.unwrap() {
                TaggedJanet::Abstract(item) => item,
                _ => jpanic!("Expected first argument to be a janet type of Absract Janet"),
            };

            j_abstract.into_inner::<LibsqlDatabaseConnection>().unwrap()
        }
        None => jpanic!(
            "Expecting three arguments. First is a db reference, second is the query, third is an array of query variables."
        ),
    };

    let execute_query: String = match args.get(1) {
        Some(j_object) => {
            let j_string = match j_object.unwrap() {
                TaggedJanet::String(item) => item.to_string(),
                _ => jpanic!("Expected second argument to be a string"),
            };
            j_string
        }
        None => jpanic!(
            "Expecting three arguments. First is a db reference, second is the query, third is an array of query variables"
        ),
    };

    let parameter_array = match args.get(2) {
        Some(j_object) => {
            let j_array = match j_object.unwrap() {
                TaggedJanet::Array(arr) => arr,
                _ => jpanic!("Expected third argument to be an array"),
            };
            j_array
        }
        None => jpanic!(
            "Execpted three arguments, First is a db reference, second is the query, third is an array of parameters"
        ),
    };

    let exec_fut = db_struct.conn.execute(&execute_query, params!());
    match executor::block_on(exec_fut) {
        Ok(_) => Janet::nil(),
        Err(err) => {
            let msg = format!("ERROR -> {}", err.to_string());
            jpanic!("{}", msg)
        }
    }
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
