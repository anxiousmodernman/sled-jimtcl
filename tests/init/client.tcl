#!/usr/bin/env jimsh


proc send_cmd { args } {
    variable client
    set client [socket unix /tmp/ipc.sock]
    $client puts $args
    $client flush
    $client readable {
        set reply [$client gets]
        puts $reply
        $client close
    }
    puts "waiting for reply..."
    # waits until "done" variable changes or until handlers are removed
    vwait done
}

send_cmd db put foo baz 
send_cmd db get foo 

