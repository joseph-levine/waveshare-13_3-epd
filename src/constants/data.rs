pub(super) const PSR_DATA: [u8; 2] = [0xDF, 0x69];
pub(super) const PWR_DATA: [u8; 6] = [0x0F, 0x00, 0x28, 0x2C, 0x28, 0x38];
pub(super) const POF_DATA: [u8; 1] = [0x00];
pub(super) const DRF_DATA: [u8; 1] = [0x00];
pub(super) const CDI_DATA: [u8; 1] = [0xF7];
pub(super) const TCON_DATA: [u8; 2] = [0x03, 0x03];
pub(super) const TRES_DATA: [u8; 4] = [0x04, 0xB0, 0x03, 0x20];
pub(super) const CMD66_DATA: [u8; 6] = [0x49, 0x55, 0x13, 0x5D, 0x05, 0x10];
pub(super) const EN_BUF_DATA: [u8; 1] = [0x07];
pub(super) const CCSET_DATA: [u8; 1] = [0x01];
pub(super) const PWS_DATA: [u8; 1] = [0x22];
pub(super) const AN_TM_DATA: [u8; 9] = [0xC0, 0x1C, 0x1C, 0xCC, 0xCC, 0xCC, 0x15, 0x15, 0x55];

pub(super) const AGID_DATA: [u8; 1] = [0x10];

pub(super) const BTST_P_DATA: [u8; 2] = [0xE8, 0x28];
pub(super) const BOOST_VDDP_EN_DATA: [u8; 1] = [0x01];
pub(super) const BTST_N_DATA: [u8; 2] = [0xE8, 0x28];
pub(super) const BUCK_BOOST_VDDN_DATA: [u8; 1] = [0x01];
pub(super) const TFT_VCOM_POWER_DATA: [u8; 1] = [0x02];

// JL, Inferred
pub(super) const DEEP_SLEEP_DATA: [u8; 1] = [0xA5];
