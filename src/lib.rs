#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate sled;

use sled::{ConfigBuilder, Iter, Tree};

// #include <jim.h>
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::{CStr, CString};
use std::mem;
use std::str;
use std::string::ToString;

use std::os::raw::{c_char, c_int, c_void};

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
    let db_cmd_name_ptr = objv.offset(1);
    let path_ptr = objv.offset(2);

    // bytes is a field defined thus, so we double dereference it
    // pub bytes: *mut ::std::os::raw::c_char,
    let path = CStr::from_ptr((**path_ptr).bytes).to_str().unwrap();

    println!("loading db at path {:?}", path);
    let config = ConfigBuilder::new().path(path).build();
    let tree = Tree::start(config).expect("error loading sled database");

    let boxed: *mut Tree = Box::into_raw(Box::new(tree));
    let sz = std::mem::size_of::<*mut Tree>();

    // Use Jim's allocator, then write over it with our own heap pointer.
    // Is this what you're supposed to do?
    #[allow(unused_assignments)]
    let mut ttptr = Jim_Alloc(sz as c_int);
    ttptr = std::mem::transmute::<*mut Tree, *mut c_void>(boxed);

    Jim_CreateCommand(
        interp,
        (**db_cmd_name_ptr).bytes,
        Some(database_cmd),
        ttptr,
        None,
    )
}

struct Error;

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
    let usage = "one of: close, put, get, scan";
    if cmd_len < 2 {
        println!("{}", usage);
        return JIM_ERR as c_int;
    }
    let mut v: Vec<*const *mut Jim_Obj> = Vec::new();
    for i in 0..objc as isize {
        v.push(objv.offset(i));
    }
    // 0 is our own command; match our first argument: 1
    match CStr::from_ptr((**v[1]).bytes).to_str().unwrap() {
        "close" => {
            Jim_Free((*interp).cmdPrivData);
            let cmd_name = (**v[0]).bytes;
            Jim_DeleteCommand(interp, cmd_name);
            return JIM_OK as c_int;
        }
        "dump" => {
            let tree = &mut *((*interp).cmdPrivData as *mut Tree);
            let mut iter = tree.scan(b"");
            while let Some(Ok((k, v))) = iter.next() {
                //let key = CString::from_vec_unchecked(k);
                //let value = CString::from_vec_unchecked(v);
                let key = String::from_utf8(k);
                let value = String::from_utf8(v);
                // let key = CString::from_vec_unchecked(k);
                // let value = CString::from_vec_unchecked(v);
                println!("key: {:?} value: {:?}", key, value);
            }
        }
        "put" => {
            if cmd_len != 4 {
                println!("put takes two args: key, value");
                return JIM_ERR as c_int;
            }

            let key = CStr::from_ptr(get_string(&mut **v[2])).to_bytes();
            let value = CStr::from_ptr(get_string(&mut **v[3])).to_bytes();

            // Note the outer parens here. We cast *mut c_void to *mut Tree, and
            // reborrow to &mut Tree, a regular reference. See:
            // https://doc.rust-lang.org/std/mem/fn.transmute.html#alternatives
            let tree = &mut *((*interp).cmdPrivData as *mut Tree);
            if tree.set(key.to_vec(), value.to_vec()).is_err() {
                return JIM_ERR as c_int;
            };
        }
        "get" => {
            if cmd_len != 3 {
                println!("get takes one arg: key");
                return JIM_ERR as c_int;
            }
            let key = CStr::from_ptr((**v[2]).bytes).to_bytes();
            let tree = &mut *((*interp).cmdPrivData as *mut Tree);
            if let Ok(Some(val)) = tree.get(key) {
                let s = CString::new(val).unwrap();
                Jim_SetResultFormatted(interp, s.as_ptr());
                return JIM_OK as c_int;
            }
        }
        "scan" => {
            if cmd_len != 5 {
                // TODO: is there a better way to set err messages in Jim?
                println!("scan takes 3 args: prefix, tempVar, and {{ script... }}");
                return JIM_ERR as c_int;
            }
            // db scan blah { k v } { puts $k $v }
            let key = CStr::from_ptr(get_string(&mut **v[2]));
            let prefix_matcher = key.clone().to_str().unwrap();
            // tempVar must be one of: a list, a string
            let tempVar = CStr::from_ptr((**v[3]).bytes).to_bytes();
            let script = CStr::from_ptr((**v[4]).bytes).to_bytes();
            let script_obj = Jim_NewStringObj(
                interp,
                script.as_ptr() as *const c_char,
                script.len() as c_int,
            );
            let tree = &mut *((*interp).cmdPrivData as *mut Tree);
            let mut iter = tree.scan(key.to_bytes());

            // When pulling values OUT of the database, we cannot assume they're null-term,
            // so we must use CString::new(vv), which handles this for us.
            while let Some(Ok((k, vv))) = iter.next() {
                // stop iterating
                if !str::from_utf8(&k).unwrap().starts_with(prefix_matcher) {
                    break;
                };
                // set stack var varName from db scan $prefix varName { ...code...}
                // TODO turn script into Obj
                let cloned = tempVar.clone();
                let name_obj = Jim_NewStringObj(
                    interp,
                    cloned.as_ptr() as *const c_char,
                    cloned.len() as c_int,
                );

                // we don't have null terminator so we need to add it here or nah?
                let value_len: c_int = vv.len() as c_int; //  + 1;
                let valued = CString::new(vv).expect("cannot make C string");
                let cloned_tempVar = tempVar.clone();
                let value_obj =
                    Jim_NewStringObj(interp, valued.as_ptr() as *const c_char, value_len);

                Jim_SetVariable(interp, name_obj, value_obj);
                Jim_Eval(interp, script.as_ptr() as *const c_char);
            }
        }
        _ => {}
    }
    JIM_OK as c_int
}

/// A wrapper for Jim_GetString. We don't care about the length pointer, because
/// the CStr functions check for proper formatting already. We need to use
/// Jim_GetString because the bytes field of a Jim_Obj may be in an invalid
/// state (e.g. NULL), and the Jim_GetString implementation lazily calls
/// an internal function pointer on Jim_Obj to rebuild the string representation.
/// If use the null bytes, we segfault.
fn get_string(jobj: &mut Jim_Obj) -> *const c_char {
    if !jobj.bytes.is_null() {
        return jobj.bytes;
    }
    let length: i32 = 0;
    unsafe { Jim_GetString(jobj as *mut Jim_Obj, length as *mut c_int) }
}

enum SledOp {
    Put,
    Get,
    Scan,
    Close,
    Unknown,
}

enum ObjType {
    List,
    OneString,
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
}
