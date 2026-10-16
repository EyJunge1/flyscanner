export type MessageSource = 'adsb' | 'modes' | 'mlat' | 'tisb' | 'other';

export interface TrackPoint {
	ts: number;
	lat: number;
	lon: number;
	alt: number | null;
}

export interface Aircraft {
	icao24: string;
	first_seen: number;
	last_seen: number;
	callsign: string | null;
	squawk: string | null;
	altitude: number | null;
	altitude_baro?: number | null;
	altitude_geom?: number | null;
	latitude: number | null;
	longitude: number | null;
	groundspeed: number | null;
	track: number | null;
	vertical_rate: number | null;
	heading: number | null;
	ias?: number | null;
	tas?: number | null;
	nac_v?: number | null;
	nac_p?: number | null;
	nuc_p?: number | null;
	sil?: number | null;
	adsb_version?: number | null;
	geo_minus_baro?: number | null;
	category?: string | null;
	emergency?: string | null;
	spi?: boolean | null;
	on_ground?: boolean | null;
	rssi: number | null;
	messages: number;
	source: MessageSource;
	registration: string | null;
	typecode: string | null;
	type_long?: string | null;
	year?: string | null;
	operator: string | null;
	db_flags?: number;
	track_history: TrackPoint[];
}

export interface StatsSummary {
	messages_total: number;
	aircraft_current: number;
	message_rate: number;
	max_range_nm: number;
	decode_errors: number;
}

export interface AppConfig {
	receiver_lat: number;
	receiver_lon: number;
}

export interface HistorySnapshot {
	ts: number;
	aircraft: Aircraft[];
}

export type WsMessage =
	| { type: 'snapshot'; aircraft: Aircraft[] }
	| { type: 'update'; aircraft: Aircraft[] }
	| { type: 'delta'; upsert: Aircraft[]; remove: string[] }
	| {
			type: 'stats';
			messages_total: number;
			aircraft_current: number;
			message_rate: number;
			max_range_nm: number;
	  }
	| { type: 'pong' };
