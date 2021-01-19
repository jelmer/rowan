use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    arc::{Arc, HeaderSlice, ThinArc},
    green::SyntaxKind,
    TextSize,
};

type Repr = HeaderSlice<SyntaxKind, [u8]>;
type ReprThin = HeaderSlice<SyntaxKind, [u8; 0]>;
#[repr(transparent)]
pub struct GreenTokenData {
    data: ReprThin,
}

/// Leaf node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenToken {
    ptr: ThinArc<SyntaxKind, u8>,
}

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    #[inline]
    fn to_owned(&self) -> GreenToken {
        unsafe {
            let green = GreenToken::from_raw(ptr::NonNull::from(self));
            let green = ManuallyDrop::new(green);
            GreenToken::clone(&green)
        }
    }
}

impl Borrow<GreenTokenData> for GreenToken {
    #[inline]
    fn borrow(&self) -> &GreenTokenData {
        &*self
    }
}

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .finish()
    }
}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = &*self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenTokenData {
    /// Kind of this Token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header
    }

    /// Text of this Token.
    #[inline]
    pub fn text(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.data.slice()) }
    }

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn text_len(&self) -> TextSize {
        TextSize::of(self.text())
    }
}

impl GreenToken {
    /// Creates new Token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &str) -> GreenToken {
        let ptr = ThinArc::from_header_and_iter(kind, text.bytes());
        GreenToken { ptr }
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {
        let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
        let arc = mem::transmute::<Arc<ReprThin>, ThinArc<SyntaxKind, u8>>(arc);
        GreenToken { ptr: arc }
    }
}

impl ops::Deref for GreenToken {
    type Target = GreenTokenData;

    #[inline]
    fn deref(&self) -> &GreenTokenData {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTokenData>(repr)
        }
    }
}
