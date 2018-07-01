# jimtsh extensions

Install Jim Tcl libraries and the `jimsh` interpreter by cloning and running

```
./configure
make
sudo make install
```

Build the rust extension as a shared library, and copy to /usr/local/lib/jim/

A **naming scheme is required** to allow the `jimsh` to find your extension.

* `package require foo`
* build artifact is named foo.so (or .tcl), located at /usr/local/lib/jim/foo.so
* Init function has signature `pub fn Jim_rustyInit(interp: *mut Jim_Interp) -> c_int`


