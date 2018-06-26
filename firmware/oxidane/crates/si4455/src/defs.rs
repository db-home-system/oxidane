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
