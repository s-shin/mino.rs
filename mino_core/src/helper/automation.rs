use std::error::Error;
use std::str::FromStr;

pub enum Move {
    Left(usize),
    LeftEnd,
    Right(usize),
    RightEnd,
    SoftDrop(usize),
    FirmDrop,
    HardDrop,
    RotateCw(usize),
    RotateCcw(usize),
    Hold,
}

impl FromStr for Move {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.splitn(2, '=');
        let mv_str = iter.next().unwrap_or("").to_lowercase();
        let count = iter.next().unwrap_or("1").parse::<usize>()?;
        Ok(match &*mv_str {
            "left" | "l" => Move::Left(count),
            "leftend" | "le" => Move::LeftEnd,
            "right" | "r" => Move::Right(count),
            "rightend" | "re" => Move::RightEnd,
            "down" | "d" | "softdrop" | "sd" => Move::SoftDrop(count),
            "firmdrop" | "fd" => Move::FirmDrop,
            "harddrop" | "hd" => Move::HardDrop,
            "rotatecw" | "cw" => Move::RotateCw(count),
            "rotateccw" | "ccw" => Move::RotateCcw(count),
            "hold" | "h" => Move::Hold,
            _ => return Err("invalid move string".into()),
        })
    }
}
