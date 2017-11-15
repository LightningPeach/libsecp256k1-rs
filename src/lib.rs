#[macro_use]
mod field;
#[macro_use]
mod group;
mod scalar;
mod ecmult;
mod ecdsa;

pub use field::Field;
pub use group::Affine;
pub use scalar::Scalar;

pub use ecmult::ECMultContext;

pub const TAG_PUBKEY_EVEN: u8 = 0x02;
pub const TAG_PUBKEY_ODD: u8 = 0x03;
pub const TAG_PUBKEY_UNCOMPRESSED: u8 = 0x04;
pub const TAG_PUBKEY_HYBRID_EVEN: u8 = 0x06;
pub const TAG_PUBKEY_HYBRID_ODD: u8 = 0x07;

pub struct PublicKey(pub [u8; 64]);
pub struct Signature(pub [u8; 64]);
pub struct RecoverableSignature(pub Signature, pub u8);

impl PublicKey {
    pub fn load(&self) -> Affine {
        let mut ge = Affine::default();
        let (mut x, mut y) = (Field::default(), Field::default());

        let mut data = [0u8; 32];
        for i in 0..32 {
            data[i] = self.0[i];
        }
        x.set_b32(&data);
        for i in 0..32 {
            data[i] = self.0[i+32];
        }
        y.set_b32(&data);

        ge.set_xy(&x, &y);
        assert!(!ge.x.is_zero());

        ge
    }
}

impl Signature {
    pub fn load(&self) -> (Scalar, Scalar) {
        let mut r = Scalar::default();
        let mut s = Scalar::default();

        let mut data = [0u8; 32];
        for i in 0..32 {
            data[i] = self.0[i];
        }
        r.set_b32(&data);
        for i in 0..32 {
            data[i] = self.0[i+32];
        }
        s.set_b32(&data);

        (r, s)
    }
}

pub fn public_key_parse(p: &[u8; 65]) -> Option<Affine> {
    use {TAG_PUBKEY_HYBRID_EVEN, TAG_PUBKEY_HYBRID_ODD};

    if !(p[0] == 0x04 || p[0] == 0x06 || p[0] == 0x07) {
        return None;
    }
    let mut x = Field::default();
    let mut y = Field::default();
    let mut data = [0u8; 32];
    for i in 0..32 {
        data[i] = p[i+1];
    }
    if !x.set_b32(&data) {
        return None;
    }
    for i in 0..32 {
        data[i] = p[i+33];
    }
    if !y.set_b32(&data) {
        return None;
    }
    let mut elem = Affine::default();
    elem.set_xy(&x, &y);
    if (p[0] == TAG_PUBKEY_HYBRID_EVEN || p[0] == TAG_PUBKEY_HYBRID_ODD) &&
        (y.is_odd() != (p[0] == TAG_PUBKEY_HYBRID_ODD))
    {
        return None;
    }
    if elem.is_valid_var() {
        return Some(elem);
    } else {
        return None;
    }
}

pub fn public_key_serialize(elem: &Affine) -> Option<[u8; 65]> {
    if elem.is_infinity() {
        return None;
    }

    let mut ret = [0u8; 65];
    let mut elem = elem.clone();

    elem.x.normalize_var();
    elem.y.normalize_var();
    let d = elem.x.b32();
    for i in 0..32 {
        ret[1+i] = d[i];
    }
    let d = elem.y.b32();
    for i in 0..32 {
        ret[33+i] = d[i];
    }
    ret[0] = TAG_PUBKEY_UNCOMPRESSED;

    Some(ret)
}
