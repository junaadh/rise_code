#!/bin/sh

type=$1

  session_name=$2
if [[ $1 == "attach" ]]; then
  path=$3
  language=$(echo $path | cut -d'/' -f5)
  repo=$(git --git-dir=$path/.git config --get remote.origin.url | cut -d':' -f2)
  
  hx_window=$(tmux list-windows -t $session_name -F '#{window_index} #{window_name}' | grep hx | cut -d' ' -f1)
  file_name=$(tmux capture-pane -p -t $session_name:$hx_window | grep sel | awk '{ print $2 }' | tail -n 1)
  
  echo $session_name:$language:$file_name:$repo | nc -U /tmp/dev_rpc
  exit 0
else
  echo $session_name | nc -U /tmp/dev_rpc
  exit 0
fi
