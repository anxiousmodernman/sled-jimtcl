#!/usr/bin/env jimsh


# Remove our test database, if it exists
set testdb .test.db
if {[file exists $testdb]} {
    file delete -force $testdb
}

# looks for sled.so (or sled.tcl) in the extensions directory, then calls
# the Jim_sledInit function. This function registers a "sled" command in our
# interpreter.
package require sled

# Use the sled command to open a database at the ".test.db" directory, and then
# register another command "db" in our environment. We could have chosen another
# name for our db command, but "db" seems right.
sled db $testdb

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
# nested

# test scanning

set count 0

db put xx:aa foo
db put xx:ab baz 
db put aa:c zaz
db scan xx theValue {
    incr count
    puts $theValue
}
puts "count is $count"
if {[expr $count != 2]} {
    error "expected exactly 2 keys to be scanned"
}


puts "putting 10k keys"
set result [time {
   set count 0
   while {[incr count] <= 10000} {
       db put timed:insert:$count value
   }
} 10]
puts "Stress test result"
puts $result


