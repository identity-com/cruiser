use std::io;

pub trait ReadExt: io::Read {
    #[inline]
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    fn read_i8(&mut self) -> io::Result<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    #[inline]
    fn read_u16_le(&mut self) -> io::Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    #[inline]
    fn read_i16_le(&mut self) -> io::Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    #[inline]
    fn read_u32_le(&mut self) -> io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    #[inline]
    fn read_i32_le(&mut self) -> io::Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    #[inline]
    fn read_u64_le(&mut self) -> io::Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    #[inline]
    fn read_i64_le(&mut self) -> io::Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    #[inline]
    fn read_u128_le(&mut self) -> io::Result<u128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Ok(u128::from_le_bytes(buf))
    }

    #[inline]
    fn read_i128_le(&mut self) -> io::Result<i128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Ok(i128::from_le_bytes(buf))
    }

    #[inline]
    fn read_f32_le(&mut self) -> io::Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    #[inline]
    fn read_f64_le(&mut self) -> io::Result<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }
}
impl<R: io::Read + ?Sized> ReadExt for R {}

pub trait WriteExt: io::Write {
    #[inline]
    fn write_u8(&mut self, n: u8) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i8(&mut self, n: i8) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u16_le(&mut self, n: u16) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i16_le(&mut self, n: i16) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u32_le(&mut self, n: u32) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i32_le(&mut self, n: i32) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u64_le(&mut self, n: u64) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i64_le(&mut self, n: i64) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_f32_le(&mut self, n: f32) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_f64_le(&mut self, n: f64) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_u128_le(&mut self, n: u128) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    #[inline]
    fn write_i128_le(&mut self, n: i128) -> io::Result<()> {
        self.write_all(&n.to_le_bytes())
    }
}
impl<W: io::Write + ?Sized> WriteExt for W {}
