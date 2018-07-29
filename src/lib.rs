#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate sled;

// #include <jim.h>
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use sled::{ConfigBuilder, Tree};
use std::ffi::{CStr, CString};
use std::fs;
use std::mem;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::str;

/// Our main command. In Tcl, when call "sled" command, we are calling this
/// function. We expect to be called with "sled dbname /path/to/db". This function
/// sets a new command dbname on the interpreter, and associates it with a pointer
/// to a sled tree.
#[no_mangle]
pub unsafe extern "C" fn db_init(
    interp: *mut Jim_Interp,
    objc: c_int,
    objv: *const *mut Jim_Obj,
) -> c_int {
    if objc as i32 != 3 {
        println!("you must pass 2 arguments");
        return JIM_ERR as c_int;
    }
    let db_cmd_name = objv.offset(1);
    let path_ptr = objv.offset(2);

    // canonicalize path to give us an absolute path
    let path = fs::canonicalize(Path::new(
        CStr::from_ptr(get_string(&mut **path_ptr))
            .to_str()
            .unwrap(),
    )).expect("could not canonicalize path");

    let config = ConfigBuilder::new().path(path).build();
    let tree = Tree::start(config).expect("error loading sled database");
    let boxed: *mut Tree = Box::into_raw(Box::new(tree));

    // Make a void pointer for C
    let ttptr = std::mem::transmute::<*mut Tree, *mut c_void>(boxed);

    Jim_CreateCommand(
        interp,
        get_string(&mut **db_cmd_name),
        Some(database_cmd),
        ttptr,
        None,
    )
}

/// This function is the procedural implementation that that backs the database
/// command created by invocations of our `sled` command. For example, when
/// `sled db /some/path` is called, a db command is created, and calls to db
/// are routed to this function.
#[no_mangle]
pub unsafe extern "C" fn database_cmd(
    interp: *mut Jim_Interp,
    objc: c_int,
    objv: *const *mut Jim_Obj,
) -> c_int {
    let cmd_len = objc.clone();
    if cmd_len < 2 {
        println!("one of: close, put, get, scan");
        return JIM_ERR as c_int;
    }
    let mut args: Vec<*const *mut Jim_Obj> = Vec::new();
    for i in 0..objc as isize {
        args.push(objv.offset(i));
    }
    // 0 is our the db command, conventionally "db"; match first argument as the subcommand
    match CStr::from_ptr(get_string(&mut **args[1])).to_str().unwrap() {
        // db close
        "close" => {
            Jim_Free((*interp).cmdPrivData);
            let cmd_name = get_string(&mut **args[0]);
            Jim_DeleteCommand(interp, cmd_name);
            return JIM_OK as c_int;
        }
        // db del somekey
        "del" => {
            if cmd_len != 3 {
                println!("usage: db del key");
                return JIM_ERR as c_int;
            }
            let key = CStr::from_ptr(get_string(&mut **args[2])).to_bytes();
            let tree: &Tree = from_cmd_private_data(interp);
            tree.del(key).expect("error: del failed");
        }
        // db dump
        "dump" => {
            let tree: &Tree = from_cmd_private_data(interp);
            let mut iter = tree.scan(b"");
            while let Some(Ok((k, v))) = iter.next() {
                let key = String::from_utf8(k);
                let value = String::from_utf8(v);
                println!("key: {:?} value: {:?}", key, value);
            }
        }
        // db exist key
        "exist" => {
            if cmd_len != 3 {
                println!("exist takes one arg: key");
                return JIM_ERR as c_int;
            }
            let tree: &Tree = from_cmd_private_data(interp);
            let key = CStr::from_ptr(get_string(&mut **args[2])).to_bytes();
            if let Ok(Some(_)) = tree.get(key) {
                let s = CString::new("1").unwrap();
                Jim_SetResultFormatted(interp, s.as_ptr());
            } else {
                let s = CString::new("0").unwrap();
                Jim_SetResultFormatted(interp, s.as_ptr());
            }
        }
        // db get key; returns value
        "get" => {
            if cmd_len != 3 {
                println!("get takes one arg: key");
                return JIM_ERR as c_int;
            }
            let key = CStr::from_ptr(get_string(&mut **args[2])).to_bytes();
            let tree: &Tree = from_cmd_private_data(interp);
            if let Ok(Some(val)) = tree.get(key) {
                let s = CString::new(val).unwrap();
                Jim_SetResultFormatted(interp, s.as_ptr());
                return JIM_OK as c_int;
            }
        }
        // db put key value
        "put" => {
            if cmd_len != 4 {
                println!("put takes two args: key, value");
                return JIM_ERR as c_int;
            }
            let key = CStr::from_ptr(get_string(&mut **args[2])).to_bytes();
            let value = CStr::from_ptr(get_string(&mut **args[3])).to_bytes();
            let tree: &Tree = from_cmd_private_data(interp);
            if tree.set(key.to_vec(), value.to_vec()).is_err() {
                return JIM_ERR as c_int;
            };
        }
        // db scan key { k v } { puts $k $v }
        "scan" => {
            if cmd_len != 5 {
                // TODO: is there a better way to set err messages in Jim?
                println!("error: scan takes 3 args: prefix, key-val list, and {{ script }}");
                return JIM_ERR as c_int;
            }
            // db scan blah { k v } { puts $k $v }
            let key = CStr::from_ptr(get_string(&mut **args[2]));
            let prefix_matcher = key.clone().to_str().unwrap();
            // tempVar must be one of: a list, a string
            let kv_vars: Vec<&str> = CStr::from_ptr(get_string(&mut **args[3]))
                .to_str()
                .unwrap()
                .split_whitespace()
                .collect();

            if kv_vars.len() != 2 {
                println!(
                    "error: the 2nd argument to scan must be a 2-item list of the form {{ key value }}"
                );
                return JIM_ERR as c_int;
            }

            let script = CStr::from_ptr(get_string(&mut **args[4])).to_bytes();
            let tree: &Tree = from_cmd_private_data(interp);
            let mut iter = tree.scan(key.to_bytes());

            while let Some(Ok((k, v))) = iter.next() {
                // stop iterating
                if !str::from_utf8(&k).unwrap().starts_with(prefix_matcher) {
                    break;
                };
                // set stack vars k and v from db scan prefix { k v } { script... }
                // and then eval the script
                set_interp_var(interp, kv_vars[0], k);
                set_interp_var(interp, kv_vars[1], v);
                Jim_Eval(interp, script.as_ptr() as *const c_char);
            }
            // TODO need to remove stack vars here?
        }
        _ => {}
    }
    JIM_OK as c_int
}

/// Takes a reference to a T off of the cmdPrivData field of Jim_Interp.
/// See: https://doc.rust-lang.org/std/mem/fn.transmute.html#alternatives
fn from_cmd_private_data<'a, T>(interp: *mut Jim_Interp) -> &'a T {
    unsafe {
        if (*interp).cmdPrivData.is_null() {
            panic!("cmdPrivData is null");
        }
        // cannot move out of dereference of raw pointer
        let obj = &mut *((*interp).cmdPrivData as *mut T);
        return obj;
    }
}

unsafe fn set_interp_var(interp: *mut Jim_Interp, name: &str, value: Vec<u8>) {
    let name_obj = Jim_NewStringObj(interp, name.as_ptr() as *const c_char, name.len() as c_int);
    let value_len: c_int = value.len() as c_int;
    let valued = CString::new(value).expect("cannot make C string");
    let value_obj = Jim_NewStringObj(interp, valued.as_ptr() as *const c_char, value_len);
    Jim_SetVariable(interp, name_obj, value_obj);
}

/// A wrapper for Jim_GetString. We don't care about the length pointer, because
/// the CStr functions do not require it. The Jim_GetString implementation
/// lazily calls an internal function pointer on the Jim_Obj to rebuild its
/// string representation. If we were to use the null bytes, we would segfault.
fn get_string(jobj: &mut Jim_Obj) -> *const c_char {
    if !jobj.bytes.is_null() {
        return jobj.bytes;
    }
    let length: i32 = 0;
    unsafe { Jim_GetString(jobj as *mut Jim_Obj, length as *mut c_int) }
}

#[no_mangle]
pub fn Jim_sledInit(interp: *mut Jim_Interp) -> c_int {
    let cmdName = CString::new("sled").unwrap();
    let delProc: Jim_DelCmdProc = None;
    let mut privData: c_void = unsafe { mem::zeroed() };

    unsafe {
        Jim_CreateCommand(
            interp,
            cmdName.as_ptr(),
            Some(db_init),
            &mut privData,
            delProc,
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    // TODO tests:
    // get_string
    // from_cmd_private_data
    //

}
