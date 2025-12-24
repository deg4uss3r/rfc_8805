use std::net::IpAddr;

use ipnet::IpNet;
use rust_iso3166::{CountryCode, from_alpha2, iso3166_2::Subdivision, iso3166_2::from_code};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
/// Errors that can happen when parsing the RFC 8805 format
pub enum Error {
    #[error("Error parsing IP subnet {0}")]
    /// There was an error parsing the String into a [`ipnet::IpNet`] type
    IpSubnetParse(String),
    #[error("Error parsing IP Address {0}")]
    /// There was an error parsing the String into a [`std::net::IpAddr`] type
    IpAddrParse(String),
}

#[derive(Debug, Deserialize)]
/// A raw Record type, this is used mainly for deserialization or if you wanted to use your
/// own types adn convert manually from Strings
/// 
/// A typical use case is to use `Record::try_from(rawRecord)` to convert this into a better format
/// and take advantage of more information in the subnet and country code libraries utilized in the [`Record`] type
pub struct RawRecord {
    /// Representation of the IP subnet advertised
    ///
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.1
    ip: String,
    /// If exists the 2 letter country code identifier (e.g. US)
    ///
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.2
    alpha2: Option<String>,
    /// If exists the region in ISO 3166-2 format
    /// 
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.3
    region: Option<String>,
    /// If exists the city name in UTF8 character encoding (excluding the `,` character)
    ///
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.4
    city: Option<String>,
    /// If exists the postal code in UTF8 character encoding (excluding the `,` character)
    /// This filed is deprecated for newer geofeeds but might be present in order formats
    /// 
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.5
    postal_code: Option<String>,
}

#[derive(Debug, Clone)]
/// A enhanced view of a Record of RFC8805 format
pub struct Record {
    /// Representation of the IP subnet advertised
    ///
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.1
    ip: IpNet,
    /// If exists the 2 letter country code identifier (e.g. US)
    ///
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.2
    alpha2: Option<CountryCode>,
    /// If exists the region in ISO 3166-2 format
    /// 
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.3
    region: Option<Subdivision>,
    /// If exists the city name in UTF8 character encoding (excluding the `,` character)
    ///
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.4
    city: Option<String>,
    /// If exists the postal code in UTF8 character encoding (excluding the `,` character)
    /// This filed is deprecated for newer geofeeds but might be present in order formats
    /// 
    /// https://datatracker.ietf.org/doc/html/rfc8805#section-2.1.1.5
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
    /// Get the advertised IP prefix of the record
    pub fn ip(self) -> IpNet {
        self.ip
    }

    /// Get the country code, in alpha 2, of the record
    pub fn alpha2(self) -> Option<CountryCode> {
        self.alpha2
    }

    /// Get the region (formally called country) of the record
    pub fn country(self) -> Option<CountryCode> {
        self.alpha2()
    }

    /// Get the region of the record
    pub fn region(self) -> Option<Subdivision> {
        self.region
    }

    /// Get the city of the record
    pub fn city(self) -> Option<String> {
        self.city
    }

    /// Get the postal code of the record 
    /// Postal codes have been deprecated but could still exist in feeds
    pub fn postal_code(self) -> Option<String> {
        self.postal_code
    }

    /// Checks if the IP exists in the IP Prefix of the record
    pub fn check_ip(&self, input: &str) -> Result<bool, Error> {
        let input_ip: IpAddr = match input.parse() {
            Ok(i) => i,
            Err(_e) => return Err(Error::IpAddrParse(input.to_string())),
        };

        Ok(self.ip.contains(&input_ip))
    }
}

#[derive(Debug, Serialize)]
/// Easily serializable representation of [`rust_iso3166::CountryCode`]
struct SerAlpha2 {
    name: String,
    alpha2: String,
    alpha3: String,
    numeric: i32,
}

#[derive(Debug, Serialize)]
/// Easily serializable representation of [`rust_iso3166::iso3166_2::Subdivision`]
struct SerRegion {
    name: String,
    subdivision_type: String,
    code: String,
    country_name: String,
    country_code: String,
}

#[derive(Debug, Serialize)]
/// A easily serializable version of [`Record`] mainly used for CLI applications to output in `json`
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
