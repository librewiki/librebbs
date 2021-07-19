use actix_web::{dev, error::ErrorUnauthorized, Error, FromRequest, HttpMessage, HttpRequest};
use chrono::prelude::*;
use derive_more::{Display, Error};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize};
use std::env;

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,
    #[serde(deserialize_with = "from_float")]
    iat: DateTime<Utc>,
    #[serde(deserialize_with = "from_float")]
    nbf: DateTime<Utc>,
    #[serde(deserialize_with = "from_float")]
    exp: DateTime<Utc>,
    sub: String,
    scopes: Vec<String>,
}

fn from_float<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: f64 = Deserialize::deserialize(deserializer)?;
    let timestamp = s as i64;
    let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
    Ok(dt)
}

#[derive(Debug, Display, Error)]
enum DecodeError {
    TokenExpired,
    TokenInvalid,
}

lazy_static! {
    static ref PUBLIC_KEY: Vec<u8> = std::fs::read("pubkey.pem").expect("public key must exist");
    static ref DECODING_KEY: DecodingKey<'static> =
        DecodingKey::from_rsa_pem(&PUBLIC_KEY).expect("must succeed");
}

fn decode(token: &str) -> Result<Claims, DecodeError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;
    validation.set_audience(&[env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID must be set")]);
    let token_data = jsonwebtoken::decode::<Claims>(&token, &DECODING_KEY, &validation)
        .map_err(|_| DecodeError::TokenInvalid)?;
    if token_data.claims.exp < Utc::now() {
        Err(DecodeError::TokenExpired)
    } else {
        Ok(token_data.claims)
    }
}

#[derive(Debug)]
pub struct UserInfo {
    pub id: String,
    pub token: String,
}

impl FromRequest for UserInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        if let Some(token_cookie) = req.cookie("access_token") {
            let token = token_cookie.value();
            match decode(&token) {
                Ok(decoded) => ok(UserInfo {
                    id: decoded.sub,
                    token: token.to_owned(),
                }),
                Err(e) => match e {
                    DecodeError::TokenExpired => err(ErrorUnauthorized("TokenExpired")),
                    DecodeError::TokenInvalid => err(ErrorUnauthorized("TokenInvalid")),
                },
            }
        } else {
            err(ErrorUnauthorized("Token is missing"))
        }
    }
}

pub struct RefreshToken {
    pub refresh_token: String,
}

impl FromRequest for RefreshToken {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        if let Some(token_cookie) = req.cookie("refresh_token") {
            ok(RefreshToken {
                refresh_token: token_cookie.value().to_owned(),
            })
        } else {
            err(ErrorUnauthorized("Token is missing"))
        }
    }
}
