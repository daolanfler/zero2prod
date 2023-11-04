use super::IdempotencyKey;

use actix_web::{body::to_bytes, HttpResponse};
use reqwest::StatusCode;
use sqlx::postgres::PgHasArrayType;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

/// sqlx knows, via `#[sqlx(type_name="header_pair")]` attribute, the name of the composite
/// type itself. It does not know the name of the type for arrays containing `header_pair` elements.
/// Postgres creates an array type implicitly when we run `CREATE TYPE` statement - it is simply
/// the composite type name prefixed by an underscore.
impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_header_pair")
    }
}

pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    // with `!` we ask `sqlx` to forcefully assume that the columns will not be null
    // if we are wrong, it will cause an error at runtime
    let saved_response = sqlx::query!(
        r#"
        SELECT
            response_status_code as "response_status_code!",
            response_headers as "response_headers!: Vec<HeaderPairRecord>",
            response_body as "response_body!"
        FROM idempotency
        WHERE
          user_id = $1 AND
          idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
        let mut response = HttpResponse::build(status_code);

        for HeaderPairRecord { name, value } in r.response_headers {
            response.append_header((name, value));
        }
        Ok(Some(response.body(r.response_body)))
    } else {
        Ok(None)
    }
}

pub async fn save_response(
    mut transaction: Transaction<'static, Postgres>,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    http_response: HttpResponse,
) -> Result<HttpResponse, anyhow::Error> {
    // takes ownership
    let (response_head, body) = http_response.into_parts();
    // `MessageBody::Error` is not `Send` + `Sync`,
    // therefore it doesn't play nicely with `anyhow`
    let body = to_bytes(body).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    let status_code = response_head.status().as_u16() as i16;
    let headers = {
        let mut h = Vec::with_capacity(response_head.headers().len());
        for (name, value) in response_head.headers().iter() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value });
        }
        h
    };

    sqlx::query_unchecked!(
        r#"
        UPDATE idempotency 
        SET 
            response_status_code = $3,
            response_headers = $4,
            response_body = $5
        WHERE 
            user_id = $1 AND 
            idempotency_key = $2
    "#,
        user_id,
        idempotency_key.as_ref(),
        status_code,
        headers,
        body.as_ref()
    )
    .execute(&mut transaction)
    .await?;
    // IMPORTANT 这里一定要 commit
    transaction.commit().await?;

    // We need `.map_into_boxed_body` to go from
    // `HttpResponse<Bytes>` to `HttpResponse<BoxBody>`
    let http_reponse = response_head.set_body(body).map_into_boxed_body();
    Ok(http_reponse)
}

pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(HttpResponse),
}

pub async fn try_processing(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<NextAction, anyhow::Error> {
    let mut transaction = pool.begin().await?;

    // with `repeatable read` isolation, the second request will error
    // sqlx::query!("SET TRANSACTION ISOLATION LEVEL repeatable read")
    //     .execute(&mut transaction)
    //     .await?;

    // The `INSERT` statement fired by the second request must wait for outcome of the SQL
    // transaction started by the first request.
    // If the latter commits, the former will DO NOTHING.
    // If the latter rolls back, the former will actually perform the insertion
    // It is worth highlighting that this strategy will **not** work if using stricter
    // isolation levels
    let n_inserted_rows = sqlx::query!(
        r#"
        INSERT INTO idempotency (
            user_id, 
            idempotency_key,
            created_at
        )
        VALUES ($1, $2, now())
        ON CONFLICT DO NOTHING
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .execute(&mut transaction)
    .await?
    .rows_affected();

    if n_inserted_rows > 0 {
        // first request goes here
        Ok(NextAction::StartProcessing(transaction))
    } else {
        // second request here
        let saved_response = get_saved_response(pool, idempotency_key, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("We expected a saved response, we didn't find it"))?;
        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}
