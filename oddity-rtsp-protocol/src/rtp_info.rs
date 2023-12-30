use std::fmt;
use std::str::FromStr;

use super::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpInfo {
    pub url: String,
    pub seq: Option<u16>,
    pub rtptime: Option<u32>,
}

impl RtpInfo {
    #[must_use]
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            seq: None,
            rtptime: None,
        }
    }

    #[must_use]
    pub fn new_with_timing(url: &str, seq: u16, rtptime: u32) -> Self {
        Self {
            url: url.to_string(),
            seq: Some(seq),
            rtptime: Some(rtptime),
        }
    }

    #[must_use]
    pub const fn with_seq(mut self, seq: u16) -> Self {
        self.seq = Some(seq);
        self
    }

    #[must_use]
    pub const fn with_rtptime(mut self, rtptime: u32) -> Self {
        self.rtptime = Some(rtptime);
        self
    }
}

impl fmt::Display for RtpInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "url={}", self.url)?;
        if let Some(seq) = self.seq {
            write!(f, ";seq={seq}")?;
        }
        if let Some(rtptime) = self.rtptime {
            write!(f, ";rtptime={rtptime}")?;
        }
        Ok(())
    }
}

impl FromStr for RtpInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_parameter(part: &str, rtp_info: &mut RtpInfo) -> Result<(), Error> {
            if let Some(seq) = part.strip_prefix("seq=") {
                let seq = seq.parse().map_err(|_| Error::RtpInfoParameterInvalid {
                    value: part.to_string(),
                })?;
                rtp_info.seq = Some(seq);
                Ok(())
            } else if let Some(rtptime) = part.strip_prefix("rtptime=") {
                let rtptime = rtptime
                    .parse()
                    .map_err(|_| Error::RtpInfoParameterInvalid {
                        value: part.to_string(),
                    })?;
                rtp_info.rtptime = Some(rtptime);
                Ok(())
            } else {
                Err(Error::RtpInfoParameterUnknown {
                    value: part.to_string(),
                })
            }
        }

        let mut parts = s.split(';');
        if let Some(url) = parts.next() {
            if let Some(url) = url.strip_prefix("url=") {
                let mut rtp_info = Self::new(url);
                if let Some(part) = parts.next() {
                    parse_parameter(part, &mut rtp_info)?;
                    if let Some(part) = parts.next() {
                        parse_parameter(part, &mut rtp_info)?;
                        parts.next().map_or_else(
                            || Ok(rtp_info),
                            |part| {
                                Err(Error::RtpInfoParameterUnexpected {
                                    value: part.to_string(),
                                })
                            },
                        )
                    } else {
                        Ok(rtp_info)
                    }
                } else {
                    Ok(rtp_info)
                }
            } else {
                Err(Error::RtpInfoParameterUnknown {
                    value: url.to_string(),
                })
            }
        } else {
            Err(Error::RtpInfoUrlMissing {
                value: s.to_string(),
            })
        }
    }
}
