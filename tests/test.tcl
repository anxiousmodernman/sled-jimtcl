#!/usr/bin/env jimsh

# Global configs:
# number of iterations for stress test. (lower for faster runs)
set PUT_ITERS 1
set STRESS_TEST 1

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

db scan xx theValue {
    incr count
    puts $theValue
}
puts "count is $count"
if {[expr $count != 2]} {
    error "expected exactly 2 keys to be scanned"
}


puts "putting 10k keys"
if {$STRESS_TEST} {
  set result [time {
     set count 0
     while {[incr count] <= 10000} {
         db put timed:insert:$count value
     }
  } $PUT_ITERS]
  puts "Stress test result"
  puts $result
}

proc dont_segfault {} {
    variable a_null {}
    global G
    db put beginning$G $a_null
    db put beginning1 $a_null
    db put beginning2 $a_null
    db put beginning3 $a_null
    db put beginning4 $a_null
    db put beginning5 $a_null
    db put beginning6 $a_null
}

dont_segfault

