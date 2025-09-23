use crate::constants::data::*;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CommandCode {
    Psr = 0x00,
    Pwr = 0x01,
    Pof = 0x02,
    Pon = 0x04,
    BtstN = 0x05,
    BtstP = 0x06,
    Dtm = 0x10,
    Drf = 0x12,
    Cdi = 0x50,
    Tcon = 0x60,
    Tres = 0x61,
    AnTm = 0x74,
    Agid = 0x86,
    BuckBoostVddn = 0xB0,
    TftVcomPower = 0xB1,
    EnBuf = 0xB6,
    BoostVddpEn = 0xB7,
    Ccset = 0xE0,
    Pws = 0xE3,
    Cmd66 = 0xF0,

    // JL, Inferred
    DeepSleep = 0x07,
}

impl CommandCode {
    pub fn cmd(self) ->  u8 {
        self as u8
    }

    pub fn data<'a>(&self) -> Option<&'a [u8]> {
        match self {
            CommandCode::Psr => Some(&PSR_DATA),
            CommandCode::Pwr => Some(&PWR_DATA),
            CommandCode::Pof => Some(&POF_DATA),
            CommandCode::Pon => None,
            CommandCode::BtstN => Some(&BTST_N_DATA),
            CommandCode::BtstP => Some(&BTST_P_DATA),
            CommandCode::Dtm => None, // FIXME?
            CommandCode::Drf => Some(&DRF_DATA),
            CommandCode::Cdi => Some(&CDI_DATA),
            CommandCode::Tcon => Some(&TCON_DATA),
            CommandCode::Tres => Some(&TRES_DATA),
            CommandCode::AnTm => Some(&AN_TM_DATA),
            CommandCode::Agid => Some(&AGID_DATA),
            CommandCode::BuckBoostVddn => Some(&BUCK_BOOST_VDDN_DATA),
            CommandCode::TftVcomPower => Some(&TFT_VCOM_POWER_DATA),
            CommandCode::EnBuf => Some(&EN_BUF_DATA),
            CommandCode::BoostVddpEn => Some(&BOOST_VDDP_EN_DATA),
            CommandCode::Ccset => Some(&CCSET_DATA),
            CommandCode::Pws => Some(&PWS_DATA),
            CommandCode::Cmd66 => Some(&CMD66_DATA),
            CommandCode::DeepSleep => Some(&DEEP_SLEEP_DATA),
        }
    }
}