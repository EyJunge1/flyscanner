//! Decode Beast Mode-S payloads with rs1090 and maintain CPR state.

use std::collections::BTreeMap;

use crate::beast::BeastFrame;
use crate::icao::Icao24;
use rs1090::decode::cpr::{self, AircraftState as CprState, Position};
use rs1090::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("not a mode-s frame")]
    NotModeS,
    #[error("decode failed: {0}")]
    Deku(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedUpdate {
    pub icao24: Icao24,
    pub timestamp: f64,
    pub signal_level: Option<u8>,
    pub callsign: Option<String>,
    pub squawk: Option<String>,
    /// Best altitude for display (baro preferred, else geom).
    pub altitude: Option<i32>,
    pub altitude_baro: Option<i32>,
    pub altitude_geom: Option<i32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub groundspeed: Option<f64>,
    pub track: Option<f64>,
    pub vertical_rate: Option<i16>,
    pub heading: Option<f64>,
    pub ias: Option<u16>,
    pub tas: Option<u16>,
    pub nac_v: Option<u8>,
    pub nac_p: Option<u8>,
    pub nuc_p: Option<u8>,
    pub sil: Option<u8>,
    pub adsb_version: Option<u8>,
    pub geo_minus_baro: Option<i16>,
    /// ADS-B category code like `A3` (TC set + CA).
    pub category: Option<String>,
    /// Emergency state short name (`none`, `general`, …).
    pub emergency: Option<String>,
    pub spi: Option<bool>,
    pub on_ground: Option<bool>,
    pub source: MessageSource,
}

impl DecodedUpdate {
    pub(crate) fn bare(icao24: Icao24, timestamp: f64, signal_level: Option<u8>) -> Self {
        Self {
            icao24,
            timestamp,
            signal_level,
            callsign: None,
            squawk: None,
            altitude: None,
            altitude_baro: None,
            altitude_geom: None,
            latitude: None,
            longitude: None,
            groundspeed: None,
            track: None,
            vertical_rate: None,
            heading: None,
            ias: None,
            tas: None,
            nac_v: None,
            nac_p: None,
            nuc_p: None,
            sil: None,
            adsb_version: None,
            geo_minus_baro: None,
            category: None,
            emergency: None,
            spi: None,
            on_ground: None,
            source: MessageSource::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageSource {
    Adsb,
    ModeS,
    Mlat,
    Tisb,
    Other,
}

/// Stateful decoder: CPR odd/even tracking + field extraction.
pub struct AdsbDecoder {
    /// rs1090 CPR decoder requires `BTreeMap`.
    cpr: BTreeMap<ICAO, CprState>,
    reference: Option<Position>,
}

impl Default for AdsbDecoder {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AdsbDecoder {
    pub fn new(reference: Option<(f64, f64)>) -> Self {
        Self {
            cpr: BTreeMap::new(),
            reference: reference.map(|(latitude, longitude)| Position {
                latitude,
                longitude,
            }),
        }
    }

    #[expect(dead_code)]
    pub fn set_reference(&mut self, lat: f64, lon: f64) {
        self.reference = Some(Position {
            latitude: lat,
            longitude: lon,
        });
    }

    pub fn decode_frame(
        &mut self,
        frame: &BeastFrame,
        timestamp: f64,
    ) -> Result<Option<DecodedUpdate>, DecodeError> {
        let payload = frame.mode_s_payload().ok_or(DecodeError::NotModeS)?;
        self.decode_payload(payload, timestamp, Some(frame.signal_level))
    }

    pub fn decode_payload(
        &mut self,
        payload: &[u8],
        timestamp: f64,
        signal_level: Option<u8>,
    ) -> Result<Option<DecodedUpdate>, DecodeError> {
        let mut message =
            Message::try_from(payload).map_err(|e| DecodeError::Deku(e.to_string()))?;

        // Resolve CPR positions in-place (mutates ME lat/lon fields).
        match &mut message.df {
            ExtendedSquitterADSB(adsb) => {
                cpr::decode_position(
                    &mut adsb.message,
                    timestamp,
                    &adsb.icao24,
                    &mut self.cpr,
                    &mut self.reference,
                    &None,
                );
            }
            ExtendedSquitterTisB { cf, .. } => {
                cpr::decode_position(
                    &mut cf.me,
                    timestamp,
                    &cf.aa,
                    &mut self.cpr,
                    &mut self.reference,
                    &None,
                );
            }
            _ => {}
        }

        Ok(extract_update(&message, timestamp, signal_level))
    }
}

fn extract_update(
    message: &Message,
    timestamp: f64,
    signal_level: Option<u8>,
) -> Option<DecodedUpdate> {
    let icao24 = icao24_of(message)?;
    let mut update = DecodedUpdate::bare(icao24, timestamp, signal_level);

    match &message.df {
        SurveillanceIdentityReply { id, .. } => {
            update.squawk = Some(id.to_string());
            update.source = MessageSource::ModeS;
        }
        SurveillanceAltitudeReply { ac, .. } => {
            update.altitude = ac.0;
            update.altitude_baro = ac.0;
            update.source = MessageSource::ModeS;
        }
        ExtendedSquitterADSB(adsb) => {
            update.source = MessageSource::Adsb;
            apply_me(&mut update, &adsb.message);
        }
        ExtendedSquitterTisB { cf, .. } => {
            update.source = MessageSource::Tisb;
            apply_me(&mut update, &cf.me);
        }
        CommBAltitudeReply { bds, ac, .. } => {
            update.source = MessageSource::ModeS;
            update.altitude = ac.0;
            update.altitude_baro = ac.0;
            if let Some(bds20) = &bds.bds20
                && let cs = bds20.callsign.to_string()
                && !cs.contains('#')
            {
                update.callsign = Some(cs);
            }
        }
        CommBIdentityReply { bds, id, .. } => {
            update.source = MessageSource::ModeS;
            update.squawk = Some(id.to_string());
            if let Some(bds20) = &bds.bds20
                && let cs = bds20.callsign.to_string()
                && !cs.contains('#')
            {
                update.callsign = Some(cs);
            }
        }
        AllCallReply { .. } => {
            update.source = MessageSource::ModeS;
        }
        _ => return None,
    }

    Some(update)
}

fn apply_me(update: &mut DecodedUpdate, me: &ME) {
    use rs1090::decode::adsb::ME;
    use rs1090::decode::bds::bds05::{Source, SurveillanceStatus};
    use rs1090::decode::bds::bds09::AirborneVelocitySubType::{
        AirspeedSubsonic, AirspeedSupersonic, GroundSpeedDecoding,
    };
    use rs1090::decode::bds::bds09::AirspeedType::{IAS, TAS};
    use rs1090::decode::bds::bds61::EmergencyState;
    use rs1090::decode::bds::bds65::{
        ADSBVersionAirborne, ADSBVersionSurface, AircraftOperationStatus,
    };

    match me {
        ME::BDS05 { inner, .. } => {
            update.latitude = inner.latitude;
            update.longitude = inner.longitude;
            update.nuc_p = Some(inner.nuc_p);
            update.on_ground = Some(false);
            update.spi = Some(matches!(inner.ss, SurveillanceStatus::SPICondition));
            match inner.source {
                Source::Barometric => {
                    update.altitude_baro = inner.alt;
                    update.altitude = inner.alt.or(update.altitude);
                }
                Source::Gnss => {
                    update.altitude_geom = inner.alt;
                    if update.altitude_baro.is_none() {
                        update.altitude = inner.alt.or(update.altitude);
                    }
                }
            }
            if update.altitude.is_none() {
                update.altitude = inner.alt;
            }
        }
        ME::BDS06 { inner, .. } => {
            update.latitude = inner.latitude;
            update.longitude = inner.longitude;
            update.track = inner.track;
            update.groundspeed = inner.groundspeed;
            update.on_ground = Some(true);
        }
        ME::BDS08 { id, inner, .. } => {
            if let cs = inner.callsign.to_string()
                && !cs.contains('#')
            {
                update.callsign = Some(cs);
            }
            let set = match *id {
                4 => 'A',
                3 => 'B',
                2 => 'C',
                1 => 'D',
                _ => '?',
            };
            update.category = Some(format!("{set}{}", inner.ca));
        }
        ME::BDS09(bds09) => {
            update.vertical_rate = bds09.vertical_rate;
            update.nac_v = Some(bds09.nac_v);
            update.geo_minus_baro = bds09.geo_minus_baro;
            match &bds09.velocity {
                GroundSpeedDecoding(spd) => {
                    update.groundspeed = Some(spd.groundspeed);
                    update.track = Some(spd.track);
                }
                AirspeedSubsonic(spd) => {
                    update.heading = spd.heading;
                    match spd.airspeed_type {
                        IAS => update.ias = spd.airspeed,
                        TAS => update.tas = spd.airspeed,
                    }
                }
                AirspeedSupersonic(spd) => {
                    update.heading = spd.heading.map(f64::from);
                    match spd.airspeed_type {
                        IAS => update.ias = spd.airspeed,
                        TAS => update.tas = spd.airspeed,
                    }
                }
                _ => {}
            }
        }
        ME::BDS61(bds61) => {
            update.squawk = Some(bds61.squawk.to_string());
            update.emergency = Some(
                match bds61.emergency_state {
                    EmergencyState::None => "none",
                    EmergencyState::General => "general",
                    EmergencyState::Medical => "lifeguard",
                    EmergencyState::MinimumFuel => "minfuel",
                    EmergencyState::NoCommunication => "nordo",
                    EmergencyState::UnlawfulInterference => "unlawful",
                    EmergencyState::DownedAircraft => "downed",
                    EmergencyState::Reserved => "reserved",
                }
                .into(),
            );
        }
        ME::BDS65(status) => match status {
            AircraftOperationStatus::Airborne(airborne) => {
                update.adsb_version = Some(airborne.version.version_number());
                match &airborne.version {
                    ADSBVersionAirborne::DOC9871AppendixB(v1) => {
                        update.nac_p = Some(v1.nac_p);
                        update.sil = Some(v1.sil);
                    }
                    ADSBVersionAirborne::DOC9871AppendixC(v2) => {
                        update.nac_p = Some(v2.nac_p);
                        update.sil = Some(v2.sil);
                    }
                    ADSBVersionAirborne::DOC9871AppendixA(_)
                    | ADSBVersionAirborne::Reserved { .. } => {}
                }
            }
            AircraftOperationStatus::Surface(surface) => {
                update.adsb_version = Some(surface.version.version_number());
                update.on_ground = Some(true);
                match &surface.version {
                    ADSBVersionSurface::DOC9871AppendixB(v1) => {
                        update.nac_p = Some(v1.nac_p);
                        update.sil = Some(v1.sil);
                    }
                    ADSBVersionSurface::DOC9871AppendixC(v2) => {
                        update.nac_p = Some(v2.nac_p);
                        update.sil = Some(v2.sil);
                    }
                    ADSBVersionSurface::DOC9871AppendixA(_)
                    | ADSBVersionSurface::Reserved { .. } => {}
                }
            }
            AircraftOperationStatus::Reserved { .. } => {}
        },
        _ => {}
    }
}

fn icao24_of(message: &Message) -> Option<Icao24> {
    let raw = match &message.df {
        ShortAirAirSurveillance { ap, .. } => ap.to_string(),
        SurveillanceAltitudeReply { ap, .. } => ap.to_string(),
        SurveillanceIdentityReply { ap, .. } => ap.to_string(),
        AllCallReply { icao, .. } => icao.to_string(),
        LongAirAirSurveillance { ap, .. } => ap.to_string(),
        ExtendedSquitterADSB(ADSB { icao24, .. }) => icao24.to_string(),
        ExtendedSquitterTisB { cf, .. } => cf.aa.to_string(),
        CommBAltitudeReply { ap, .. } => ap.to_string(),
        CommBIdentityReply { ap, .. } => ap.to_string(),
        _ => return None,
    };
    Icao24::parse(&raw)
}
