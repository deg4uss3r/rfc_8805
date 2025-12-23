use std::net::IpAddr;

use ipnet::IpNet;
use rust_iso3166::{CountryCode, from_alpha2, iso3166_2::Subdivision, iso3166_2::from_code};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error parsing IP subnet {0}")]
    IpSubnetParse(String),
    #[error("Erro parsing IP Address {0}")]
    IpAddrParse(String),
}

#[derive(Debug, Deserialize)]
pub struct RawRecord {
    ip: String,
    alpha2: Option<String>,
    region: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Record {
    ip: IpNet,
    alpha2: Option<CountryCode>,
    region: Option<Subdivision>,
    city: Option<String>,
    postal_code: Option<String>,
}

impl TryFrom<RawRecord> for Record {
    type Error = Error;

    fn try_from(value: RawRecord) -> Result<Self, Self::Error> {
        Ok(Record {
            ip: match value.ip.parse() {
                Ok(i) => i,
                Err(_e) => return Err(Error::IpSubnetParse(value.ip.to_string())),
            },
            alpha2: if let Some(alpha2) = value.alpha2 {
                from_alpha2(&alpha2)
            } else {
                None
            },
            region: if let Some(region) = value.region {
                from_code(&region)
            } else {
                None
            },
            city: value.city,
            postal_code: value.postal_code,
        })
    }
}

impl Record {
    pub fn ip(self) -> IpNet {
        self.ip
    }

    pub fn alpha2(self) -> Option<CountryCode> {
        self.alpha2
    }

    pub fn country(self) -> Option<CountryCode> {
        self.alpha2()
    }

    pub fn region(self) -> Option<Subdivision> {
        self.region
    }

    pub fn city(self) -> Option<String> {
        self.city
    }

    pub fn postal_code(self) -> Option<String> {
        self.postal_code
    }

    pub fn check_ip(&self, input: &str) -> Result<bool, Error> {
        let input_ip: IpAddr = match input.parse() {
            Ok(i) => i,
            Err(_e) => return Err(Error::IpAddrParse(input.to_string())),
        };

        Ok(self.ip.contains(&input_ip))
    }
}

#[derive(Debug, Serialize)]
struct SerAlpha2 {
    name: String,
    alpha2: String,
    alpha3: String,
    numeric: i32,
}

#[derive(Debug, Serialize)]
struct SerRegion {
    name: String,
    subdivision_type: String,
    code: String,
    country_name: String,
    country_code: String,
}

#[derive(Debug, Serialize)]
pub struct SerRecord {
    ip: String,
    alpha2: Option<SerAlpha2>,
    region: Option<SerRegion>,
    city: Option<String>,
    postal_code: Option<String>,
}

impl From<&Record> for SerRecord {
    fn from(value: &Record) -> Self {
        SerRecord {
            ip: value.ip.to_string(),
            alpha2: value.alpha2.map(|a| SerAlpha2 {
                name: a.name.to_string(),
                alpha2: a.alpha2.to_string(),
                alpha3: a.alpha3.to_string(),
                numeric: a.numeric,
            }),
            region: value.region.map(|r| SerRegion {
                name: r.name.to_string(),
                subdivision_type: r.subdivision_type.to_string(),
                code: r.code.to_string(),
                country_name: r.country_name.to_string(),
                country_code: r.country_code.to_string(),
            }),
            city: value.city.clone(),
            postal_code: value.postal_code.clone(),
        }
    }
}
