#!/usr/bin/env jimsh

package require sled

sled db /tmp/sled-jimtcl.db

set SOCKET /tmp/ipc.sock
if {[file exists $SOCKET]} {
    puts "Error: socket exists. Removing..."
    file delete $SOCKET
}

proc is_whitespace {s} {
    set val [string length [string trim $s]]
    puts "val: $val"
    return $val
}

proc serve {path} {
    variable f {}
    set f [socket unix.server $path]
    $f readable {
        set client [$f accept]
        puts "accepting conn..."
        set cmds [list]
        while {[$client gets buf] > -1} {
            if {[string length [string trim $buf]] > 0} {
                puts "received: $buf"
                set computed [eval $buf]
                if {[string length [string trim $computed]] > 0} {
                    puts "computed: $computed"
                    $client puts $computed
                }
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


