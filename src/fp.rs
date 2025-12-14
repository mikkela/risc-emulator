#[derive(Debug, Clone, Copy)]
pub struct IDiv {
    pub quot: u32,
    pub rem: u32,
}

pub fn fp_add(x: u32, y: u32, u: bool, v: bool) -> u32 {
    let xs = (x & 0x8000_0000) != 0;
    let (xe, x0): (u32, i32) = if !u {
        let xe = (x >> 23) & 0xFF;
        let xm = ((x & 0x007F_FFFF) << 1) | 0x0100_0000;
        let x0 = if xs { -(xm as i32) } else { xm as i32 };
        (xe, x0)
    } else {
        let xe = 150u32;
        let x0 = (((x & 0x00FF_FFFF) as i32) << 8) >> 7;
        (xe, x0)
    };

    let ys = (y & 0x8000_0000) != 0;
    let ye = (y >> 23) & 0xFF;
    let mut ym = (y & 0x007F_FFFF) << 1;
    if !u && !v {
        ym |= 0x0100_0000;
    }
    let y0 = if ys { -(ym as i32) } else { ym as i32 };

    let (e0, x3, y3): (u32, i32, i32) = if ye > xe {
        let shift = ye - xe;
        let x3 = if shift > 31 { x0 >> 31 } else { x0 >> (shift as i32) };
        (ye, x3, y0)
    } else {
        let shift = xe - ye;
        let y3 = if shift > 31 { y0 >> 31 } else { y0 >> (shift as i32) };
        (xe, x0, y3)
    };

    let xs_u = xs as u32;
    let ys_u = ys as u32;

    let sum = ((xs_u << 26) | (xs_u << 25) | ((x3 as u32) & 0x01FF_FFFF))
        .wrapping_add((ys_u << 26) | (ys_u << 25) | ((y3 as u32) & 0x01FF_FFFF));

    let s = (if (sum & (1 << 26)) != 0 { sum.wrapping_neg() } else { sum })
        .wrapping_add(1)
        & 0x07FF_FFFF;

    let mut e1 = e0.wrapping_add(1);
    let mut t3 = s >> 1;

    if (s & 0x03FF_FFFC) != 0 {
        while (t3 & (1 << 24)) == 0 {
            t3 <<= 1;
            e1 = e1.wrapping_sub(1);
        }
    } else {
        t3 <<= 24;
        e1 = e1.wrapping_sub(24);
    }

    let xn = (x & 0x7FFF_FFFF) == 0;
    let yn = (y & 0x7FFF_FFFF) == 0;

    if v {
        (((sum << 5) as i32) >> 6) as u32
    } else if xn {
        if u || yn { 0 } else { y }
    } else if yn {
        x
    } else if (t3 & 0x01FF_FFFF) == 0 || (e1 & 0x100) != 0 {
        0
    } else {
        ((sum & 0x0400_0000) << 5) | (e1 << 23) | ((t3 >> 1) & 0x007F_FFFF)
    }
}

pub fn fp_mul(x: u32, y: u32) -> u32 {
    let sign = (x ^ y) & 0x8000_0000;
    let xe = (x >> 23) & 0xFF;
    let ye = (y >> 23) & 0xFF;

    let xm = (x & 0x007F_FFFF) | 0x0080_0000;
    let ym = (y & 0x007F_FFFF) | 0x0080_0000;
    let m = (xm as u64) * (ym as u64);

    let mut e1 = xe.wrapping_add(ye).wrapping_sub(127);
    let z0: u32 = if (m & (1u64 << 47)) != 0 {
        e1 = e1.wrapping_add(1);
        (((m >> 23) as u32).wrapping_add(1)) & 0x00FF_FFFF
    } else {
        (((m >> 22) as u32).wrapping_add(1)) & 0x00FF_FFFF
    };

    if xe == 0 || ye == 0 {
        0
    } else if (e1 & 0x100) == 0 {
        sign | ((e1 & 0xFF) << 23) | (z0 >> 1)
    } else if (e1 & 0x80) == 0 {
        sign | (0xFF << 23) | (z0 >> 1)
    } else {
        0
    }
}

pub fn fp_div(x: u32, y: u32) -> u32 {
    let sign = (x ^ y) & 0x8000_0000;
    let xe = (x >> 23) & 0xFF;
    let ye = (y >> 23) & 0xFF;

    let xm = (x & 0x007F_FFFF) | 0x0080_0000;
    let ym = (y & 0x007F_FFFF) | 0x0080_0000;
    let q1 = ((xm as u64) * (1u64 << 25) / (ym as u64)) as u32;

    let mut e1 = xe.wrapping_sub(ye).wrapping_add(126);
    let q2: u32 = if (q1 & (1 << 25)) != 0 {
        e1 = e1.wrapping_add(1);
        (q1 >> 1) & 0x00FF_FFFF
    } else {
        q1 & 0x00FF_FFFF
    };
    let q3 = q2.wrapping_add(1);

    if xe == 0 {
        0
    } else if ye == 0 {
        sign | (0xFF << 23)
    } else if (e1 & 0x100) == 0 {
        sign | ((e1 & 0xFF) << 23) | (q3 >> 1)
    } else if (e1 & 0x80) == 0 {
        sign | (0xFF << 23) | (q2 >> 1)
    } else {
        0
    }
}

pub fn idiv(x: u32, y: u32, signed_div: bool) -> IDiv {
    let sign = signed_div && ((x as i32) < 0);
    let x0 = if sign { x.wrapping_neg() } else { x };

    let mut rq: u64 = x0 as u64;
    for _ in 0..32 {
        let w0 = (rq >> 31) as u32;
        let w1 = w0.wrapping_sub(y);
        if (w1 as i32) < 0 {
            rq = ((w0 as u64) << 32) | (((rq & 0x7FFF_FFFF) << 1) as u64);
        } else {
            rq = ((w1 as u64) << 32) | (((rq & 0x7FFF_FFFF) << 1) as u64) | 1;
        }
    }

    let mut d = IDiv { quot: rq as u32, rem: (rq >> 32) as u32 };
    if sign {
        d.quot = d.quot.wrapping_neg();
        if d.rem != 0 {
            d.quot = d.quot.wrapping_sub(1);
            d.rem = y.wrapping_sub(d.rem);
        }
    }
    d
}
