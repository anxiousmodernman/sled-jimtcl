#!/usr/bin/env jimsh

# Global configs:
# number of iterations for stress test. (lower for faster runs)
set STRESS_TEST 1
set TIMED_ITERS 1

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

set G foo

proc nested {} {
    # db captured is captured from the surrounding env. Behaves the same.
    db put key1 val1
    set a [db get key1]
    # some other command
    puts "got $a"
}

# Call the proc with the captured env.
nested

if {[db get key1] != "val1" } {
    error "db should be captured by nested proc closure"
}

# test scanning

set count 0

db put xx:aa foo
db put xx:ab baz 
db put aa:c zaz

db scan xx { k v } {
    incr count
    puts "k: $k v: $v"
}
puts "count is $count"
if {[expr $count != 2]} {
    error "expected exactly 2 keys to be scanned"
}


if {$STRESS_TEST} {
    puts "stress tests..."

    set result [time {
        set count 0
        while {[incr count] <= 10000} {
            db put timed:insert:$count value
        }
    } $TIMED_ITERS]
    puts "Putting 10000 keys:"
    puts $result

    set result [time {
        db scan timed:insert: { k v } {}
    } $TIMED_ITERS]
    puts "Scanning 10000 keys:"
    puts $result
  
    set result [time {
        db close
        sled db $testdb
    } 100]
    puts "Consecutive db open/closes:"
    puts $result
}

# clean up test db
file delete -force $testdb

