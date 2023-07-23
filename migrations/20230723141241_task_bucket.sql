-- task bucket

CREATE TABLE task_bucket (
    created_at timestamptz
        NOT NULL
        DEFAULT now(),

    not_before timestamptz
        DEFAULT NULL
        CONSTRAINT not_before_check CHECK (
            created_at + '1month'::interval >= not_before
        ),

    id uuid
        NOT NULL,

    PRIMARY KEY (id)
);
