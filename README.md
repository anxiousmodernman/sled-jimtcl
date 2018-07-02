# jimtsh extensions

Install Jim Tcl libraries and the `jimsh` interpreter by cloning and running

```
./configure
make
sudo make install
```

Build the rust extension (this project) as a shared library, and copy to /usr/local/lib/jim/

A **naming scheme is required** to allow the `jimsh` to find your extension.

* `package require foo`
* build artifact is named foo.so (or .tcl), located at /usr/local/lib/jim/foo.so
* Init function is named `Jim_<name>Init` and has signature 
  `pub fn Jim_fooInit(interp: *mut Jim_Interp) -> c_int`
* Init and command functions are marked `#[no_mangle]` so C can find them. 

## Hacking

Use a symlink so you can avoid copying the .so file after every `cargo build`,
e.g. debug builds. Note that we rename the shared object to comply with the 
Jim Tcl extension naming scheme.

```
sudo ln -s $(pwd)/target/debug/libsled_jimtcl.so /usr/local/lib/jim/sled.so
```

