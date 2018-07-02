# jimtsh extensions

Install Jim Tcl libraries and the `jimsh` interpreter by cloning and running

```
./configure
make
sudo make install
jimsh  # the jim tcl interpreter
```

Build the rust extension (this project) as a shared library, and copy to /usr/local/lib/jim/

A **naming scheme is required** to allow the `jimsh` to find your extension. For
example, to create a package named `foo`, the name should be the same across 
several locations in Tcl, Rust (or C) code, and on disk.

* Users of your package call `package require foo` in Tcl
* Build artifact is named foo.so (or .tcl), located at /usr/local/lib/jim/foo.so
* Init function (in Rust or C) is named `Jim_<name>Init` and has signature 
  `pub fn Jim_fooInit(interp: *mut Jim_Interp) -> c_int`
* In Rust, Init and command functions are marked `#[no_mangle]` so C can find them.

## Hacking

Use a symlink so you can avoid copying the .so file after every `cargo build`,
e.g. debug builds. Note that we rename the shared object to comply with the 
Jim Tcl extension naming scheme.

```
sudo ln -s $(pwd)/target/debug/libsled_jimtcl.so /usr/local/lib/jim/sled.so
```

You will need to restart the Jim Tcl interpreter if you rebuild this extension.

