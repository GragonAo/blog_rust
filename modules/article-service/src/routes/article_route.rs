use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{delete, get, post, put},
};
use common_core::domain::page::{Page, PageResult};
use common_core::error::AppError;
use common_web::{domain::r::R, error::ApiError};

use crate::{
    domain::{
        bo::article_bo::{ArticleDetailBo, ArticleQuery},
        request::article::ArticleRequest,
        response::article::{ArticleDetailRes, ArticleSummaryRes},
    },
    startup::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_article))
        .route("/update", put(update_article))
        .route("/delete/{id}", delete(delete_article))
        .route("/detail/{id}", get(get_article_detail))
        .route("/list", get(get_article_list))
}

/// 创建文章
async fn create_article(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ArticleRequest>,
) -> Result<Json<R<i64>>, ApiError> {
    let user_id = get_user_id_from_header(&headers)?;
    let article_detail_bo = ArticleDetailBo {
        id: None,
        uid: user_id,
        title: req.title,
        description: req.description,
        content: req.content,
        cover_urls: req.cover_urls,
    };
    let id = state.article_service.insert(article_detail_bo).await?;
    Ok(Json(R::ok(id)))
}

/// 更新文章
async fn update_article(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ArticleRequest>,
) -> Result<Json<R<()>>, ApiError> {
    let user_id = get_user_id_from_header(&headers)?;
    let article_detail_bo = ArticleDetailBo {
        id: req.id,
        uid: user_id,
        title: req.title,
        description: req.description,
        content: req.content,
        cover_urls: req.cover_urls,
    };

    // 校验所有权
    let id = article_detail_bo
        .id
        .ok_or_else(|| ApiError(AppError::internal("Article ID is required")))?;
    if !state.article_service.check_ownership(id, user_id).await? {
        return Err(ApiError(AppError::internal("Permission denied")));
    }

    state.article_service.update(article_detail_bo).await?;
    Ok(Json(R::ok(())))
}

/// 删除文章
async fn delete_article(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ApiError> {
    let user_id = get_user_id_from_header(&headers)?;

    // 校验所有权
    if !state.article_service.check_ownership(id, user_id).await? {
        return Err(ApiError(AppError::internal("Permission denied")));
    }

    state.article_service.delete(id).await?;
    Ok(Json(R::ok(())))
}

/// 获取文章详情
async fn get_article_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<R<ArticleDetailRes>>, ApiError> {
    let article = state.article_service.get_article_details(id).await?;
    match article {
        Some(a) => Ok(Json(R::ok(a.into()))),
        None => Err(ApiError(AppError::db("Article not found"))),
    }
}

/// 获取文章列表
async fn get_article_list(
    State(state): State<AppState>,
    Query(article_query): Query<ArticleQuery>,
    Query(page): Query<Page>,
) -> Result<Json<R<PageResult<ArticleSummaryRes>>>, ApiError> {
    let result = state
        .article_service
        .get_article_list(article_query, page)
        .await?;

    let summary_list: Vec<ArticleSummaryRes> =
        result.list.into_iter().map(|item| item.into()).collect();

    Ok(Json(R::ok(PageResult {
        list: summary_list,
        total: result.total,
        page_num: result.page_num,
        page_size: result.page_size,
    })))
}

/// 辅助函数：从 header 获取用户 ID
fn get_user_id_from_header(headers: &HeaderMap) -> Result<i64, ApiError> {
    headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| ApiError(AppError::internal("User not authenticated")))
}
