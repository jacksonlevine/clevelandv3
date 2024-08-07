
use bevy::prelude::*;
use num_enum::FromPrimitive;

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq)]
#[repr(usize)]
pub enum CubeSide {
    #[num_enum(default)]
    LEFT = 0,
    RIGHT = 1,
    BOTTOM = 2,
    TOP = 3,
    BACK = 4,
    FRONT = 5,
}

pub fn get_normal(side: CubeSide) -> IVec3 {
    static NORMALS: [IVec3; 6] = [
            IVec3 { x: -1, y: 0, z: 0 },
            IVec3 { x: 1, y: 0, z: 0 },
            IVec3 { x: 0, y: -1, z: 0 },
            IVec3 { x: 0, y: 1, z: 0 },
            IVec3 { x: 0, y: 0, z: -1 },
            IVec3 { x: 0, y: 0, z: 1 },
        ];
    return NORMALS[side as usize];
}
pub struct Cube {}
impl Cube {
    pub fn get_neighbors() -> &'static [IVec3] {
        static NEIGHBORS: [IVec3; 6] = [
            IVec3 { x: -1, y: 0, z: 0 },
            IVec3 { x: 1, y: 0, z: 0 },
            IVec3 { x: 0, y: -1, z: 0 },
            IVec3 { x: 0, y: 1, z: 0 },
            IVec3 { x: 0, y: 0, z: -1 },
            IVec3 { x: 0, y: 0, z: 1 },
        ];
        return NEIGHBORS.as_slice();
    }
    pub fn get_side(side: CubeSide) -> &'static [u8] {
        #[rustfmt::skip]
        static SIDES: [[u8; 18]; 6] = [
            [
                0, 0, 1, 
                0, 0, 0, 
                0, 1, 0, 
                0, 1, 0, 
                0, 1, 1, 
                0, 0, 1, 
            ],
            [
                1, 0, 0, 
                1, 0, 1, 
                1, 1, 1, 
                1, 1, 1, 
                1, 1, 0, 
                1, 0, 0, 
            ],
            [
                0, 0, 1,
                1, 0, 1,
                1, 0, 0,
                1, 0, 0, 
                0, 0, 0, 
                0, 0, 1, 
            ],
            [
                0, 1, 0, 
                1, 1, 0, 
                1, 1, 1, 
                1, 1, 1, 
                0, 1, 1, 
                0, 1, 0, 
            ],
            [
                0, 0, 0,
                1, 0, 0, 
                1, 1, 0, 
                1, 1, 0, 
                0, 1, 0, 
                0, 0, 0, 
            ],
            [
                1, 0, 1, 
                0, 0, 1, 
                0, 1, 1, 
                0, 1, 1, 
                1, 1, 1,
                1, 0, 1,
            ],
        ];

        return SIDES[side as usize].as_slice();
    }

}
