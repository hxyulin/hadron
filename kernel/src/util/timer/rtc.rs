use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use x86_64::instructions::port::Port;

pub type UtcTime = DateTime<Utc>;

fn rtc_read(reg: u8) -> u8 {
    let mut port_70: Port<u8> = Port::new(0x70);
    let mut port_71: Port<u8> = Port::new(0x71);

    unsafe {
        // Write register index
        port_70.write(0x80 | reg);
        // Read register C
        port_71.read()
    }
}

fn is_bcd() -> bool {
    rtc_read(0x0B) & 0x04 == 0
}

fn bcd_to_binary(bcd: u8) -> u8 {
    (bcd & 0x0F) + ((bcd >> 4) * 10)
}

fn update_in_progress() -> bool {
    rtc_read(0x0A) & 0x80 != 0
}

pub fn read_time() -> UtcTime {
    while update_in_progress() {}

    let mut seconds = rtc_read(0x00);
    let mut minutes = rtc_read(0x02);
    let mut hours = rtc_read(0x04);
    let mut day = rtc_read(0x07);
    let mut month = rtc_read(0x08);
    let mut year = rtc_read(0x09);

    if is_bcd() {
        seconds = bcd_to_binary(seconds);
        minutes = bcd_to_binary(minutes);
        hours = bcd_to_binary(hours);
        day = bcd_to_binary(day);
        month = bcd_to_binary(month);
        year = bcd_to_binary(year);
    }

    let naive_date = NaiveDate::from_ymd_opt(year as i32 + 2000, month as u32, day as u32).unwrap();
    let naive_time = NaiveTime::from_hms_opt(hours as u32, minutes as u32, seconds as u32).unwrap();
    let naive_date_time = NaiveDateTime::new(naive_date, naive_time);
    UtcTime::from_naive_utc_and_offset(naive_date_time, Utc)
}
