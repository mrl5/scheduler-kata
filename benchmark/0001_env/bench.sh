#!/bin/bash

psql -h localhost -U postgres -d scheduler-kata \
    -c 'DELETE FROM tenant_default.queue'

psql -h localhost -U postgres -d scheduler-kata \
    -c "UPDATE tenant_default.task SET inactive_since = null, state = null WHERE state = 'failed' OR state = 'done'"

psql -h localhost -U postgres -d scheduler-kata \
    -c 'SELECT pg_stat_statements_reset()'
