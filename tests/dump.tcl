#!/usr/bin/env jimsh

package require sled

sled db /home/coleman/.config/dailybuilds/data.db

db dump
