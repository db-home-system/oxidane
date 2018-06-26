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
