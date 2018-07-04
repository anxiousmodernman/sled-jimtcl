#!/usr/bin/env jimsh

# looks for sled.so (or sled.tcl) in the extensions directory, then calls
# the Jim_sledInit function. This function registers a "sled" command in our
# interpreter.
package require sled

# Use the sled command to open a database at the ".test.db" directory, and then
# register another command "db" in our environment. We could have chosen another
# name.
sled db .test.db

# The db command holds a reference to the open database. Call subcommands on
# db to do stuff with the database. TODO nothing's implemented yet, we're just
# debugging/printing stuff for now.

proc nested {} {
    # db captured is captured from the surrounding env. Behaves the same.
    db put key1 val1
    set a [db get key1]
    # some other command
    puts "got $a"
}

# Call the proc with the captured env.
nested
