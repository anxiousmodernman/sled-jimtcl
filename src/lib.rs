#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

extern crate sled;

use sled::{ConfigBuilder, Tree};

// #include <jim.h>
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::{CStr, CString};
use std::mem;

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
    let tree = Tree::start(config).unwrap();

    let boxed: *mut Tree = Box::into_raw(Box::new(tree));
    let sz = std::mem::size_of::<*mut Tree>();

    // Use Jim's allocator, then write over it with our own heap pointer.
    // Is this what you're supposed to do?
    #[allow(unused_assignments)]
    let mut ttptr = Jim_Alloc(sz as c_int);
    ttptr = std::mem::transmute::<*mut Tree, *mut c_void>(boxed);

    Jim_CreateCommand(interp, (**db_cmd_name_ptr).bytes, Some(database_cmd), ttptr, None)
}

/// This function is the procedural implementation that that backs the database
/// command created by invocations of our `sled` command. For example, when
/// `sled db /some/path` is called, a db command is created, and calls to db
/// are routed to this function.
pub unsafe extern "C" fn database_cmd(
    interp: *mut Jim_Interp,
    objc: c_int,
    objv: *const *mut Jim_Obj,
) -> c_int {
    let cmd_len = objc.clone();
    let usage = "one of: close, put, get, scan";
    if cmd_len < 2 {
        println!("{}", usage);
        return JIM_ERR as c_int
    }
    let mut v: Vec<*const *mut Jim_Obj> = Vec::new();
    for i in 0..objc as isize {
        //println!("COMMAND LINE: {}", i); 
        // dbg_obj(objv.offset(i));
        v.push(objv.offset(i));
    }
    // 0 is our own command; match our first argument: 1 
    match CStr::from_ptr((**v[1]).bytes).to_str().unwrap() {
        "close" => {
             Jim_Free((*interp).cmdPrivData);
             let cmd_name = (**v[0]).bytes;
             Jim_DeleteCommand(interp, cmd_name);
             return JIM_OK as c_int;
        },
        "put" => {
            if cmd_len != 4 {
                println!("put takes two args: key, value");
                return JIM_ERR as c_int;
            }
            let key = CStr::from_ptr((**v[2]).bytes).to_bytes();
            let value = CStr::from_ptr((**v[3]).bytes).to_bytes();

            // Note the outer parens here. We cast *mut c_void to *mut Tree, and 
            // reborrow to &mut Tree, a regular reference. See:
            // https://doc.rust-lang.org/std/mem/fn.transmute.html#alternatives
            let tree = &mut *((*interp).cmdPrivData as *mut Tree);
            tree.set(key.to_vec(), value.to_vec());

        },
        "get" => {
            if cmd_len != 3 {
                println!("get takes one arg: key");
                return JIM_ERR as c_int;
            }
            let key = CStr::from_ptr((**v[2]).bytes).to_bytes();
            let tree = &mut *((*interp).cmdPrivData as *mut Tree);
            if let Ok(Some(val)) = tree.get(key) {
                println!("val: {:?}", val);
                let s = CString::new(val).unwrap();
                Jim_SetResultFormatted(interp, s.as_ptr());
                return JIM_OK as c_int;
            }
        },
        _ => {},
    }
    //dbg_interp(interp);
    JIM_OK as c_int
}

fn dbg_interp(interp: *mut Jim_Interp) {
    unsafe {
        // cur_script is the _entire_ script we're running
        let cur_script = *(*interp).currentScriptObj;
        dbg_obj_struct(&cur_script, "cur_script");
        println!("eval depth: {:?}", unsafe {(*interp).evalDepth});
    }
}

fn dbg_obj_struct(obj: &Jim_Obj, msg: &str) {
        println!("\tOBJECT {:?}", msg);
        println!("typePtr: {:?}", unsafe {CStr::from_ptr((*obj.typePtr).name )});
        println!("bytes: {:?}", unsafe {CStr::from_ptr(obj.bytes)});
}

fn dbg_obj(obj: *const *mut Jim_Obj) {
        println!("\t*const *mut OBJECT: {:?}", obj);
        println!("typePtr: {:?}", unsafe {CStr::from_ptr((*((**obj).typePtr)).name )});
        println!("bytes: {:?}", unsafe {CStr::from_ptr((**obj).bytes )});
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
