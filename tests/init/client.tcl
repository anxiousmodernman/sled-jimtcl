#!/usr/bin/env jimsh

set SOCKET /tmp/ipc.sock

set client [socket unix $SOCKET]
set msg [list db put foo baz]
variable msg2 {
    db put blah 1
    db put blah 99
    db get blah
}

$client puts $msg2
$client flush
puts "waiting for reply..."
$client readable {
    set reply [$client gets]
    puts "reply $reply"
    $client close
}

# waits until "done" variable changes or until handlers are removed
vwait done
