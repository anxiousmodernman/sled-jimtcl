#!/usr/bin/env jimsh

set SOCKET /tmp/ipc.sock

set client [socket unix $SOCKET]
set msg [list db put foo baz]
variable msg2 {
    db put blah 1
    db put blah 99
    db get blah
}
puts [list $msg2]

$client puts $msg2
$client flush
$client gets reply
puts $reply
$client gets reply
puts $reply
$client gets reply
puts $reply


$client close
