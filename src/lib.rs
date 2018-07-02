#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate sled;

use sled::{ConfigBuilder, Tree};

// #include <jim.h>
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/*
    pub fn Jim_String(objPtr: *mut Jim_Obj) -> *const ::std::os::raw::c_char;

    pub fn Jim_GetString(
        objPtr: *mut Jim_Obj,
        lenPtr: *mut ::std::os::raw::c_int,
    ) -> *const ::std::os::raw::c_char;

    // ALLOC the tree?
    // get size of ref
    // allocate that
    // cast ref transmute into void ptr
    pub fn Jim_Alloc(size: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_void;

 */

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
    let name_ptr = objv.offset(1);
    let path_ptr = objv.offset(2);

    // bytes is a field defined thus, so we double dereference it
    // pub bytes: *mut ::std::os::raw::c_char,
    let path = CStr::from_ptr((**path_ptr).bytes).to_str().unwrap();

    println!("loading db at path {:?}", path);
    let config = ConfigBuilder::new().path(path).build();
    let mut tree = Tree::start(config).unwrap();

    let boxed: *mut Tree = Box::into_raw(Box::new(tree));
    let sz = std::mem::size_of::<*mut Tree>();
    let mut ttptr = Jim_Alloc(sz as c_int);
    ttptr = std::mem::transmute::<*mut Tree, *mut c_void>(boxed);

    let name = CStr::from_ptr((**name_ptr).bytes);
    Jim_CreateCommand(interp, name.as_ptr(), Some(wrapper), ttptr, None)
}

pub unsafe extern "C" fn wrapper(
    interp: *mut Jim_Interp,
    objc: c_int,
    objv: *const *mut Jim_Obj,
) -> c_int {
    let mut tree = (*interp).cmdPrivData as *mut Tree;
    let mut v: Vec<*const *mut Jim_Obj> = Vec::new();
    for i in 0..objc as isize {
        v.push(objv.offset(i));
    }
    dbg_interp(interp);
    db_cmd(&mut(*tree), v);
    JIM_OK as c_int
}

fn db_cmd(tree: &mut Tree, cmd_line: Vec<*const *mut Jim_Obj>) {

    // types encountered:
    // command, source, dict, list
    for item in cmd_line {
        dbg_obj(item);
    }
}

fn dbg_interp(interp: *mut Jim_Interp) {
    unsafe {
        // cur_script is the _entire_ script we're running
        let cur_script = *(*interp).currentScriptObj;
        dbg_obj_struct(&cur_script, "cur_script");
        println!("eval depth: {:?}", unsafe {(*interp).evalDepth});
        //println!("current script: {:?}", cur_script);
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
        println!("bytes: {:?}", unsafe {CStr::from_ptr(((**obj).bytes) )});
}

#[no_mangle]
pub fn Jim_sledInit(interp: *mut Jim_Interp) -> c_int {
    let cmdName = CString::new("sled").unwrap();
    let delProc: Jim_DelCmdProc = None;
    let mut privData: c_void = unsafe { mem::zeroed() };

    let mut i: c_int = 1;
    unsafe {
        i = Jim_CreateCommand(
            interp,
            cmdName.as_ptr(),
            Some(db_init),
            &mut privData,
            delProc,
        );
    }
    if i != 0 {
        return JIM_ERR as c_int;
    }

    JIM_OK as c_int
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
