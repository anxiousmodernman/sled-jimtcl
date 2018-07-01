#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// #include <jim.h>
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/*
    pub fn Jim_String(objPtr: *mut Jim_Obj) -> *const ::std::os::raw::c_char;

    pub fn Jim_GetString(
        objPtr: *mut Jim_Obj,
        lenPtr: *mut ::std::os::raw::c_int,
    ) -> *const ::std::os::raw::c_char;

    pub fn Jim_Alloc(size: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_void;

 */

use std::mem;
use std::ffi::{CStr, CString};

use std::os::raw::{c_int, c_void, c_char};

/// Our main command. In Tcl, when call "sled" command, we are calling this
/// function. We expect to be called with "sled dbname /path/to/db". This function
/// sets a new command dbname on the interpreter, and associates it with a pointer
/// to a sled tree.
#[no_mangle]
pub unsafe extern "C" fn Rusty_Cmd(interp: *mut Jim_Interp, objc: c_int, objv: *const *mut Jim_Obj) ->  c_int {

    let rawvec = objv as *mut u8;
    let constructed: Vec<*mut Jim_Obj> = Vec::from_raw_parts(rawvec, 2, 2);
    println!("constructed: {:?}", constructed);
    
    let temp = CString::new("subcmd").unwrap();
    unsafe {
//        let mut objref: Jim_Obj = std::ptr::read(objv);
    }
    //Jim_SetResultFormatted(interp, format.as_ptr(), msg.as_ptr());
    let mut privData: c_void = mem::uninitialized();
    Jim_CreateCommand(interp, temp.as_ptr(), Some(wrapper), &mut privData, None);
    JIM_OK as c_int
}

pub unsafe extern "C" fn wrapper(interp: *mut Jim_Interp, objc: c_int, objv: *const *mut Jim_Obj) ->  c_int {
    println!("subcommand wrapper!");
    JIM_OK as c_int
}

#[no_mangle]
pub fn Jim_rustyInit(interp: *mut Jim_Interp) -> c_int {
    let cmdName = CString::new("sled").unwrap();
    let delProc: Jim_DelCmdProc = None;
    let mut privData: c_void = unsafe { mem::zeroed() };

    let mut i: c_int = 1;
    unsafe {
        i = Jim_CreateCommand(interp, cmdName.as_ptr(), Some(Rusty_Cmd), &mut privData, delProc);
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
