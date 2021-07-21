use actix_web::{dev, error::ErrorBadRequest, Error, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use std::{net::IpAddr, str::FromStr};

pub struct ConnectionInfo {
    pub ip: IpAddr,
}

impl FromRequest for ConnectionInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let conn_info = req.connection_info();
        match conn_info.realip_remote_addr() {
            Some(remote) => ok(ConnectionInfo {
                ip: IpAddr::from_str(remote.split(':').collect::<Vec<&str>>()[0]).unwrap(),
            }),
            None => err(ErrorBadRequest("No IP address")),
        }
    }
}
