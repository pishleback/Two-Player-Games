/*
There are 144 squares. They are numbered as follows:

                                   Top
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  56 |  57 |  58 |  59 |  60 |  61 |  62 |  63 |
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  48 |  49 |  50 |  51 |  52 |  53 |  54 |  55 |
            +-----+-----x-----f-----g-----h-----y-----+-----+
            |  40 |  41 |                       |  46 |  47 |
            +-----+-----e                       i-----+-----+
            |  32 |  33 |                       |  38 |  39 |
            +-----+-----d          Hole         j-----+-----+
            |  24 |  25 |                       |  30 |  31 |
            +-----+-----c                       k-----+-----+
            |  16 |  17 |                       |  22 |  23 |
            +-----+-----w-----b-----a-----l-----z-----+-----+
            |  8  |  9  |  10 |  11 |  12 |  13 |  14 |  15 |
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  0  |  1  |  2  |  3  |  4  |  5  |  6  |  7  |
            +-----+-----+-----+-----+-----+-----+-----+-----+

        -w-               -x-               -y-               -z-
       /   \             /   \     Hole    /   \             /   \
a-----b     c-----d-----e     f-----g-----h     i-----j-----k     l-----a
|  42 | 140 | 106 |  90 | 132 |  26 |  18 | 128 |  82 |  98 | 136 |  34 |
+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
|  44 | 142 | 108 |  92 | 134 |  28 |  20 | 130 |  84 | 100 | 138 |  36 |
+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
|  45 | 143 | 109 |  93 | 135 |  29 |  21 | 131 |  85 | 101 | 139 |  37 |
+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+
|  43 | 141 | 107 |  91 | 133 |  27 |  19 | 129 |  83 |  99 | 137 |  35 |
A-----B     C-----D-----E     F-----G-----H     I-----J-----K     L-----A
       \   /             \   /             \   /             \   /
        -W-               -X-               -Y-               -X-

                                  Bottom
            +-----+-----+-----+-----+-----+-----+-----+-----+
            | 120 | 121 | 122 | 123 | 124 | 125 | 126 | 127 |
            +-----+-----+-----+-----+-----+-----+-----+-----+
            | 112 | 113 | 114 | 115 | 116 | 117 | 118 | 119 |
            +-----+-----X-----F-----G-----H-----Y-----+-----+
            | 104 | 105 |                       | 110 | 111 |
            +-----+-----E                       I-----+-----+
            |  96 |  97 |                       | 102 | 103 |
            +-----+-----D          Hole         J-----+-----+
            |  88 |  89 |                       |  94 |  95 |
            +-----+-----C                       K-----+-----+
            |  80 |  81 |                       |  86 |  87 |
            +-----+-----W-----B-----A-----L-----Z-----+-----+
            |  72 |  73 |  74 |  75 |  76 |  77 |  78 |  79 |
            +-----+-----+-----+-----+-----+-----+-----+-----+
            |  64 |  65 |  66 |  67 |  68 |  69 |  70 |  71 |
            +-----+-----+-----+-----+-----+-----+-----+-----+

The numbering has been chosen such that the following symmetries are easier to compute:
 - Flip along x-axis i.e. flip top-bottom as in above diagrams
 - Flip along y-axis i.e. flip left-right as in above diagrams
 - Flip along z-axis i.e. swap the top and bottom of the board
 - Flip along the xy-diagonal i.e. flip along the Top and Bottom along the nw-se diagonal and the flip the Hole left-right on the line connecting `y` to `Y` or equivalently the line connecting `w` to `W`.
*/

use glam::Vec3;

use crate::wormhole::board_render::BoardParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosType {
    Top,        // A number in the "Top" part of the diagram.
    Bottom,     // A number in the "Bottom" part of the diagram.
    HoleTop,    // A number in the "Hole" part of the diagram between 18 and 45 inclusive.
    HoleBottom, // A number in the "Hole" part of the diagram between 82 and 109 inclusive.
    HolePent,   // A number in the "Hole" part of the diagram between 128 and 143 inclusive.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    n: u8,
}

impl Pos {
    pub fn new(idx: u8) -> Self {
        debug_assert!(idx < 144);
        Self { n: idx }
    }

    pub fn all() -> Vec<Self> {
        (0..144).map(|i| Self::new(i)).collect()
    }

    pub fn u8_idx(self) -> u8 {
        self.n
    }

    pub fn idx(self) -> usize {
        self.n as usize
    }

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

// The 11 positions whose orbits under x-flips, y-flips, z-flips, and xy-flips give all positions.
#[derive(Debug, Clone, Copy)]
pub enum OrbitFull {
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

impl OrbitFull {
    #[allow(unused)]
    pub fn pos(&self) -> Pos {
        match self {
            OrbitFull::P0 => Pos { n: 0 },
            OrbitFull::P1 => Pos { n: 1 },
            OrbitFull::P2 => Pos { n: 2 },
            OrbitFull::P3 => Pos { n: 3 },
            OrbitFull::P9 => Pos { n: 9 },
            OrbitFull::P10 => Pos { n: 10 },
            OrbitFull::P11 => Pos { n: 11 },
            OrbitFull::P42 => Pos { n: 42 },
            OrbitFull::P44 => Pos { n: 44 },
            OrbitFull::P140 => Pos { n: 140 },
            OrbitFull::P142 => Pos { n: 142 },
        }
    }
}

// The 18 positions whose orbits under x-flips, y-flips, and z-flips give all positions.
#[derive(Debug, Clone, Copy)]
pub enum OrbitCardinal {
    P0,
    P1,
    P2,
    P3,
    P8,
    P9,
    P10,
    P11,
    P16,
    P17,
    P24,
    P25,
    P42,
    P44,
    P106,
    P108,
    P140,
    P142,
}

impl OrbitCardinal {
    #[allow(unused)]
    pub fn pos(&self) -> Pos {
        match self {
            OrbitCardinal::P0 => Pos { n: 0 },
            OrbitCardinal::P1 => Pos { n: 1 },
            OrbitCardinal::P2 => Pos { n: 2 },
            OrbitCardinal::P3 => Pos { n: 3 },
            OrbitCardinal::P8 => Pos { n: 8 },
            OrbitCardinal::P9 => Pos { n: 9 },
            OrbitCardinal::P10 => Pos { n: 10 },
            OrbitCardinal::P11 => Pos { n: 11 },
            OrbitCardinal::P16 => Pos { n: 16 },
            OrbitCardinal::P17 => Pos { n: 17 },
            OrbitCardinal::P24 => Pos { n: 24 },
            OrbitCardinal::P25 => Pos { n: 25 },
            OrbitCardinal::P42 => Pos { n: 42 },
            OrbitCardinal::P44 => Pos { n: 44 },
            OrbitCardinal::P106 => Pos { n: 106 },
            OrbitCardinal::P108 => Pos { n: 108 },
            OrbitCardinal::P140 => Pos { n: 140 },
            OrbitCardinal::P142 => Pos { n: 142 },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Symmetry {
    // Apply these 3
    pub flip_x: bool,
    pub flip_y: bool,
    pub flip_z: bool,
    // Followed by this one
    pub flip_xy: bool,
}

impl Symmetry {
    pub fn identity() -> Self {
        Self {
            flip_x: false,
            flip_y: false,
            flip_z: false,
            flip_xy: false,
        }
    }

    #[allow(unused)]
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
    pub fn full_symmetry_and_orbit(self) -> (Symmetry, OrbitFull) {
        let t = self.get_type();
        let (mut x, mut y) = match t {
            PosType::Top | PosType::Bottom => (self.n & 0b00000111, (self.n & 0b00111000) >> 3),
            PosType::HoleTop => {
                [(4, 5), (3, 5), (4, 2), (3, 2)][(((self.n - 18) & 0b11111000) >> 3) as usize]
            }
            PosType::HoleBottom => {
                [(5, 4), (2, 4), (5, 3), (2, 3)][(((self.n - 82) & 0b11111000) >> 3) as usize]
            }
            PosType::HolePent => {
                [(5, 5), (2, 5), (5, 2), (2, 2)][(((self.n - 128) & 0b11111100) >> 2) as usize]
            }
        };
        // `false` if on the top half, `true` if on the bottom half.
        let z = match t {
            PosType::Top => false,
            PosType::Bottom => true,
            PosType::HoleTop | PosType::HoleBottom | PosType::HolePent => self.n & 1 == 1,
        };

        let mut symmetry = Symmetry::identity();
        let mut orbit = self;
        if z {
            symmetry.flip_z = true;
            orbit = orbit.flip_z();
        }
        if y >= 4 {
            symmetry.flip_y = true;
            orbit = orbit.flip_y();
            y = 7 - y;
        }
        if x >= 4 {
            symmetry.flip_x = true;
            orbit = orbit.flip_x();
            x = 7 - x;
        }
        if x < y {
            symmetry.flip_xy = true;
            orbit = orbit.flip_xy();
        }

        debug_assert_eq!(orbit, self.apply_symmetry(symmetry));

        let orbit = match orbit.n {
            0 => OrbitFull::P0,
            1 => OrbitFull::P1,
            2 => OrbitFull::P2,
            3 => OrbitFull::P3,
            9 => OrbitFull::P9,
            10 => OrbitFull::P10,
            11 => OrbitFull::P11,
            42 => OrbitFull::P42,
            44 => OrbitFull::P44,
            140 => OrbitFull::P140,
            142 => OrbitFull::P142,
            _ => unreachable!(),
        };

        (symmetry, orbit)
    }

    // Find a symmetry and orbit representative such that applying the symmetry to self gives the orbit representative
    pub fn cardinal_symmetry_and_orbit(self) -> (Symmetry, OrbitCardinal) {
        let (mut sym, orb) = self.full_symmetry_and_orbit();
        match sym.flip_xy {
            false => {
                let c_orb = match orb {
                    OrbitFull::P0 => OrbitCardinal::P0,
                    OrbitFull::P1 => OrbitCardinal::P1,
                    OrbitFull::P2 => OrbitCardinal::P2,
                    OrbitFull::P3 => OrbitCardinal::P3,
                    OrbitFull::P9 => OrbitCardinal::P9,
                    OrbitFull::P10 => OrbitCardinal::P10,
                    OrbitFull::P11 => OrbitCardinal::P11,
                    OrbitFull::P42 => OrbitCardinal::P42,
                    OrbitFull::P44 => OrbitCardinal::P44,
                    OrbitFull::P140 => OrbitCardinal::P140,
                    OrbitFull::P142 => OrbitCardinal::P142,
                };
                (sym, c_orb)
            }
            true => {
                sym.flip_xy = !sym.flip_xy;
                let c_orb = match orb {
                    OrbitFull::P1 => OrbitCardinal::P8,
                    OrbitFull::P2 => OrbitCardinal::P16,
                    OrbitFull::P3 => OrbitCardinal::P24,
                    OrbitFull::P10 => OrbitCardinal::P17,
                    OrbitFull::P11 => OrbitCardinal::P25,
                    OrbitFull::P42 => OrbitCardinal::P106,
                    OrbitFull::P44 => OrbitCardinal::P108,
                    _ => unreachable!(),
                };
                (sym, c_orb)
            }
        }
    }
}

pub struct PosCoords {
    pub origin: Vec3,
    pub up: Vec3,
    pub vec: Vec3,
}

pub fn all_pos_coords(board_params: &BoardParams) -> [PosCoords; 144] {
    let dsq = board_params.side_length / 8.0;
    let dx = Vec3 {
        x: dsq,
        y: 0.0,
        z: 0.0,
    };
    let dy = Vec3 {
        x: 0.0,
        y: dsq,
        z: 0.0,
    };
    let face_offset = Vec3 {
        x: 0.0,
        y: 0.0,
        z: board_params.face_offset,
    };
    let p0_middle = Vec3 {
        x: -board_params.side_length * 0.5 + dsq * 0.5,
        y: -board_params.side_length * 0.5 + dsq * 0.5,
        z: 0.0,
    };

    struct OriginAndUp {
        origin: Vec3,
        up: Vec3,
    }

    let tube_origin_up = |phi: f32, psi: f32| -> OriginAndUp {
        // compute a point on the wormhole tube and its normal based on the angle phi around in the xy-plane and the angle psi between the xy-plane and z
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();
        let cos_psi = psi.cos();
        let sin_psi = psi.sin();
        let sqrt5 = (5.0 as f32).sqrt();
        let origin = {
            let w = (sqrt5 - sin_psi * board_params.hole_offset) * (board_params.side_length / 8.0);
            let z = cos_psi * board_params.face_offset;
            Vec3 {
                x: -sin_phi * w,
                y: cos_phi * w,
                z,
            }
        };
        let up = {
            let w = sin_psi * board_params.face_offset;
            let z = (cos_psi * board_params.hole_offset) * (board_params.side_length / 8.0);
            Vec3 {
                x: sin_phi * w,
                y: -cos_phi * w,
                z,
            }
        };
        OriginAndUp { origin, up }
    };

    const PI: f32 = std::f32::consts::PI;
    const TAU: f32 = std::f32::consts::TAU;

    let p44_p45_p36_p37_origin_up = tube_origin_up(0.0, 0.5 * PI);
    let p42_p34_origin_up = tube_origin_up(0.0, 0.15 * PI);
    let p42_origin_up = tube_origin_up(1.0 * TAU / 24.0, 0.15 * PI);
    let p44_origin_up = tube_origin_up(1.0 * TAU / 24.0, 0.4 * PI);
    let p140_origin_up = tube_origin_up(3.0 * TAU / 24.0, 0.08 * PI);
    let p142_origin_up = tube_origin_up(3.0 * TAU / 24.0, 0.4 * PI);
    let p143_origin_up = tube_origin_up(3.0 * TAU / 24.0, 0.6 * PI);
    let p106_origin_up = tube_origin_up(5.0 * TAU / 24.0, 0.15 * PI);
    let p108_origin_up = tube_origin_up(5.0 * TAU / 24.0, 0.4 * PI);
    let p109_origin_up = tube_origin_up(5.0 * TAU / 24.0, 0.6 * PI);

    std::array::from_fn(|i| {
        let pos = Pos::new(i as u8);
        let (sym, orb) = pos.cardinal_symmetry_and_orbit();
        let (mut origin, mut up, mut vec, len) = match orb {
            OrbitCardinal::P0 => (
                p0_middle + face_offset,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P1 => (
                p0_middle + face_offset + dx,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P2 => (
                p0_middle + face_offset + 2.0 * dx,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P3 => (
                p0_middle + face_offset + 3.0 * dx,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P8 => (
                p0_middle + face_offset + dy,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P9 => (
                p0_middle + face_offset + dx + dy,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P10 => (
                p0_middle + face_offset + 2.0 * dx + dy,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P11 => (
                p0_middle + face_offset + 3.0 * dx + 0.9 * dy,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P16 => (
                p0_middle + face_offset + 2.0 * dy,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P17 => (
                p0_middle + face_offset + dx + 2.0 * dy,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P24 => (
                p0_middle + face_offset + 3.0 * dy,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
            ),
            OrbitCardinal::P25 => (
                p0_middle + face_offset + 0.9 * dx + 3.0 * dy,
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 0.0),
                0.9,
            ),
            OrbitCardinal::P42 => (
                p42_origin_up.origin,
                p42_origin_up.up,
                p42_p34_origin_up.origin - p42_origin_up.origin,
                0.9,
            ),
            OrbitCardinal::P44 => (
                p44_origin_up.origin,
                p44_origin_up.up,
                p44_p45_p36_p37_origin_up.origin - p44_origin_up.origin,
                0.8,
            ),
            OrbitCardinal::P140 => (
                p140_origin_up.origin,
                p140_origin_up.up,
                p44_origin_up.origin - p140_origin_up.origin,
                1.0,
            ),
            OrbitCardinal::P142 => (
                p142_origin_up.origin,
                p142_origin_up.up,
                p143_origin_up.origin - p142_origin_up.origin,
                0.8,
            ),
            OrbitCardinal::P106 => (
                p106_origin_up.origin,
                p106_origin_up.up,
                p108_origin_up.origin - p106_origin_up.origin,
                0.9,
            ),
            OrbitCardinal::P108 => (
                p108_origin_up.origin,
                p108_origin_up.up,
                p109_origin_up.origin - p108_origin_up.origin,
                0.8,
            ),
        };

        let flip_x = |v: Vec3| Vec3 {
            x: -v.x,
            y: v.y,
            z: v.z,
        };
        let flip_y = |v: Vec3| Vec3 {
            x: v.x,
            y: -v.y,
            z: v.z,
        };
        let flip_z = |v: Vec3| Vec3 {
            x: v.x,
            y: v.y,
            z: -v.z,
        };

        if sym.flip_x {
            up = flip_x(up);
            vec = flip_z(flip_y(vec));
            origin = flip_x(origin);
        }

        if sym.flip_y {
            up = flip_y(up);
            vec = flip_y(vec);
            origin = flip_y(origin);
        }

        if sym.flip_z {
            up = flip_z(up);
            vec = flip_x(flip_y(vec));
            origin = flip_z(origin);
        }

        let up = up.normalize();
        let vec = len * (vec - vec.project_onto(up)).normalize(); //make vec perp to up

        PosCoords { origin, up, vec }
    })
}

pub struct Slides {
    pub next: Pos,
    pub after: Vec<Slides>,
}

pub struct MovesLookup {
    white_forward: [Vec<Pos>; 144],
    black_forward: [Vec<Pos>; 144],
    cardinal_adjacent: [Vec<Pos>; 144],
    diagonal_adjacent: [Vec<Pos>; 144],
    continuations: [[Vec<Pos>; 144]; 144],
    knight_moves: [Vec<Pos>; 144],
    cardinal_slides: [Vec<Slides>; 144],
    diagonal_slides: [Vec<Slides>; 144],
}

impl MovesLookup {
    pub fn new() -> Self {
        let white_forward = std::array::from_fn(|i| {
            let mut pos = Pos::new(i as u8);
            let flip_z = pos.full_symmetry_and_orbit().0.flip_z;
            if flip_z {
                pos = pos.flip_x();
            }
            let (sym, orb) = pos.cardinal_symmetry_and_orbit();
            let mut forward = match (orb, sym.flip_x) {
                (OrbitCardinal::P0, false) => vec![Pos::new(1)],
                (OrbitCardinal::P1, false) => vec![Pos::new(2)],
                (OrbitCardinal::P2, false) => vec![Pos::new(3)],
                (OrbitCardinal::P3, false) => vec![Pos::new(4)],
                (OrbitCardinal::P3, true) => vec![Pos::new(5)],
                (OrbitCardinal::P2, true) => vec![Pos::new(6)],
                (OrbitCardinal::P1, true) => vec![Pos::new(7)],
                (OrbitCardinal::P0, true) => vec![],
                (OrbitCardinal::P8, false) => vec![Pos::new(9)],
                (OrbitCardinal::P9, false) => vec![Pos::new(10)],
                (OrbitCardinal::P10, false) => vec![Pos::new(11)],
                (OrbitCardinal::P11, false) => vec![Pos::new(12)],
                (OrbitCardinal::P11, true) => vec![Pos::new(13)],
                (OrbitCardinal::P10, true) => vec![Pos::new(14)],
                (OrbitCardinal::P9, true) => vec![Pos::new(15)],
                (OrbitCardinal::P8, true) => vec![],
                (OrbitCardinal::P16, false) => vec![Pos::new(17)],
                (OrbitCardinal::P17, false) => vec![Pos::new(140)],
                (OrbitCardinal::P17, true) => vec![Pos::new(23)],
                (OrbitCardinal::P16, true) => vec![],
                (OrbitCardinal::P24, false) => vec![Pos::new(25)],
                (OrbitCardinal::P25, false) => vec![Pos::new(106)],
                (OrbitCardinal::P25, true) => vec![Pos::new(31)],
                (OrbitCardinal::P24, true) => vec![],
                (OrbitCardinal::P42, false) => vec![Pos::new(34)],
                (OrbitCardinal::P140, false) => vec![Pos::new(42), Pos::new(142)],
                (OrbitCardinal::P106, false) => vec![Pos::new(108)],
                (OrbitCardinal::P44, false) => vec![Pos::new(45), Pos::new(36)],
                (OrbitCardinal::P142, false) => vec![Pos::new(143)],
                (OrbitCardinal::P108, false) => vec![Pos::new(109)],
                (OrbitCardinal::P42, true) => vec![Pos::new(136)],
                (OrbitCardinal::P140, true) => vec![Pos::new(22)],
                (OrbitCardinal::P106, true) => vec![Pos::new(30)],
                (OrbitCardinal::P44, true) => vec![Pos::new(34), Pos::new(138)],
                (OrbitCardinal::P142, true) => vec![Pos::new(136)],
                (OrbitCardinal::P108, true) => vec![Pos::new(98)],
            };
            if sym.flip_y {
                forward = forward.into_iter().map(|p| p.flip_y()).collect();
            }
            if sym.flip_z {
                forward = forward.into_iter().map(|p| p.flip_z()).collect();
            }
            if flip_z {
                forward = forward.into_iter().map(|p| p.flip_x()).collect();
            }
            forward
        });

        let black_forward = std::array::from_fn(|i| {
            white_forward[Pos::new(i as u8).flip_x().idx()]
                .iter()
                .map(|p| p.flip_x())
                .collect()
        });

        let cardinal_adjacent = std::array::from_fn(|i| {
            let pos = Pos::new(i as u8);
            let (sym, orb) = pos.full_symmetry_and_orbit();
            let mut adj = match orb {
                OrbitFull::P0 => vec![Pos::new(1), Pos::new(8)],
                OrbitFull::P1 => vec![Pos::new(0), Pos::new(2), Pos::new(9)],
                OrbitFull::P2 => vec![Pos::new(1), Pos::new(3), Pos::new(10)],
                OrbitFull::P3 => vec![Pos::new(2), Pos::new(4), Pos::new(11)],
                OrbitFull::P9 => vec![Pos::new(1), Pos::new(8), Pos::new(10), Pos::new(17)],
                OrbitFull::P10 => vec![Pos::new(2), Pos::new(9), Pos::new(11), Pos::new(140)],
                OrbitFull::P11 => vec![Pos::new(3), Pos::new(10), Pos::new(12), Pos::new(42)],
                OrbitFull::P42 => vec![Pos::new(11), Pos::new(34), Pos::new(44), Pos::new(140)],
                OrbitFull::P44 => vec![Pos::new(36), Pos::new(42), Pos::new(45), Pos::new(142)],
                OrbitFull::P140 => vec![
                    Pos::new(10),
                    Pos::new(17),
                    Pos::new(42),
                    Pos::new(106),
                    Pos::new(142),
                ],
                OrbitFull::P142 => vec![Pos::new(44), Pos::new(108), Pos::new(140), Pos::new(143)],
            };
            if sym.flip_xy {
                adj = adj.into_iter().map(|p| p.flip_xy()).collect();
            }
            if sym.flip_x {
                adj = adj.into_iter().map(|p| p.flip_x()).collect();
            }
            if sym.flip_y {
                adj = adj.into_iter().map(|p| p.flip_y()).collect();
            }
            if sym.flip_z {
                adj = adj.into_iter().map(|p| p.flip_z()).collect();
            }
            adj
        });

        let diagonal_adjacent = std::array::from_fn(|i| {
            let pos = Pos::new(i as u8);
            let (sym, orb) = pos.full_symmetry_and_orbit();
            let mut adj = match orb {
                OrbitFull::P0 => vec![Pos::new(9)],
                OrbitFull::P1 => vec![Pos::new(8), Pos::new(10)],
                OrbitFull::P2 => vec![Pos::new(9), Pos::new(11)],
                OrbitFull::P3 => vec![Pos::new(10), Pos::new(12)],
                OrbitFull::P9 => vec![Pos::new(0), Pos::new(2), Pos::new(16), Pos::new(140)],
                OrbitFull::P10 => vec![Pos::new(1), Pos::new(3), Pos::new(17), Pos::new(42)],
                OrbitFull::P11 => vec![Pos::new(2), Pos::new(4), Pos::new(140), Pos::new(34)],
                OrbitFull::P42 => vec![Pos::new(10), Pos::new(12), Pos::new(36), Pos::new(142)],
                OrbitFull::P44 => vec![Pos::new(34), Pos::new(37), Pos::new(140), Pos::new(143)],
                OrbitFull::P140 => vec![
                    Pos::new(9),
                    Pos::new(11),
                    Pos::new(25),
                    Pos::new(44),
                    Pos::new(108),
                ],
                OrbitFull::P142 => vec![Pos::new(42), Pos::new(45), Pos::new(106), Pos::new(109)],
            };
            if sym.flip_xy {
                adj = adj.into_iter().map(|p| p.flip_xy()).collect();
            }
            if sym.flip_x {
                adj = adj.into_iter().map(|p| p.flip_x()).collect();
            }
            if sym.flip_y {
                adj = adj.into_iter().map(|p| p.flip_y()).collect();
            }
            if sym.flip_z {
                adj = adj.into_iter().map(|p| p.flip_z()).collect();
            }
            adj
        });

        let continuations = std::array::from_fn(|a_idx| {
            let a_pos = Pos::new(a_idx as u8);
            std::array::from_fn(|b_idx| {
                let b_pos = Pos::new(b_idx as u8);
                let (sym, b_orb) = b_pos.full_symmetry_and_orbit();
                let a_orb = a_pos.apply_symmetry(sym);
                let mut c_poses = match b_orb {
                    OrbitFull::P0 => vec![],
                    OrbitFull::P1 => match a_orb.u8_idx() {
                        0 => {
                            vec![Pos::new(2)]
                        }
                        2 => {
                            vec![Pos::new(0)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P2 => match a_orb.u8_idx() {
                        1 => {
                            vec![Pos::new(3)]
                        }
                        3 => {
                            vec![Pos::new(1)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P3 => match a_orb.u8_idx() {
                        2 => {
                            vec![Pos::new(4)]
                        }
                        4 => {
                            vec![Pos::new(2)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P9 => match a_orb.u8_idx() {
                        0 => {
                            vec![Pos::new(140)]
                        }
                        1 => {
                            vec![Pos::new(17)]
                        }
                        2 => {
                            vec![Pos::new(16)]
                        }
                        8 => {
                            vec![Pos::new(10)]
                        }
                        10 => {
                            vec![Pos::new(8)]
                        }
                        16 => {
                            vec![Pos::new(2)]
                        }
                        17 => {
                            vec![Pos::new(1)]
                        }
                        140 => {
                            vec![Pos::new(0)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P10 => match a_orb.u8_idx() {
                        1 => {
                            vec![Pos::new(42)]
                        }
                        2 => {
                            vec![Pos::new(140)]
                        }
                        3 => {
                            vec![Pos::new(17)]
                        }
                        9 => {
                            vec![Pos::new(11)]
                        }
                        11 => {
                            vec![Pos::new(9)]
                        }
                        17 => {
                            vec![Pos::new(3)]
                        }
                        140 => {
                            vec![Pos::new(2)]
                        }
                        42 => {
                            vec![Pos::new(1)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P11 => match a_orb.u8_idx() {
                        2 => {
                            vec![Pos::new(34)]
                        }
                        3 => {
                            vec![Pos::new(42)]
                        }
                        4 => {
                            vec![Pos::new(140)]
                        }
                        10 => {
                            vec![Pos::new(12)]
                        }
                        12 => {
                            vec![Pos::new(10)]
                        }
                        140 => {
                            vec![Pos::new(4)]
                        }
                        42 => {
                            vec![Pos::new(3)]
                        }
                        34 => {
                            vec![Pos::new(2)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P42 => match a_orb.u8_idx() {
                        10 => {
                            vec![Pos::new(36)]
                        }
                        11 => {
                            vec![Pos::new(44)]
                        }
                        12 => {
                            vec![Pos::new(142)]
                        }
                        34 => {
                            vec![Pos::new(140)]
                        }
                        140 => {
                            vec![Pos::new(34)]
                        }
                        142 => {
                            vec![Pos::new(12)]
                        }
                        44 => {
                            vec![Pos::new(11)]
                        }
                        36 => {
                            vec![Pos::new(10)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P44 => match a_orb.u8_idx() {
                        34 => {
                            vec![Pos::new(143)]
                        }
                        143 => {
                            vec![Pos::new(34)]
                        }
                        42 => {
                            vec![Pos::new(45)]
                        }
                        45 => {
                            vec![Pos::new(42)]
                        }
                        140 => {
                            vec![Pos::new(37)]
                        }
                        37 => {
                            vec![Pos::new(140)]
                        }
                        36 => {
                            vec![Pos::new(142)]
                        }
                        142 => {
                            vec![Pos::new(36)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P140 => match a_orb.u8_idx() {
                        9 => {
                            vec![Pos::new(44), Pos::new(108)]
                        }
                        11 => {
                            vec![Pos::new(25), Pos::new(108)]
                        }
                        44 => {
                            vec![Pos::new(9), Pos::new(25)]
                        }
                        108 => {
                            vec![Pos::new(9), Pos::new(11)]
                        }
                        25 => {
                            vec![Pos::new(11), Pos::new(44)]
                        }
                        10 => {
                            vec![Pos::new(106), Pos::new(142)]
                        }
                        42 => {
                            vec![Pos::new(17), Pos::new(106)]
                        }
                        142 => {
                            vec![Pos::new(17), Pos::new(10)]
                        }
                        106 => {
                            vec![Pos::new(10), Pos::new(42)]
                        }
                        17 => {
                            vec![Pos::new(42), Pos::new(142)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                    OrbitFull::P142 => match a_orb.u8_idx() {
                        42 => {
                            vec![Pos::new(109)]
                        }
                        109 => {
                            vec![Pos::new(42)]
                        }
                        140 => {
                            vec![Pos::new(143)]
                        }
                        143 => {
                            vec![Pos::new(140)]
                        }
                        106 => {
                            vec![Pos::new(45)]
                        }
                        45 => {
                            vec![Pos::new(106)]
                        }
                        44 => {
                            vec![Pos::new(108)]
                        }
                        108 => {
                            vec![Pos::new(44)]
                        }
                        _ => {
                            vec![]
                        }
                    },
                };
                if sym.flip_xy {
                    c_poses = c_poses.into_iter().map(|p| p.flip_xy()).collect();
                }
                if sym.flip_x {
                    c_poses = c_poses.into_iter().map(|p| p.flip_x()).collect();
                }
                if sym.flip_y {
                    c_poses = c_poses.into_iter().map(|p| p.flip_y()).collect();
                }
                if sym.flip_z {
                    c_poses = c_poses.into_iter().map(|p| p.flip_z()).collect();
                }
                c_poses
            })
        });

        Self {
            white_forward,
            black_forward,
            cardinal_adjacent,
            diagonal_adjacent,
            continuations,
            knight_moves: std::array::from_fn(|_| Vec::new()),
            cardinal_slides: std::array::from_fn(|_| Vec::new()),
            diagonal_slides: std::array::from_fn(|_| Vec::new()),
        }
    }

    pub fn white_forward(&self, pos: Pos) -> &[Pos] {
        &self.white_forward[pos.idx()]
    }

    pub fn black_forward(&self, pos: Pos) -> &[Pos] {
        &self.black_forward[pos.idx()]
    }

    pub fn cardinal_adjacent(&self, pos: Pos) -> &[Pos] {
        &self.cardinal_adjacent[pos.idx()]
    }

    pub fn diagonal_adjacent(&self, pos: Pos) -> &[Pos] {
        &self.diagonal_adjacent[pos.idx()]
    }

    pub fn continuations(&self, pos_a: Pos, pos_b: Pos) -> &[Pos] {
        &self.continuations[pos_a.idx()][pos_b.idx()]
    }

    pub fn knight_moves(&self, pos: Pos) -> &[Pos] {
        &self.knight_moves[pos.idx()]
    }

    pub fn cardinal_slides(&self, pos: Pos) -> &[Slides] {
        &self.cardinal_slides[pos.idx()]
    }

    pub fn diagonal_slides(&self, pos: Pos) -> &[Slides] {
        &self.diagonal_slides[pos.idx()]
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
