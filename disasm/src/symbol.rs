use std::borrow::Cow;
use std::fmt;

#[derive(Eq, PartialEq)]
pub struct Symbol<'data> {
    /// The demangled name of the symbol.
    name: Cow<'data, str>,

    /// The virtual address of the symbol.
    addr: u64,

    /// The starting byte position of the symbol in its binary.
    bpos: usize,

    /// The length of the symbol in its binary.
    blen: usize,

    /// Possible source language of the symbol.
    lang: SymbolLang,

    /// Where this symbol is from.
    source: SymbolSource,

    /// The type of this symbol.
    type_: SymbolType,
}

impl<'data> Symbol<'data> {
    pub fn new(
        name: impl Into<Cow<'data, str>>,
        addr: u64,
        bpos: usize,
        blen: usize,
        type_: SymbolType,
        source: SymbolSource,
        mut lang: SymbolLang,
    ) -> Self {
        use cpp_demangle::Symbol as CppSymbol;
        use rustc_demangle::try_demangle;

        // FIXME demangle C names (e.g. stdcall and fastcall naming conventions).
        let name = name.into();
        let demangled_name = try_demangle(&*name)
            .map(|n| {
                lang.update(SymbolLang::Rust);
                Cow::from(format!("{}", n))
            })
            .or_else(|_| {
                CppSymbol::new(name.as_bytes()).map(|s| {
                    lang.update(SymbolLang::Cpp);
                    Cow::from(s.to_string())
                })
            })
            .unwrap_or_else(|_| name);

        Symbol {
            name: demangled_name,
            addr,
            bpos,
            blen,
            type_,
            source,
            lang,
        }
    }

    pub fn address(&self) -> u64 {
        self.addr
    }

    pub fn offset(&self) -> usize {
        self.bpos
    }

    pub fn end(&self) -> usize {
        self.bpos + self.blen
    }

    pub fn size(&self) -> usize {
        self.blen
    }

    pub fn name(&self) -> &str {
        &*self.name
    }

    pub fn lang(&self) -> SymbolLang {
        self.lang
    }

    pub fn source(&self) -> SymbolSource {
        self.source
    }

    pub fn type_(&self) -> SymbolType {
        self.type_
    }

    /// Converts this into a static owned symbol.
    pub fn owned(self) -> Symbol<'static> {
        Symbol {
            name: Cow::from(self.name.into_owned()),
            addr: self.addr,
            bpos: self.bpos,
            blen: self.blen,
            lang: self.lang,
            source: self.source,
            type_: self.type_,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SymbolType {
    Function,

    /// Static variable.
    Static,
}

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            SymbolType::Function => "function",
            SymbolType::Static => "static",
        };
        write!(f, "{}", t)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SymbolLang {
    Rust,
    Cpp,
    C,
    Unknown,
}

impl SymbolLang {
    /// Update the language if it is unknown.
    fn update(&mut self, new_lang: SymbolLang) {
        if *self == SymbolLang::Unknown {
            *self = new_lang
        }
    }
}

impl fmt::Display for SymbolLang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            SymbolLang::Rust => "Rust",
            SymbolLang::Cpp => "C++",
            SymbolLang::C => "C",
            SymbolLang::Unknown => "unknown",
        };
        write!(f, "{}", t)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SymbolSource {
    /// The symbol was stored as part of the object file's (elf, mach-o, archive, pe, ...)
    /// structure.
    Object,

    /// The symbol was stored in DWARF debug data.
    Dwarf,

    /// The symbol was found in a PDB.
    PDB,
}

impl fmt::Display for SymbolSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            SymbolSource::Object => "object",
            SymbolSource::Dwarf => "DWARF",
            SymbolSource::PDB => "PDB",
        };
        write!(f, "{}", t)
    }
}
