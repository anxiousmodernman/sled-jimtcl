#!/usr/bin/env jimsh

package require sled

sled db /tmp/sled-jimtcl.db

set SOCKET /tmp/ipc.sock
if {[file exists $SOCKET]} {
    puts "Error: socket exists. Removing..."
    file delete $SOCKET
}

proc serve {path} {
    variable f {}
    variable cmds [list]
    set f [socket unix.server $path]
    fconfigure $f -buffering full -blocking 0
    $f readable {
        set client [$f accept]
        $client buffering none
        puts "accepting conn..."
        set expected 1
        set cmds [list]
        $client gets buf
        puts "buf: $buf"
        switch [lindex [split $buf] 0] {
            "db" {
                puts "a db command"
                set resp [eval $buf]
                $client puts $resp
            }
            default {
                $client puts OK
            }
        }
        $client close
    }
    # We block here.
    # A call to `vwait` enters the eventloop. `vwait` processes
    # events until the named (global) variable changes or all
    # event handlers are removed. The variable need not exist
    # beforehand.  If there are no event handlers defined, `vwait`
    # returns immediately.
    vwait done
}

signal handle SIGINT
try -signal {
    serve $SOCKET
    vwait done
} on signal {sig} {
    # we aren't checking what $sig is, since we only handle SIGINT
    file delete $SOCKET
    puts "\nGoodbye."
}


