/// Basic information about the device.
#[derive(Debug, Clone, Copy)]
pub struct PartInfo {
    pub revision: u8,
    pub part: u16,
    pub builder: u8,
    pub id: u16,
    pub customer: u8,
    pub rom_id: u8,
    pub bond: u8,
}

/// Function revision information of the device.
#[derive(Debug, Clone, Copy)]
pub struct FuncInfo {
    pub rev_ext: u8,
    pub rev_branch: u8,
    pub rev_int: u8,
    pub patch: u16,
    pub func: u8,
    pub svn_flags: u8,
    pub svn_rev: u32,
}

/// Interrupt status
#[derive(Debug, Clone, Copy)]
pub struct IntStatus {
    pub int_pending: u8,
    pub int_status: u8,
    pub ph_pending: u8,
    pub ph_status: u8,
    pub modem_pending: u8,
    pub modem_status: u8,
    pub chip_pending: u8,
    pub chip_status: u8,
}

/// Device status
#[derive(Debug, Clone, Copy)]
pub struct DeviceState {
    pub state: u8,
    pub channel: u8,
}
