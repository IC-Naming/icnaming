use std::cell::RefCell;

pub trait StableState: Sized {
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: Vec<u8>) -> Result<Self, String>;
}

pub fn decode_store_or_default<T>(bytes: Option<Vec<u8>>) -> Result<RefCell<T>, String>
where
    T: Default + StableState,
{
    let inner = if let Some(bytes) = bytes {
        T::decode(bytes)?
    } else {
        T::default()
    };
    Ok(RefCell::new(inner))
}

pub fn decode_store<T>(bytes: Vec<u8>) -> Result<RefCell<T>, String>
where
    T: Default + StableState,
{
    let inner = T::decode(bytes)?;
    Ok(RefCell::new(inner))
}

#[cfg(test)]
mod stable_tests;
