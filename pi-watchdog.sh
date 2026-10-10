#!/bin/bash

router="10.41.0.1"
ping_count=3
fail_threshold=3
fail_file="/tmp/wifi_watchdog_fails"

if [ ! -f "$fail_file" ]; then
    echo 0 > "$fail_file"
fi

failures=$(cat "$fail_file")

if ping -c "$ping_count" -W 3 "$router" > /dev/null 2>&1; then
    echo 0 > "$fail_file"
    logger "wifi_watchdog: ping OK, fail counter reset"
else
    failures=$((failures + 1))
    echo "$failures" > "$fail_file"
    logger "wifi_watchdog: ping failed, consecutive failures: $failures"

    if [[ "$failures" -ge "$fail_threshold" ]]; then
        logger "wifi_watchdog: threshold reached, attempting WiFi restart"

        nmcli connection down "YourConnectionName"
        sleep 5
        nmcli connection up "YourConnectionName"
        sleep 15

        if ping -c 3 -W 3 "$router" > /dev/null 2>&1; then
            logger "wifi_watchdog: WiFi restart successful"
            echo 0 > "$fail_file"
        else
            logger "wifi_watchdog: WiFi restart failed, rebooting"
            echo 0 > "$fail_file"
            /sbin/reboot
        fi
    fi
fi