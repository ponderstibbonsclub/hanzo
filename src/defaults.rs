use crate::{Direction, Point};

pub const MAP: &str = "
################################################
#..............................................#
#..............................##..............#
#....................#.........................#
#....................#.................#.......#
#......####.........#...................#......#
#......#...........#.....................#.....#
#......#..........#...........#...........#....#
#......#..........#..........#.............#...#
#......#..........#.........#..................#
#.................#........#...................#
#.................#.......#....................#
#.................#......#.....................#
#.................#......#.....................#
#.....#############......###############.......#
#..............................................#
#..............................................#
#....#.........................................#
#.....#..................................#.....#
#......#.................................#.....#
#.......#............#########...........#.....#
#........#...............................#.....#
#.........#..............................#.....#
#..........#.............................#.....#
#...........#..................................#
#............#.....................#...........#
#.............###########..........#...........#
#........................#.........#...........#
#.........................#........#...........#
#..........................#.......#...........#
#...........................#......#...........#
#............................#.....#...........#
#.......#......................................#
#.......#..............###.....................#
#.......###............#.......................#
#......................#........#..............#
#...............................#.........#....#
#.............####.............####......#.....#
#.................#.............#.......#......#
#..................#....................#......#
##....################..................#......#
#....................#..................#......#
#....................#..................#......#
#.........#..........#......###.....#####......#
#.........#..........#.....#...................#
#....................######....................#
#..............................................#
################################################
";

pub const PLAYERS: usize = 4;

pub const POSITIONS: [Option<Point>; 4] =
    [Some((40, 1)), Some((1, 5)), Some((45, 45)), Some((1, 40))];

pub const TARGETS: [Option<Point>; 4] =
    [Some((1, 46)), Some((45, 45)), Some((1, 1)), Some((42, 1))];

pub const GUARDS: [Option<(Point, Direction)>; 5] = [
    Some(((15, 12), Direction::Up)),
    Some(((10, 45), Direction::Right)),
    Some(((20, 30), Direction::Left)),
    Some(((31, 30), Direction::Up)),
    Some(((41, 10), Direction::Left)),
];
