use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct TNum : u8 {
        const WHOLE     = 0b0000_0001;
        const NAT       = 0b0000_0010;
        const INT       = 0b0000_0100;
        const ALG       = 0b0000_1000;
        const REAL      = 0b0001_0000;
        const COMPLEX   = 0b0010_0000;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct TText : u8 {
        const ASCII     = 0b0000_0001;
        const CHAR      = 0b0000_0010;
        const GRAPHEME  = 0b0000_0100;
        const STR       = 0b0000_1000;
    }
}

impl TNum {
    pub fn whole() -> Self {
        Self::WHOLE | Self::nat()
    }

    pub fn nat() -> Self {
        Self::NAT | Self::int()
    }

    pub fn int() -> Self {
        Self::INT | Self::alg()
    }

    pub fn alg() -> Self {
        Self::ALG | Self::real()
    }

    pub fn real() -> Self {
        Self::REAL | Self::complex()
    }

    pub fn complex() -> Self {
        Self::COMPLEX
    }
}

impl TText {
    pub fn ascii() -> Self {
        Self::ASCII | Self::char()
    }

    pub fn char() -> Self {
        Self::CHAR | Self::grapheme()
    }

    pub fn grapheme() -> Self {
        Self::GRAPHEME | Self::str()
    }

    pub fn str() -> Self {
        Self::STR
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Num(TNum),
    Text(TText),
    Symbol
}
