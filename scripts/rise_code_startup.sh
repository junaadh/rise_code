#!/bin/sh

program="rise_code"

tmux start-server

if ! pgrep "$program" >/dev/null; then
  "$program" > /tmp/$program.stdout 2> /tmp/$program.stderr & disown $!
fi
