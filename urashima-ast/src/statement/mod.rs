pub mod impls;

use crate::span::Spanned;

pub type Statement = Spanned<impls::Statement>;
