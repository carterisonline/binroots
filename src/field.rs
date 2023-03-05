#[derive(Default)]
pub struct BinrootsField<const N: &'static str, T> {
    pub(crate) value: T,
}

impl<const N: &'static str, T: serde::Serialize> serde::Serialize for BinrootsField<N, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<const N: &'static str, T> std::ops::Deref for BinrootsField<N, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const N: &'static str, T> std::ops::DerefMut for BinrootsField<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const N: &'static str, T> AsRef<T> for BinrootsField<N, T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<const N: &'static str, T> AsMut<T> for BinrootsField<N, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<const N: &'static str, T: std::fmt::Debug> std::fmt::Debug for BinrootsField<N, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("BinrootsField<\"{}\">", N))
            .field("value", &self.value)
            .finish()
    }
}

impl<const N: &'static str, T> BinrootsField<N, T> {
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    pub const fn name() -> &'static str {
        N
    }
}
