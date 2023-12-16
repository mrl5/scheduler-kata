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
    Ok(sqlx::query_file_as!(
        model::TaskId,
        "sql/create_task.sql",
        id,
        task_type.to_string(),
        not_before
    )
    .fetch_one(&db)
    .await?)
}

pub async fn show_task(db: DB, id: Uuid) -> anyhow::Result<Option<model::TaskView>> {
    Ok(
        sqlx::query_file_as!(model::TaskView, "sql/show_task.sql", id,)
            .fetch_optional(&db)
            .await?,
    )
}

pub async fn delete_task(
    db: DB,
    id: Uuid,
    forbidden_states: &[String],
) -> anyhow::Result<Option<model::TaskSnapshot>> {
    Ok(sqlx::query_file_as!(
        model::TaskSnapshot,
        "sql/delete_task.sql",
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
select 1 as t from task where id = $1::uuid
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
select
    id,
    typ,
    state
from task_state
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
            query.push(" where ");
            let mut separated = query.separated(" and ");
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
        let test_sql = "select id from task";
        let anchor: Uuid = Ulid::new().into();
        let test_cases = vec![
            (
                None,
                model::TaskFilter {
                    typ: None,
                    state: None,
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "select id from task",
            ),
            (
                None,
                model::TaskFilter {
                    typ: Some(model::TaskType::TypeA),
                    state: None,
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "select id from task where typ = $1",
            ),
            (
                None,
                model::TaskFilter {
                    typ: None,
                    state: Some(model::TaskState::Pending),
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "select id from task where state = $1",
            ),
            (
                None,
                model::TaskFilter {
                    typ: Some(model::TaskType::TypeA),
                    state: Some(model::TaskState::Pending),
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "select id from task where typ = $1 and state = $2",
            ),
            (
                Some(anchor),
                model::TaskFilter {
                    typ: None,
                    state: None,
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "select id from task where id < $1",
            ),
            (
                Some(anchor),
                model::TaskFilter {
                    typ: Some(model::TaskType::TypeA),
                    state: None,
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "select id from task where id < $1 and typ = $2",
            ),
            (
                Some(anchor),
                model::TaskFilter {
                    typ: None,
                    state: Some(model::TaskState::Pending),
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "select id from task where id < $1 and state = $2",
            ),
            (
                Some(anchor),
                model::TaskFilter {
                    typ: Some(model::TaskType::TypeA),
                    state: Some(model::TaskState::Pending),
                },
                QueryBuilder::new(test_sql) as QueryBuilder<Postgres>,
                "select id from task where id < $1 and typ = $2 and state = $3",
            ),
        ];

        for (anchor, filter, query, expected) in test_cases {
            let result = filter.append_query_with_fragment(query, anchor);
            assert_eq!(result.sql(), expected);
        }
    }
}
