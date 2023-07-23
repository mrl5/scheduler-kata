use chrono::{DateTime, Utc};
use common::db::{paginate, Pagination, DB};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use super::model;

pub async fn create_task(
    db: DB,
    id: Uuid,
    task_type: model::TaskType,
    not_before: Option<DateTime<Utc>>,
) -> anyhow::Result<model::TaskId> {
    Ok(sqlx::query_as!(
        model::TaskId,
        r#"
WITH t AS (
    INSERT INTO task (
        id,
        typ,
        not_before
    )
    VALUES (
        $1,
        $2,
        $3
    ) RETURNING created_at, not_before, id
) INSERT INTO task_bucket SELECT created_at, not_before, id from t RETURNING id
        "#,
        id,
        task_type.to_string(),
        not_before
    )
    .fetch_one(&db)
    .await?)
}

pub async fn show_task(db: DB, id: Uuid) -> anyhow::Result<Option<model::TaskView>> {
    Ok(sqlx::query_as!(
        model::TaskView,
        r#"
        SELECT
            id,
            typ,
            state,
            created_at,
            not_before,
            inactive_since
        FROM task_state
        WHERE id = $1::uuid
        "#,
        id,
    )
    .fetch_optional(&db)
    .await?)
}

pub async fn delete_task(
    db: DB,
    id: Uuid,
    forbidden_states: &[String],
) -> anyhow::Result<Option<model::TaskSnapshot>> {
    Ok(sqlx::query_as!(
        model::TaskSnapshot,
        r#"
        WITH deleted_task AS (
            UPDATE task
            SET inactive_since = now(), state = $1
            FROM (
                SELECT id as task_id FROM task_state
                WHERE id = $2::uuid
                    AND state != ANY($3)
                    AND inactive_since IS NULL
            ) as t
            WHERE id = t.task_id
            RETURNING id, state, inactive_since
        ), dt AS (
            DELETE FROM task_bucket USING deleted_task
            WHERE task_bucket.id = deleted_task.id
        ) SELECT id, state, inactive_since FROM (
            SELECT id, state, inactive_since FROM deleted_task
            UNION ALL
            SELECT id, state, inactive_since FROM task
            WHERE id = $2::uuid AND state = $1
        ) t
        "#,
        model::TaskState::Deleted.to_string(),
        id,
        forbidden_states,
    )
    .fetch_optional(&db)
    .await?)
}

pub async fn does_task_exist(db: DB, id: Uuid) -> anyhow::Result<bool> {
    let res = sqlx::query!(
        r#"
        SELECT 1 as t FROM task WHERE id = $1::uuid
        "#,
        id,
    )
    .fetch_optional(&db)
    .await?;

    Ok(res.is_some())
}

pub async fn list_tasks(
    db: DB,
    pagination: Pagination,
    task_filter: model::TaskFilter,
) -> anyhow::Result<model::TasksList> {
    let (per_page, anchor) = paginate(pagination);
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT
            id,
            typ,
            state
        FROM task_state
        "#,
    );
    query = task_filter.append_query_with_fragment(query, anchor);
    query
        .push(" ORDER BY id desc")
        .push(" LIMIT ")
        .push_bind(per_page as i64);
    let rows = query
        .build_query_as::<model::TaskSummary>()
        .fetch_all(&db)
        .await?;

    let mut new_anchor = None;
    if rows.len() >= per_page {
        if let Some(last_row) = rows.last() {
            new_anchor = Some(last_row.id);
        }
    }

    Ok(model::TasksList {
        tasks: rows,
        anchor: new_anchor,
        per_page,
    })
}

impl model::TaskFilter {
    pub fn append_query_with_fragment(
        self,
        mut query: QueryBuilder<Postgres>,
        anchor: Option<Uuid>,
    ) -> QueryBuilder<Postgres> {
        if anchor.is_none() && self.typ.is_none() && self.state.is_none() {
            return query;
        } else {
            query.push(" WHERE ");
            let mut separated = query.separated(" AND ");
            if let Some(a) = anchor {
                separated.push("id < ").push_bind_unseparated(a);
            }
            if let Some(t) = self.typ {
                separated
                    .push("typ = ")
                    .push_bind_unseparated(t.to_string());
            }
            if let Some(s) = self.state {
                separated
                    .push("state = ")
                    .push_bind_unseparated(s.to_string());
            }
        }

        query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ulid::Ulid;

    #[test]
    fn it_should_serialize_into_where_sql_fragment() {
        let test_sql = "SELECT id FROM task";
        let anchor: Uuid = Ulid::new().into();
        let test_cases = vec![
            (
                None,
                model::TaskFilter {
                    typ: None,
                    state: None,
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "SELECT id FROM task",
            ),
            (
                None,
                model::TaskFilter {
                    typ: Some(model::TaskType::TypeA),
                    state: None,
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "SELECT id FROM task WHERE typ = $1",
            ),
            (
                None,
                model::TaskFilter {
                    typ: None,
                    state: Some(model::TaskState::Pending),
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "SELECT id FROM task WHERE state = $1",
            ),
            (
                None,
                model::TaskFilter {
                    typ: Some(model::TaskType::TypeA),
                    state: Some(model::TaskState::Pending),
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "SELECT id FROM task WHERE typ = $1 AND state = $2",
            ),
            (
                Some(anchor),
                model::TaskFilter {
                    typ: None,
                    state: None,
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "SELECT id FROM task WHERE id < $1",
            ),
            (
                Some(anchor),
                model::TaskFilter {
                    typ: Some(model::TaskType::TypeA),
                    state: None,
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "SELECT id FROM task WHERE id < $1 AND typ = $2",
            ),
            (
                Some(anchor),
                model::TaskFilter {
                    typ: None,
                    state: Some(model::TaskState::Pending),
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "SELECT id FROM task WHERE id < $1 AND state = $2",
            ),
            (
                Some(anchor),
                model::TaskFilter {
                    typ: Some(model::TaskType::TypeA),
                    state: Some(model::TaskState::Pending),
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "SELECT id FROM task WHERE id < $1 AND typ = $2 AND state = $3",
            ),
        ];

        for (anchor, filter, query, expected) in test_cases {
            let result = filter.append_query_with_fragment(query, anchor);
            assert_eq!(result.sql(), expected);
        }
    }
}
