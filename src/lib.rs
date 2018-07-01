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
use std::ffi::{CString};

use std::os::raw::{c_int, c_void, c_char};

#[no_mangle]
pub unsafe extern "C" fn Rusty_Cmd(interp: *mut Jim_Interp, objc: c_int, objv: *const *mut Jim_Obj) ->  c_int {
    //println!("can we print");
    //Jim_SetResultString(interp, "Hello, World!", -1);
    let format = CString::new("Hello %s").unwrap();
    let msg = CString::new("world").unwrap();
    Jim_SetResultFormatted(interp, format.as_ptr(), msg.as_ptr());
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
