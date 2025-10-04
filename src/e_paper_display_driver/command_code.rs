#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CommandCode {
    Psr = 0x00,
    Pwr = 0x01,
    Pof = 0x02,
    PowerOn = 0x04,
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

const PSR_DATA: [u8; 2] = [0xDF, 0x69];
const PWR_DATA: [u8; 6] = [0x0F, 0x00, 0x28, 0x2C, 0x28, 0x38];
const POF_DATA: [u8; 1] = [0x00];
const DRF_DATA: [u8; 1] = [0x00];
const CDI_DATA: [u8; 1] = [0xF7];
const TCON_DATA: [u8; 2] = [0x03, 0x03];
const TRES_DATA: [u8; 4] = [0x04, 0xB0, 0x03, 0x20];
const CMD66_DATA: [u8; 6] = [0x49, 0x55, 0x13, 0x5D, 0x05, 0x10];
const EN_BUF_DATA: [u8; 1] = [0x07];
const CCSET_DATA: [u8; 1] = [0x01];
const PWS_DATA: [u8; 1] = [0x22];
const AN_TM_DATA: [u8; 9] = [0xC0, 0x1C, 0x1C, 0xCC, 0xCC, 0xCC, 0x15, 0x15, 0x55];

const AGID_DATA: [u8; 1] = [0x10];

const BTST_P_DATA: [u8; 2] = [0xE8, 0x28];
const BOOST_VDDP_EN_DATA: [u8; 1] = [0x01];
const BTST_N_DATA: [u8; 2] = [0xE8, 0x28];
const BUCK_BOOST_VDDN_DATA: [u8; 1] = [0x01];
const TFT_VCOM_POWER_DATA: [u8; 1] = [0x02];

// JL, Inferred
const DEEP_SLEEP_DATA: [u8; 1] = [0xA5];


impl CommandCode {
    pub fn cmd(self) ->  u8 {
        self as u8
    }

    pub fn data<'a>(&self) -> Option<&'a [u8]> {
        match self {
            CommandCode::Psr => Some(&PSR_DATA),
            CommandCode::Pwr => Some(&PWR_DATA),
            CommandCode::Pof => Some(&POF_DATA),
            CommandCode::PowerOn => None, // Power On, no data
            CommandCode::BtstN => Some(&BTST_N_DATA),
            CommandCode::BtstP => Some(&BTST_P_DATA),
            CommandCode::Dtm => None, // Display the image, the data is the image itself
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