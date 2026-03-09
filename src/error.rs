use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use uuid::Uuid;

/// All errors that Corro can return, mapped to their S3 error code equivalents.
// Variants will be constructed by HTTP handlers in issues #13–#32.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum CorroError {
    #[error("The specified bucket does not exist")]
    NoSuchBucket,

    #[error("The specified key does not exist")]
    NoSuchKey,

    #[error("The specified bucket already exists")]
    BucketAlreadyExists,

    #[error("The bucket you tried to delete is not empty")]
    BucketNotEmpty,

    #[error("The specified bucket is not valid: {0}")]
    InvalidBucketName(String),

    #[error("The Content-MD5 you specified is not valid")]
    InvalidDigest,

    #[error("The request signature we calculated does not match the signature you provided")]
    SignatureDoesNotMatch,

    #[error("Access Denied")]
    AccessDenied,

    #[error("The AWS Access Key Id you provided does not exist in our records")]
    InvalidAccessKeyId,

    #[error("You must provide the Content-Length HTTP header")]
    MissingContentLength,

    #[error("Your proposed upload exceeds the maximum allowed object size")]
    EntityTooLarge,

    #[error("A header you provided implies functionality that is not implemented")]
    NotImplemented,

    #[error("The XML you provided was not well-formed or did not validate against our schema")]
    MalformedXML,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("We encountered an internal error. Please try again")]
    Internal(#[source] anyhow::Error),
}

impl CorroError {
    /// The S3 error code string returned to clients.
    pub fn s3_code(&self) -> &'static str {
        match self {
            Self::NoSuchBucket => "NoSuchBucket",
            Self::NoSuchKey => "NoSuchKey",
            Self::BucketAlreadyExists => "BucketAlreadyExists",
            Self::BucketNotEmpty => "BucketNotEmpty",
            Self::InvalidBucketName(_) => "InvalidBucketName",
            Self::InvalidDigest => "InvalidDigest",
            Self::SignatureDoesNotMatch => "SignatureDoesNotMatch",
            Self::AccessDenied => "AccessDenied",
            Self::InvalidAccessKeyId => "InvalidAccessKeyId",
            Self::MissingContentLength => "MissingContentLength",
            Self::EntityTooLarge => "EntityTooLarge",
            Self::NotImplemented => "NotImplemented",
            Self::MalformedXML => "MalformedXML",
            Self::InvalidArgument(_) => "InvalidArgument",
            Self::Internal(_) => "InternalError",
        }
    }

    pub fn http_status(&self) -> StatusCode {
        match self {
            Self::NoSuchBucket => StatusCode::NOT_FOUND,
            Self::NoSuchKey => StatusCode::NOT_FOUND,
            Self::BucketAlreadyExists => StatusCode::CONFLICT,
            Self::BucketNotEmpty => StatusCode::CONFLICT,
            Self::InvalidBucketName(_) => StatusCode::BAD_REQUEST,
            Self::InvalidDigest => StatusCode::BAD_REQUEST,
            Self::SignatureDoesNotMatch => StatusCode::FORBIDDEN,
            Self::AccessDenied => StatusCode::FORBIDDEN,
            Self::InvalidAccessKeyId => StatusCode::FORBIDDEN,
            Self::MissingContentLength => StatusCode::LENGTH_REQUIRED,
            Self::EntityTooLarge => StatusCode::BAD_REQUEST,
            Self::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            Self::MalformedXML => StatusCode::BAD_REQUEST,
            Self::InvalidArgument(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Wrap any error as an internal server error.
    #[allow(dead_code)]
    pub fn internal(err: impl Into<anyhow::Error>) -> Self {
        Self::Internal(err.into())
    }
}

impl IntoResponse for CorroError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4();
        let code = self.s3_code();
        let message = self.to_string();
        let status = self.http_status();

        // Log internal errors with the full source chain
        if matches!(self, CorroError::Internal(_)) {
            tracing::error!(error = %self, request_id = %request_id, "Internal server error");
        }

        let body = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Error>\
<Code>{code}</Code>\
<Message>{message}</Message>\
<RequestId>{request_id}</RequestId>\
</Error>"
        );

        (
            status,
            [("content-type", "application/xml; charset=utf-8")],
            body,
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn no_such_bucket_returns_404() {
        let response = CorroError::NoSuchBucket.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn error_body_is_valid_xml() {
        let response = CorroError::NoSuchBucket.into_response();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let xml = std::str::from_utf8(&body).unwrap();
        assert!(xml.contains("<Code>NoSuchBucket</Code>"));
        assert!(xml.contains("<Message>"));
        assert!(xml.contains("<RequestId>"));
    }

    #[tokio::test]
    async fn internal_error_returns_500() {
        let err = CorroError::internal(anyhow::anyhow!("disk full"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn all_variants_have_s3_code() {
        // Ensures no variant is forgotten in the match
        let errors: Vec<CorroError> = vec![
            CorroError::NoSuchBucket,
            CorroError::NoSuchKey,
            CorroError::BucketAlreadyExists,
            CorroError::BucketNotEmpty,
            CorroError::InvalidBucketName("x".into()),
            CorroError::InvalidDigest,
            CorroError::SignatureDoesNotMatch,
            CorroError::AccessDenied,
            CorroError::InvalidAccessKeyId,
            CorroError::MissingContentLength,
            CorroError::EntityTooLarge,
            CorroError::NotImplemented,
            CorroError::MalformedXML,
            CorroError::InvalidArgument("x".into()),
            CorroError::internal(anyhow::anyhow!("test")),
        ];
        for e in errors {
            assert!(!e.s3_code().is_empty());
        }
    }
}
