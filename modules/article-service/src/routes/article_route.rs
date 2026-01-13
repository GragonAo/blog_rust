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
        response::{
            article::{ArticleDetailRes, ArticleSummaryRes},
            authorship::AuthorshipRes,
        },
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
        .route("/like/{id}", post(like_article))
        .route("/collect/{id}", post(collect_article))
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
    State(mut state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<R<ArticleDetailRes>>, ApiError> {
    // 使用service层的view_article方法，包含浏览统计
    let article = state.article_service.view_article(id).await?;
    match article {
        Some(a) => {
            let authorship_res = fetch_authorship_res(&mut state, a.uid).await?;
            let res = ArticleDetailRes {
                id: a.id.to_string(),
                authorship: Some(authorship_res),
                title: a.title,
                description: a.description,
                content: a.content,
                status: a.status,
                likes: a.likes,
                views: a.views,
                collects: a.collects,
                cover_urls: a.cover_urls,
                created_at: a.created_at,
                updated_at: a.updated_at,
                deleted_at: a.deleted_at,
            };
            Ok(Json(R::ok(res)))
        }
        None => Err(ApiError(AppError::db("Article not found"))),
    }
}

/// 获取文章列表
async fn get_article_list(
    State(mut state): State<AppState>,
    Query(article_query): Query<ArticleQuery>,
    Query(page): Query<Page>,
) -> Result<Json<R<PageResult<ArticleSummaryRes>>>, ApiError> {
    let result = state
        .article_service
        .get_article_list(article_query, page)
        .await?;

    // 获取每篇文章的作者信息
    let mut summary_list: Vec<ArticleSummaryRes> = Vec::new();
    for item in result.list.into_iter() {
        let authorship_res = fetch_authorship_res(&mut state, item.uid).await?;
        let mut summary_res: ArticleSummaryRes = item.into();
        summary_res.authorship = Some(authorship_res);
        summary_list.push(summary_res);
    }

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

/// 辅助函数：获取作者信息响应对象
async fn fetch_authorship_res(state: &mut AppState, uid: i64) -> Result<AuthorshipRes, ApiError> {
    let authorship = state.authorship_service.get_authorship(uid).await?;

    // 调用 gRPC 获取用户信息
    let (username, avatar_url) = state
        .user_grpc_client
        .get_user_info(authorship.uid)
        .await
        .unwrap_or_else(|_| ("Unknown".to_string(), String::new()));

    Ok(AuthorshipRes {
        id: authorship.uid.to_string(),
        name: username,
        avatar_url,
        like_count: authorship.like_count,
        fellow_count: authorship.fellow_count,
        collect_count: authorship.collect_count,
        article_count: authorship.article_count_public,
    })
}

/// 点赞文章
/// 点赞文章
async fn like_article(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ApiError> {
    let _user_id = get_user_id_from_header(&headers)?;

    // 调用service层的点赞方法
    state.article_service.like_article(id).await?;

    Ok(Json(R::ok(())))
}

/// 收藏文章
async fn collect_article(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, ApiError> {
    let _user_id = get_user_id_from_header(&headers)?;

    // 调用service层的收藏方法
    state.article_service.collect_article(id).await?;

    Ok(Json(R::ok(())))
}
