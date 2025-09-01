use axum::RequestPartsExt;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::Deref;

use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request;

pub struct Auth<A, E = FromCookie>
where
    A: Authenticator,
    E: SessionIdExtractor,
{
    inner: A::User,

    _authenticator_marker: PhantomData<A>,
    _session_id_extractor_marker: PhantomData<E>,
}

impl<A, E> Auth<A, E>
where
    A: Authenticator,
    E: SessionIdExtractor,
{
    pub fn into_inner(self) -> A::User {
        self.inner
    }
}

impl<A, E> Deref for Auth<A, E>
where
    A: Authenticator,
    E: SessionIdExtractor,
{
    type Target = A::User;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S, A, E> FromRequestParts<S> for Auth<A, E>
where
    S: Send + Sync,
    A: Authenticator + axum::extract::FromRef<S> + Send + Sync,
    E: SessionIdExtractor,
{
    type Rejection = A::Error;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match E::extract(parts).await {
            Some(session_id) => {
                let authenticator = A::from_ref(state);
                let inner = authenticator.authenticate(&session_id).await?;

                Ok(Self {
                    inner,
                    _authenticator_marker: PhantomData,
                    _session_id_extractor_marker: PhantomData,
                })
            }
            None => Err(A::missing_session_id()),
        }
    }
}

impl<S, A, E> OptionalFromRequestParts<S> for Auth<A, E>
where
    S: Send + Sync,
    A: Authenticator + axum::extract::FromRef<S> + Send + Sync,
    E: SessionIdExtractor,
{
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        <Self as FromRequestParts<S>>::from_request_parts(parts, state)
            .await
            .map_or_else(|_| Ok(None), |v| Ok(Some(v)))
    }
}

pub trait Authenticator {
    type User;
    type Error: axum::response::IntoResponse;

    fn authenticate(
        &self,
        session_id: &str,
    ) -> impl Future<Output = Result<Self::User, Self::Error>> + Send + Sync;

    fn missing_session_id() -> Self::Error;
}

pub trait SessionIdExtractor {
    fn extract(parts: &mut request::Parts) -> impl Future<Output = Option<String>> + Send + Sync;
}

pub struct FromCookie;

impl SessionIdExtractor for FromCookie {
    async fn extract(parts: &mut request::Parts) -> Option<String> {
        let cookies = parts
            .extract::<axum_extra::TypedHeader<axum_extra::headers::Cookie>>()
            .await
            .ok()?;
        let session_cookie = cookies.get("cookie")?;
        Some(session_cookie.to_string())
    }
}
