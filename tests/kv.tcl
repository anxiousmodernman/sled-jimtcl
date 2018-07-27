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

set count 0

db put xx:aa foo
db put xx:ab baz 
db put aa:c zaz
db scan xx { k v } {
    incr count
    puts $v
}
puts "count is $count"
if {[expr $count != 2]} {
    error "expected exactly 2 keys to be scanned"
}


#puts "putting 10k keys"
#set result [time {
#   set count 0
#   while {[incr count] <= 10000} {
#       db put timed:insert:$count value
#   }
#} 10]
#puts "Stress test result"
#puts $result


