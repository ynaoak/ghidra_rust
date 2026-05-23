use gr_loader::{BinaryInfo, BinaryLoader};
use gr_arch::arch::create_architecture;
use gr_arch::Architecture;

use crate::listing::Listing;
use crate::reference::ReferenceManager;
use crate::symbol::{SourceType, Symbol, SymbolTable, SymbolType};

pub struct Program {
    pub name: String,
    pub arch: Box<dyn Architecture>,
    pub info: BinaryInfo,
    pub listing: Listing,
    pub symbol_table: SymbolTable,
    pub references: ReferenceManager,
}

impl Program {
    pub fn from_binary(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let info = BinaryLoader::load(path)?;
        let arch = create_architecture(info.arch)?;

        let mut symbol_table = SymbolTable::new();
        for sym in &info.symbols {
            let sym_type = match sym.kind {
                gr_loader::SymbolKind::Function => SymbolType::Function,
                gr_loader::SymbolKind::Import => SymbolType::ExternalFunction,
                gr_loader::SymbolKind::Export => SymbolType::Function,
                gr_loader::SymbolKind::Data => SymbolType::Data,
                _ => SymbolType::Label,
            };
            symbol_table.add(Symbol::new(
                sym.name.clone(),
                sym.address,
                sym_type,
                SourceType::Imported,
            ));
        }

        Ok(Self {
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            arch,
            info,
            listing: Listing::new(),
            symbol_table,
            references: ReferenceManager::new(),
        })
    }

    pub fn entry_point(&self) -> u64 {
        self.info.entry_point
    }
}
