/*
There are 144 squares. They are numbered as follows:

                                   Top
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  0  |  1  |  2  |  3  |  4  |  5  |  6  |  7  |
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  8  |  9  |  10 |  11 |  12 |  13 |  14 |  15 |
            +-----+-----x-----f-----g-----h-----y-----+-----+
            |  16 |  17 |                       |  22 |  23 |
            +-----+-----e                       i-----+-----+
            |  24 |  25 |                       |  30 |  31 |
            +-----+-----d          Hole         j-----+-----+
            |  32 |  33 |                       |  38 |  39 |
            +-----+-----c                       k-----+-----+
            |  40 |  41 |                       |  46 |  47 |
            +-----+-----w-----b-----a-----l-----z-----+-----+
            |  48 |  49 |  50 |  51 |  52 |  53 |  54 |  55 |
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  56 |  57 |  58 |  59 |  60 |  61 |  62 |  63 |
            +-----+-----+-----+-----+-----+-----+-----+-----+

        -w-               -x-               -y-               -z-
       /   \             /   \     Hole    /   \             /   \
a-----b     c-----d-----e     f-----g-----h     i-----j-----k     l-----a
|  26 | 132 |  90 | 106 | 140 |  42 |  34 | 136 |  98 |  82 | 128 |  18 |
+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
|  28 | 134 |  92 | 108 | 142 |  44 |  36 | 138 | 100 |  84 | 130 |  20 |
+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
|  29 | 135 |  93 | 109 | 143 |  45 |  37 | 139 | 101 |  85 | 131 |  21 |
+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
|  27 | 133 |  91 | 107 | 141 |  43 |  35 | 137 |  99 |  83 | 129 |  19 |
A-----B     C-----D-----E     F-----G-----H     I-----J-----K     L-----A
       \   /             \   /             \   /             \   /
        -W-               -X-               -Y-               -X-

                                  Bottom
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  64 |  65 |  66 |  67 |  68 |  69 |  70 |  71 |
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  72 |  73 |  74 |  75 |  76 |  77 |  78 |  79 |
            +-----+-----X-----F-----G-----H-----Y-----+-----+
            |  80 |  81 |                       |  86 |  87 |
            +-----+-----E                       I-----+-----+
            |  88 |  89 |                       |  94 |  95 |
            +-----+-----D          Hole         J-----+-----+
            |  96 |  97 |                       | 102 | 103 |
            +-----+-----C                       K-----+-----+
            | 104 | 105 |                       | 110 | 111 |
            +-----+-----W-----B-----A-----L-----Z-----+-----+
            | 112 | 113 | 114 | 115 | 116 | 117 | 118 | 119 |
            +-----+-----+-----+-----+-----+-----+-----+-----+
            | 120 | 121 | 122 | 123 | 124 | 125 | 126 | 127 |
            +-----+-----+-----+-----+-----+-----+-----+-----+

The numbering has been chosen such that the following symmetries are easier to compute:
 - Flip along x-axis i.e. flip top-bottom as in above diagrams
 - Flip along y-axis i.e. flip left-right as in above diagrams
 - Flip along z-axis i.e. swap the top and bottom of the board
 - Flip along the xy-diagonal i.e. flip along the Top and Bottom along the nw-se diagonal and the flip the Hole left-right on the line connecting `y` to `Y` or equivalently the line connecting `w` to `W`.
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosType {
    Top,        // A number in the "Top" part of the diagram.
    Bottom,     // A number in the "Bottom" part of the diagram.
    HoleTop,    // A number in the "Hole" part of the diagram between 18 and 45 inclusive.
    HoleBottom, // A number in the "Hole" part of the diagram between 82 and 109 inclusive.
    HolePent,   // A number in the "Hole" part of the diagram between 128 and 143 inclusive.
}

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub n: u8,
}

impl Pos {
    pub fn get_type(&self) -> PosType {
        assert!(self.n < 144);
        if self.n < 64 {
            if 18 <= self.n && self.n < 46 && self.n - 2 & 4 == 0 {
                PosType::HoleTop
            } else {
                PosType::Top
            }
        } else if self.n < 128 {
            if 82 <= self.n && self.n < 110 && self.n - 2 & 4 == 0 {
                PosType::HoleBottom
            } else {
                PosType::Bottom
            }
        } else {
            PosType::HolePent
        }
    }
}

// The 11 positions whose orbits under x-flips, y-flips, z-flips, and xy-flips give all positions
#[derive(Debug, Clone, Copy)]
pub enum Orbit {
    P0,
    P1,
    P2,
    P3,
    P9,
    P10,
    P11,
    P42,
    P44,
    P140,
    P142,
}

impl Orbit {
    pub fn pos(&self) -> Pos {
        match self {
            Orbit::P0 => Pos { n: 0 },
            Orbit::P1 => Pos { n: 1 },
            Orbit::P2 => Pos { n: 2 },
            Orbit::P3 => Pos { n: 3 },
            Orbit::P9 => Pos { n: 9 },
            Orbit::P10 => Pos { n: 10 },
            Orbit::P11 => Pos { n: 11 },
            Orbit::P42 => Pos { n: 42 },
            Orbit::P44 => Pos { n: 44 },
            Orbit::P140 => Pos { n: 140 },
            Orbit::P142 => Pos { n: 142 },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Symmetry {
    // Apply these 3
    flip_x: bool,
    flip_y: bool,
    flip_z: bool,
    // Followed by this one
    flip_xy: bool,
}

impl Symmetry {
    pub fn inverse(self) -> Self {
        Self {
            flip_x: self.flip_y,
            flip_y: self.flip_x,
            flip_z: self.flip_z,
            flip_xy: self.flip_xy,
        }
    }
}

impl Pos {
    pub fn flip_x(&self) -> Pos {
        match self.get_type() {
            PosType::Bottom | PosType::Top => {
                let a = self.n & 0b00000111;
                let b = self.n & 0b11111000;
                Pos { n: (7 - a) | b }
            }
            PosType::HoleTop | PosType::HoleBottom => Pos {
                n: ((self.n - 18) ^ 8) + 18,
            },
            PosType::HolePent => Pos { n: self.n ^ 4 },
        }
    }

    pub fn flip_y(&self) -> Pos {
        match self.get_type() {
            PosType::Bottom | PosType::Top => {
                let a = self.n & 0b11000111;
                let b = self.n & 0b00111000;
                Pos {
                    n: a | ((7 - (b >> 3)) << 3),
                }
            }
            PosType::HoleTop | PosType::HoleBottom => Pos {
                n: ((self.n - 18) ^ 16) + 18,
            },
            PosType::HolePent => Pos { n: self.n ^ 8 },
        }
    }

    pub fn flip_z(&self) -> Pos {
        match self.get_type() {
            PosType::Bottom | PosType::Top => Pos { n: self.n ^ 64 },
            PosType::HoleTop | PosType::HoleBottom | PosType::HolePent => Pos { n: self.n ^ 1 },
        }
    }

    pub fn flip_xy(&self) -> Pos {
        match self.get_type() {
            PosType::Bottom | PosType::Top => {
                let a = self.n & 0b00000111;
                let b = self.n & 0b00111000;
                let c = self.n & 0b11000000;
                Pos {
                    n: (a << 3) | (b >> 3) | c,
                }
            }
            PosType::HoleTop | PosType::HoleBottom => {
                let n = self.n - 18;
                let m = n & 0b00111111;
                let mut f = 64u8;
                if 8 <= m && m < 20 {
                    f |= 8 | 16; // flip x and y too
                }
                Pos { n: (n ^ f) + 18 }
            }
            PosType::HolePent => {
                if 132 <= self.n && self.n < 138 {
                    // flip x and y
                    Pos {
                        n: self.n ^ (4 | 8),
                    }
                } else {
                    *self
                }
            }
        }
    }

    pub fn apply_symmetry(mut self, symmetry: Symmetry) -> Self {
        if symmetry.flip_x {
            self = self.flip_x();
        }
        if symmetry.flip_y {
            self = self.flip_y();
        }
        if symmetry.flip_z {
            self = self.flip_z();
        }
        if symmetry.flip_xy {
            self = self.flip_xy();
        }
        self
    }

    // Find a symmetry and orbit representative such that applying the symmetry to self gives the orbit representative
    pub fn orbit(self) -> (Symmetry, Orbit) {
        let t = self.get_type();
        let (x, y) = match t {
            PosType::Top | PosType::Bottom => (self.n & 0b00000111, (self.n & 0b00111000) >> 3),
            PosType::HoleTop => todo!(),
            PosType::HoleBottom => todo!(),
            PosType::HolePent => todo!(),
        };
        // `false` if on the top half, `true` if on the bottom half.
        let z = match t {
            PosType::Top => false,
            PosType::Bottom => true,
            PosType::HoleTop | PosType::HoleBottom | PosType::HolePent => self.n & 1 == 1,
        };

        println!("x={:?} y={:?} z={:?}", x, y, z);

        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pos_type() {
        // Top
        assert_eq!(Pos { n: 0 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 17 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 18 }.get_type(), PosType::HoleTop);
        assert_eq!(Pos { n: 21 }.get_type(), PosType::HoleTop);
        assert_eq!(Pos { n: 22 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 25 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 26 }.get_type(), PosType::HoleTop);
        assert_eq!(Pos { n: 29 }.get_type(), PosType::HoleTop);
        assert_eq!(Pos { n: 30 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 33 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 34 }.get_type(), PosType::HoleTop);
        assert_eq!(Pos { n: 37 }.get_type(), PosType::HoleTop);
        assert_eq!(Pos { n: 38 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 41 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 42 }.get_type(), PosType::HoleTop);
        assert_eq!(Pos { n: 45 }.get_type(), PosType::HoleTop);
        assert_eq!(Pos { n: 46 }.get_type(), PosType::Top);
        assert_eq!(Pos { n: 63 }.get_type(), PosType::Top);

        // Bottom
        assert_eq!(Pos { n: 64 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 81 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 82 }.get_type(), PosType::HoleBottom);
        assert_eq!(Pos { n: 85 }.get_type(), PosType::HoleBottom);
        assert_eq!(Pos { n: 86 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 89 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 90 }.get_type(), PosType::HoleBottom);
        assert_eq!(Pos { n: 93 }.get_type(), PosType::HoleBottom);
        assert_eq!(Pos { n: 94 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 97 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 98 }.get_type(), PosType::HoleBottom);
        assert_eq!(Pos { n: 101 }.get_type(), PosType::HoleBottom);
        assert_eq!(Pos { n: 102 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 105 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 106 }.get_type(), PosType::HoleBottom);
        assert_eq!(Pos { n: 109 }.get_type(), PosType::HoleBottom);
        assert_eq!(Pos { n: 110 }.get_type(), PosType::Bottom);
        assert_eq!(Pos { n: 127 }.get_type(), PosType::Bottom);

        // Pent
        assert_eq!(Pos { n: 128 }.get_type(), PosType::HolePent);
        assert_eq!(Pos { n: 143 }.get_type(), PosType::HolePent);
    }
}
