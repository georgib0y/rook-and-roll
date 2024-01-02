#!/bin/bash

LOG_DIR="/home/george/progs/rookandroll/logs"
NEWEST_LOG=$(find $LOG_DIR -type f -exec ls -t1 {} + | head -1)

PID=$(echo $NEWEST_LOG | sed -E "s/.*\/([0-9]+)_log.log/\1/g")

if [[ -z $(ps -p $PID --no-header) ]]; then
    echo "No current process running"
    exit 1
fi
 
echo $NEWEST_LOG
tail -f $NEWEST_LOG

