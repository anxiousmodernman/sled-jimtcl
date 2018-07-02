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
pub unsafe extern "C" fn Rusty_Cmd(
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

    // TODO can we avoid transmute?
//    let transmuted = 
 //       std::mem::transmute::<&mut Tree, *mut c_void>(&mut tree);

//    let casted = &mut tree as *mut Tree;
    let boxed: *mut Tree = Box::into_raw(Box::new(tree));
    let sz = std::mem::size_of::<*mut Tree>();
    let mut ttptr = Jim_Alloc(sz as c_int);
    let tptr = 
        std::mem::transmute::<*mut Tree, *mut c_void>(boxed);
    ttptr = tptr;


    let name = CStr::from_ptr((**name_ptr).bytes);
    Jim_CreateCommand(interp, name.as_ptr(), Some(wrapper), ttptr, None)
}

pub unsafe extern "C" fn wrapper(
    interp: *mut Jim_Interp,
    objc: c_int,
    objv: *const *mut Jim_Obj,
) -> c_int {
    println!("subcommand wrapper!");
    // pub cmdPrivData: *mut ::std::os::raw::c_void,
    //let mut private = (*interp).cmdPrivData;
    let mut tree = (*interp).cmdPrivData as *mut Tree;
    println!("ref casted!");
    let k = b"first".to_vec();
    if let Ok(Some(val)) = (*tree).get(&k) {
        let newval: u8 = val[0] + 1;
        println!("val: {:?}", newval);
        (*tree).set(k.clone(), vec![newval]);
        return JIM_OK as c_int;
    }
    let k = b"first".to_vec();
    (*tree).set(k, vec![0]);
    JIM_OK as c_int
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
            Some(Rusty_Cmd),
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
