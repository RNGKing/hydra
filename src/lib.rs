use std::{ffi::c_void, mem::ManuallyDrop};

use futures::executor;
use janetrs::{
    IsJanetAbstract, Janet, JanetAbstract, JanetArray, TaggedJanet, declare_janet_mod, janet_fn,
    jpanic, lowlevel::JanetAbstractType,
};
use libsql::{Database, params::IntoValue, params_from_iter};

struct JanetTypeToLibsqlValueWrapper(Janet);

impl IntoValue for JanetTypeToLibsqlValueWrapper {
    fn into_value(self) -> libsql::Result<libsql::Value> {
        match self.0.unwrap() {
            TaggedJanet::String(janet_string) => {
                let string_arg = janet_string.to_string();
                let out_value = libsql::Value::Text(string_arg);
                Ok(out_value)
            }
            TaggedJanet::Number(janet_number) => {
                let out_value = libsql::Value::Real(janet_number);
                Ok(out_value)
            }
            _ => jpanic!("Input must be a string, int or number"),
        }
    }
}

struct LibsqlDatabaseConnection {
    conn: libsql::Connection,
}

impl LibsqlDatabaseConnection {
    pub fn new_from_janet_abstract(
        input: JanetAbstract,
    ) -> Result<Box<LibsqlDatabaseConnection>, String> {
        match input.into_inner::<Box<LibsqlDatabaseConnection>>() {
            Ok(output) => Ok(output),
            Err(_) => Err(
                "Failed to convert Janet Abstract type into LibsqlDatabaseConnection".to_string(),
            ),
        }
    }
}

unsafe impl IsJanetAbstract for Box<LibsqlDatabaseConnection> {
    type Get = Self;
    const SIZE: usize = size_of::<Self>();
    fn type_info() -> &'static JanetAbstractType {
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

impl From<Option<Janet>> for Box<LibsqlDatabaseConnection> {
    fn from(value: Option<Janet>) -> Self {
        match value {
            Some(j_object) => {
                let j_abstract = match j_object.unwrap() {
                    TaggedJanet::Abstract(item) => item,
                    _ => jpanic!(
                        "A LibsqlDatabaseConnection can only be formed from a Abstract Janet type."
                    ),
                };
                let result = j_abstract.into_inner::<Box<LibsqlDatabaseConnection>>();
                if result.is_err() {
                    jpanic!("Coult not retrieve database connection from arguments")
                }
                result.unwrap()
            }
            None => {
                jpanic!("There is no argument present to convert into LibsqlDatabaseConnection")
            }
        }
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

fn try_get_janet_abstract(args: &mut [Janet], index: usize) -> Result<JanetAbstract, String> {
    match args.get(index) {
        Some(j_object) => match j_object.unwrap() {
            TaggedJanet::Abstract(item) => Ok(item),
            _ => {
                let err_msg_type = format!(
                    "Janet argument at {} is not the type of Janet Abstract",
                    index
                );
                Err(err_msg_type)
            }
        },
        None => {
            let err_msg = format!("No janet argument at index {}", index);
            Err(err_msg)
        }
    }
}

fn try_get_janet_string(args: &mut [Janet], index: usize) -> Result<String, String> {
    match args.get(index) {
        Some(j_object) => match j_object.unwrap() {
            TaggedJanet::String(j_string) => Ok(j_string.to_string()),
            _ => {
                let err_msg_type = format!(
                    "Janet argument at {} is not the type of Janet String",
                    index
                );
                Err(err_msg_type)
            }
        },
        None => {
            let err_msg = format!("No janet argument at index {}", index);
            Err(err_msg)
        }
    }
}

fn try_get_janet_array(args: &mut [Janet], index: usize) -> Result<JanetArray<'_>, String> {
    match args.get(index) {
        Some(j_object) => match j_object.unwrap() {
            TaggedJanet::Array(arr) => Ok(arr),
            _ => {
                let err_msg_type =
                    format!("Janet argument at {} is not the type of Janet Array", index);
                Err(err_msg_type)
            }
        },
        None => {
            let err_msg = format!("No janet argument at index {}", index);
            Err(err_msg)
        }
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
    let null_terminated = format!("{}", db_url);
    let db_fut = libsql::Builder::new_local(&null_terminated).build();
    match executor::block_on(db_fut) {
        Ok(db) => {
            let connection = db.connect().unwrap();
            let db_struct = Box::new(LibsqlDatabaseConnection { conn: connection });
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

fn handle_execute_internal(args: &mut [Janet]) -> Result<Janet, String> {
    let abstract_db = try_get_janet_abstract(args, 0)?;
    let query_string = try_get_janet_string(args, 1)?;
    let query_args = try_get_janet_array(args, 2)?;
    let libsql_db_struct = LibsqlDatabaseConnection::new_from_janet_abstract(abstract_db)?;
    let arguments = query_args.iter().map(|item| match item.unwrap() {
        TaggedJanet::String(string_arg) => string_arg.to_string(),
        TaggedJanet::Number(num_arg) => num_arg.to_string(),
        _ => jpanic!("WE SUCK"),
    });
    let exec_fut = libsql_db_struct
        .conn
        .execute(&query_string, params_from_iter(arguments));

    match executor::block_on(exec_fut) {
        Ok(_) => {
            Box::leak(libsql_db_struct);
            Ok(Janet::nil())
        }
        Err(err) => {
            Box::leak(libsql_db_struct);
            let err_msg = format!("Error while executing query : {}", err.to_string());
            Err(err_msg.to_string())
        }
    }
}

#[janet_fn(arity(fix(3)))]
fn execute(args: &mut [Janet]) -> Janet {
    match handle_execute_internal(args) {
        Ok(output) => output,
        Err(err) => jpanic!("{}", err),
    }
    /*
    let db try_get_janet_abstract(args, 0);


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

    let parameter_array : Array = match args.get(2) {
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

    let arguments = parameter_array.iter().map(|item| match item.unwrap() {
        TaggedJanet::String(string_arg) => string_arg.to_string(),
        TaggedJanet::Number(num_arg) => num_arg.to_string(),
        _ => jpanic!("WE SUCK"),
    });
    println!("FROM RUST BEFORE WE TRY TO EXECUTE {}", &execute_query);

    let exec_fut = db_struct
        .conn
        .execute(&execute_query, params_from_iter(arguments));

    let result = match executor::block_on(exec_fut) {
        Ok(_) => Janet::nil(),
        Err(err) => {
            let msg = format!("ERROR -> {}", err.to_string());
            jpanic!("{}", msg)
        }
    };
    Box::leak(db_struct);
    result
    */
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
