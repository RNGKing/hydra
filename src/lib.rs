use std::{fmt::Pointer, ops::Deref};

use futures::executor;
use janetrs::{
    IsJanetAbstract, Janet, JanetAbstract, JanetArray, JanetBuffer, JanetString, TaggedJanet,
    declare_janet_mod, janet_fn, jpanic, lowlevel::JanetAbstractType,
};
use libsql::params_from_iter;

enum LibsqlDatabaseConnection {
    DbOpen(libsql::Connection),
    DbClose,
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
            let db_struct = Box::new(LibsqlDatabaseConnection::DbOpen(connection));
            let j_abstract = JanetAbstract::new(db_struct);
            Janet::j_abstract(j_abstract)
        }
        Err(_) => jpanic!("Error while accessing database"),
    }
}

fn open_remote_db_internal(args: &mut [Janet]) -> Result<Janet, String> {
    let db_url: String = try_get_janet_string(args, 0)?;
    let db_access_token: String = try_get_janet_string(args, 1)?;

    let db_fut = libsql::Builder::new_remote(db_url, db_access_token).build();
    match executor::block_on(db_fut) {
        Ok(db) => {
            let connection = db.connect().unwrap();
            let db_struct = Box::new(LibsqlDatabaseConnection::DbOpen(connection));
            let j_abstract = JanetAbstract::new(db_struct);
            Ok(Janet::j_abstract(j_abstract))
        }
        Err(err) => {
            let err_msg = format!(
                "Failed to connect to database : Reason -> {}",
                err.to_string()
            );
            Err(err_msg)
        }
    }
}

#[janet_fn(arity(fix(2)))]
fn open_remote_db(args: &mut [Janet]) -> Janet {
    match open_remote_db_internal(args) {
        Ok(j_abstract) => j_abstract,
        Err(error) => jpanic!("{}", error),
    }
}

fn close_boxed_db(ptr: *mut LibsqlDatabaseConnection) {
    unsafe {
        (*ptr) = LibsqlDatabaseConnection::DbClose;
    }
}

fn close_db_internal(args: &mut [Janet]) -> Result<Janet, String> {
    let abstract_db = try_get_janet_abstract(args, 0)?;
    let libsql_db_conn: Box<LibsqlDatabaseConnection> =
        LibsqlDatabaseConnection::new_from_janet_abstract(abstract_db)?;
    let ptr = Box::leak(libsql_db_conn);
    close_boxed_db(ptr);
    Ok(Janet::nil())
}

#[janet_fn(arity(fix(1)))]
fn close_db(args: &mut [Janet]) -> Janet {
    match close_db_internal(args) {
        Ok(janet) => janet,
        Err(msg) => jpanic!("Error while closing database: {}", msg),
    }
}

fn handle_execute_internal(args: &mut [Janet]) -> Result<Janet, String> {
    let abstract_db = try_get_janet_abstract(args, 0)?;
    let query_string = try_get_janet_string(args, 1)?;
    let query_args = try_get_janet_array(args, 2)?;
    let arguments = query_args.iter().map(|item| match item.unwrap() {
        TaggedJanet::String(string_arg) => string_arg.to_string(),
        TaggedJanet::Number(num_arg) => num_arg.to_string(),
        _ => jpanic!("Must be a number or a string as an argument."),
    });

    let libsql_db_struct = LibsqlDatabaseConnection::new_from_janet_abstract(abstract_db)?;
    println!("EXECUTING");
    let output = match libsql_db_struct.as_ref() {
        LibsqlDatabaseConnection::DbOpen(conn) => {
            let exec_fut = conn.execute(&query_string, params_from_iter(arguments));

            match executor::block_on(exec_fut) {
                Ok(_) => Ok(Janet::nil()),
                Err(err) => {
                    let err_msg = format!("Error while executing query : {}", err.to_string());
                    Err(err_msg.to_string())
                }
            }
        }
        LibsqlDatabaseConnection::DbClose => Err("Database is in a closed state".to_string()),
    };

    Box::leak(libsql_db_struct);
    output
}

fn map_libsql_to_janet(input: libsql::Value) -> Janet {
    match input {
        libsql::Value::Integer(int_val) => Janet::int64(int_val),
        libsql::Value::Real(real_value) => Janet::number(real_value),
        libsql::Value::Text(text) => Janet::string(JanetString::new(text.as_bytes())),
        libsql::Value::Null => Janet::nil(),
        libsql::Value::Blob(data) => {
            let buffer = JanetBuffer::from_iter(data.iter());
            Janet::buffer(buffer)
        }
    }
}

fn handle_query_internal(args: &mut [Janet]) -> Result<Janet, String> {
    let abstract_db = try_get_janet_abstract(args, 0)?;
    let query_string = try_get_janet_string(args, 1)?;
    let query_args = try_get_janet_array(args, 2)?;
    let arguments = query_args.iter().map(|item| match item.unwrap() {
        TaggedJanet::String(string_arg) => string_arg.to_string(),
        TaggedJanet::Number(num_arg) => num_arg.to_string(),
        _ => jpanic!("Must be a number or a string as an argument."),
    });

    let libsql_db_struct = LibsqlDatabaseConnection::new_from_janet_abstract(abstract_db)?;

    let output = match libsql_db_struct.as_ref() {
        LibsqlDatabaseConnection::DbOpen(conn) => {
            let query_fut = conn.query(&query_string, params_from_iter(arguments));
            match executor::block_on(query_fut) {
                Ok(mut rows) => {
                    let mut output_list = JanetArray::new();
                    while let Ok(opt_row) = executor::block_on(rows.next()) {
                        if let Some(row) = opt_row {
                            let col_count = row.column_count();
                            let capacity: usize = col_count as usize;
                            let mut row_j_array = JanetArray::with_capacity(capacity);
                            for idx in 0..col_count {
                                if let Ok(val) = row.get_value(idx) {
                                    let mapped = map_libsql_to_janet(val);
                                    row_j_array.push(mapped);
                                }
                            }
                            output_list.push(Janet::array(row_j_array));
                        } else {
                            break;
                        }
                    }
                    Ok(Janet::array(output_list))
                }
                Err(err) => {
                    let err_msg = format!("Error while executing query : {}", err.to_string());
                    Err(err_msg)
                }
            }
        }
        LibsqlDatabaseConnection::DbClose => {
            Err("Cannot perform query on a closed connection.".to_string())
        }
    };
    Box::leak(libsql_db_struct);
    output
}

#[janet_fn(arity(fix(3)))]
fn execute(args: &mut [Janet]) -> Janet {
    // get the boxed value here ... then leak it after we're done with it
    match handle_execute_internal(args) {
        Ok(output) => output,
        Err(err) => jpanic!("{}", err),
    }
}

#[janet_fn(arity(fix(3)))]
fn query(args: &mut [Janet]) -> Janet {
    match handle_query_internal(args) {
        Ok(output) => output,
        Err(err) => jpanic!("{}", err),
    }
}

declare_janet_mod!("hydra";
    {"open-local-db", open_local_db},
    {"open-remote-db", open_remote_db},
    {"close-db", close_db},
    {"execute", execute},
    {"query", query}
);
