SELECT
    id
FROM
    worker.dequeue ($1::int);

