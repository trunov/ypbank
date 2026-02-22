
struct YPBankBinRecord {
    // Поля структуры, представляющей содержимое записи
}

impl YPBankBinRecord {
    // Парсит из любого источника, реализующего трейт Read
    pub fn from_read<R: std::io::Read>(r: &mut R) -> Result<Self> {
        todo!()
    }

    // Записывает отчёт в любой приёмник, реализующий трейт Write
    pub fn write_to<W: std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        todo!()
    }
}