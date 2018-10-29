//Table 11-24. CC26_FCFG1_MMAP1 Registers

// Offset      Acronym                             Register Name                               Section
// A0h         MISC_CONF_1              Misc configurations                         Section 11.4.1.1
// ...
// 164h        FLASH_NUMBER             Flash information                           Section 11.4.1.20
// 16Ch        FLASH_COORDINATE         Flash information                           Section 11.4.1.21
// ...
// 294h        USER_ID                  User Identification.                        Section 11.4.1.33
// ...
// 2E8h        MAC_BLE_0                MAC BLE Address 0                           Section 11.4.1.37
// 2ECh        MAC_BLE_1                MAC BLE Address 1                           Section 11.4.1.38
// 2F0h        MAC_15_4_0               MAC IEEE 802.15.4 Address 0                 Section 11.4.1.39
// 2F4h        MAC_15_4_1               MAC IEEE 802.15.4 Address 1                 Section 11.4.1.40
// ...
// 30Ch        MISC_TRIM                Miscellaneous Trim Parameters               Section 11.4.1.42
// ...
// 31Ch        FCFG1_REVISION           Factory Configuration (FCFG1) Revision      Section 11.4.1.45
// 320h        MISC_OTP_DATA            Misc OTP Data                               Section 11.4.1.46
//...
// 344h        IOCONF                   IO Configuration                            Section 11.4.1.47
// ...
// 35Ch        SOC_ADC_ABS_GAIN         AUX_ADC Gain in Absolute Reference Mode     Section 11.4.1.50
// 360h        SOC_ADC_REL_GAIN         AUX_ADC Gain in Relative Reference Mode     Section 11.4.1.51
// 368h        SOC_ADC_OFFSET_INT       AUX_ADC Temp Offsets in Abs Ref Mode        Section 11.4.1.52
// ...
// 38Ch        OSC_CONF                 OSC Configuration                           Section 11.4.1.59
// ...
// 39Ch        PWD_CURR_20C             Power Down Current Control 20C              Section 11.4.1.62
// 3A0h        PWD_CURR_35C             Power Down Current Control 35C              Section 11.4.1.63
// 3A4h        PWD_CURR_50C             Power Down Current Control 50C              Section 11.4.1.64
// 3A8h        PWD_CURR_65C             Power Down Current Control 65C              Section 11.4.1.65
// 3ACh        PWD_CURR_80C             Power Down Current Control 80C              Section 11.4.1.66
// 3B0h        PWD_CURR_95C             Power Down Current Control 95C              Section 11.4.1.67
// 3B4h        PWD_CURR_110C            Power Down Current Control 110C             Section 11.4.1.68
// 3B8h        PWD_CURR_125C            Power Down Current Control 125C             Section 11.4.1.69
// 3D0h        SHDW_DIE_ID_0            Shadow of [JTAG_TAP::EFUSE:DIE_ID_0.*]      Section 11.4.1.70
// 3D4h        SHDW_DIE_ID_1            Shadow of [JTAG_TAP::EFUSE:DIE_ID_1.*]      Section 11.4.1.71
// 3D8h        SHDW_DIE_ID_2            Shadow of [JTAG_TAP::EFUSE:DIE_ID_2.*]      Section 11.4.1.72
// 3DCh        SHDW_DIE_ID_3            Shadow of [JTAG_TAP::EFUSE:DIE_ID_3.*]      Section 11.4.1.73

use kernel::common::registers::ReadOnly;
use kernel::common::StaticRef;

use memory_map::FCFG1_BASE;

pub const REG: StaticRef<Registers> = unsafe { StaticRef::new(FCFG1_BASE as *const Registers) };

#[repr(C)]
pub struct Registers {
    _offset0: [u8; 0xA0],
    misc_conf: ReadOnly<u32>,
    _offset1: [u8; 0xC0],
    flash_number: ReadOnly<u32>,
    _offset2: [u8; 0x4],
    flash_coordinate: ReadOnly<u32>,
    _offset3: [u8; 0x124],
    user_id: ReadOnly<u32>,
    _offset4: [u8; 0x50],
    mac_ble0: ReadOnly<u32>,
    mac_ble1: ReadOnly<u32>,
    mac_15_4_0: ReadOnly<u32>,
    mac_15_4_1: ReadOnly<u32>,
    _offset5: [u8; 0x14],
    misc_trim: ReadOnly<u32>,
    _offset6: [u8; 0x0C],
    fcfg_rev: ReadOnly<u32>,
    misc_otp_data: ReadOnly<u32>,
    _offset7: [u8; 0x20],
    pub ioconf: ReadOnly<u32>,
    _offset8: [u8; 0x14],
    pub adc_abs_gain: ReadOnly<u32, AdcGain::Register>,
    pub adc_rel_gain: ReadOnly<u32, AdcGain::Register>,
    pub adc_offset: ReadOnly<u32, AdcOffset::Register>,
    osc_conf: ReadOnly<u32>,
}

register_bitfields![
    u32,
    AdcGain[
        VALUE OFFSET(0) NUMBITS(16)
    ],
    AdcOffset[
        REL OFFSET(16) NUMBITS(8),
        ABS OFFSET(0) NUMBITS(8)
    ]
];
