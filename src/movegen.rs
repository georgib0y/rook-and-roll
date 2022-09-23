use std::cmp::{max, min};
use crate::board::{BISHOP, Board, KING, KNIGHT, PAWN, QUEEN, ROOK};
use crate::move_info::{FA, FH, R1, R2, R3, R6, R7, R8, SQUARES};
use crate::moves::{Move};
use crate::{MoveTables, print_bb};
use crate::move_tables::MT;

pub const QUIET: u32 = 0;
pub const DOUBLE: u32 = 1;
pub const CAP: u32 = 2;
pub const KINGSIDE: u32 = 3; // more readable in gen_king_castle
pub const WKINGSIDE: u32 = 3;
pub const BKINGSIDE: u32 = 4;
pub const QUEENSIDE: u32 = 5; // ^ readability
pub const WQUEENSIDE: u32 = 5;
pub const BQUEENSIDE: u32 = 6;
pub const PROMO: u32 = 7; // when promo is used, xpiece determines the promo piece type
pub const N_PROMO_CAP: u32 = 8;
pub const R_PROMO_CAP: u32 = 9;
pub const B_PROMO_CAP: u32 = 10;
pub const Q_PROMO_CAP: u32 = 11;
pub const EP: u32 = 12;

pub fn gen_moves(b: &Board, check: bool) -> Vec<Move> {
    if check {
        gen_check_moves(b)
    } else {
        gen_all_moves(b)
    }
}

pub fn gen_all_moves(b: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(218);
    if b.colour_to_move == 0 {
        gen_wpawn_quiet(&mut moves, b.pieces[0], b.util[2], b.util[2]);
        gen_wpawn_attack(&mut moves, b.pieces[0], b, b.util[1]);
    } else {
        gen_bpawn_quiet(&mut moves, b.pieces[1], b.util[2], b.util[2]);
        gen_bpawn_attack(&mut moves, b.pieces[1], b, b.util[0]);
    }

    gen_knight_quiet(&mut moves, b.pieces[2+b.colour_to_move], b, b.util[2]);
    gen_knight_attacks(&mut moves, b.pieces[2+b.colour_to_move], b, b.util[1-b.colour_to_move]);
    gen_rook_quiet(&mut moves, b.pieces[4+b.colour_to_move], b, b.util[2]);
    gen_rook_attacks(&mut moves, b.pieces[4+b.colour_to_move], b, b.util[1-b.colour_to_move]);
    gen_bishop_quiet(&mut moves, b.pieces[6+b.colour_to_move], b, b.util[2]);
    gen_bishop_attacks(&mut moves, b.pieces[6+b.colour_to_move], b, b.util[1-b.colour_to_move]);
    gen_queen_quiet(&mut moves, b.pieces[8+b.colour_to_move], b, b.util[2]);
    gen_queen_attacks(&mut moves, b.pieces[8+b.colour_to_move], b, b.util[1-b.colour_to_move]);
    gen_king_quiet(&mut moves, b);
    gen_king_attack(&mut moves, b);
    gen_king_castle(&mut moves, b);

    moves
}

pub fn gen_attacks(b: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(120);

    if b.colour_to_move == 0 {
        gen_wpawn_attack(&mut moves, b.pieces[0], b, b.util[1]);
    } else {
        gen_bpawn_attack(&mut moves, b.pieces[1], b, b.util[0]);
    }

    gen_knight_attacks(&mut moves, b.pieces[2+b.colour_to_move], b, b.util[1-b.colour_to_move]);
    gen_rook_attacks(&mut moves, b.pieces[4+b.colour_to_move], b, b.util[1-b.colour_to_move]);
    gen_bishop_attacks(&mut moves, b.pieces[6+b.colour_to_move], b, b.util[1-b.colour_to_move]);
    gen_queen_attacks(&mut moves, b.pieces[8+b.colour_to_move], b, b.util[1-b.colour_to_move]);
    gen_king_attack(&mut moves, b);

    moves
}

pub fn gen_quiet(b: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(120);

    if b.colour_to_move == 0 {
        gen_wpawn_quiet(&mut moves, b.pieces[0], b.util[2], b.util[2]);
    } else {
        gen_bpawn_quiet(&mut moves, b.pieces[1], b.util[2], b.util[2]);
    }

    gen_knight_quiet(&mut moves, b.pieces[2+b.colour_to_move], b, b.util[2]);
    gen_rook_quiet(&mut moves, b.pieces[4+b.colour_to_move], b, b.util[2]);
    gen_bishop_quiet(&mut moves, b.pieces[6+b.colour_to_move], b, b.util[2]);
    gen_queen_quiet(&mut moves, b.pieces[8+b.colour_to_move], b, b.util[2]);
    gen_king_quiet(&mut moves, b);
    gen_king_castle(&mut moves, b);

    moves
}

// gen all legal moves
pub fn gen_check_moves(b: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(75);
    let ksq = b.pieces[KING+b.colour_to_move].trailing_zeros() as usize;
    // generate legal king moves
    gen_king_in_check(&mut moves, b);
    let attackers = get_attackers(b, b.colour_to_move^1, ksq);

    // if double check then can return just the king moves
    if attackers.count_ones() >= 2 {
        return moves;
    }

    // get pins
    let mut pinned: u64 = 0;
    let pinners = MT.get_rook_xray(b.util[2], b.util[b.colour_to_move], ksq) |
        MT.get_bishop_xray(b.util[2], b.util[b.colour_to_move], ksq);

    // for every opp sliding piece gen moves and add any point where they intersect with the kings
    let mut rq = b.pieces[ROOK+(b.colour_to_move^1)] | b.pieces[QUEEN+(b.colour_to_move^1)];
    rq &= pinners;
    while rq > 0 {
        let rq_sq = rq.trailing_zeros() as usize;
        pinned |= (MT.rays[1][ksq] & MT.rays[5][rq_sq]) | (MT.rays[3][ksq] & MT.rays[7][rq_sq]) |
            (MT.rays[5][ksq] & MT.rays[1][rq_sq]) | (MT.rays[7][ksq] & MT.rays[3][rq_sq]);
        rq &= rq - 1;
    }

    let mut bq = b.pieces[BISHOP+(b.colour_to_move^1)] | b.pieces[QUEEN+(b.colour_to_move^1)];
    bq &= pinners;
    while bq > 0 {
        let bq_sq = bq.trailing_zeros() as usize;
        pinned |= (MT.rays[0][ksq] & MT.rays[4][bq_sq]) | (MT.rays[2][ksq] & MT.rays[6][bq_sq]) |
            (MT.rays[4][ksq] & MT.rays[0][bq_sq]) | (MT.rays[6][ksq] & MT.rays[2][bq_sq]);
        bq &= bq - 1;
    }

    // try to cap the piece
    if b.colour_to_move == 0 {
        gen_wpawn_attack(&mut moves, b.pieces[b.colour_to_move] & !pinned, b, attackers);
    } else {
        gen_bpawn_attack(&mut moves, b.pieces[b.colour_to_move] & !pinned, b, attackers);
    }
    gen_knight_attacks(&mut moves, b.pieces[KNIGHT+b.colour_to_move] & !pinned, b, attackers);
    gen_rook_attacks(&mut moves, b.pieces[ROOK+b.colour_to_move] & !pinned, b, attackers);
    gen_bishop_attacks(&mut moves, b.pieces[BISHOP+b.colour_to_move] & !pinned, b, attackers);
    gen_queen_attacks(&mut moves, b.pieces[QUEEN+b.colour_to_move] & !pinned, b, attackers);

    // try to move in the way of a sliding piece
    let attacker = get_xpiece(b, attackers.trailing_zeros());
    if attacker >= ROOK as u32 {
        // get sq of attacker + king
        let king_sq = b.pieces[KING+b.colour_to_move].trailing_zeros();
        let attk_sq = attackers.trailing_zeros();
        let higher = max(king_sq, attk_sq);
        let lower = min(king_sq, attk_sq);
        // get the positive ray from the lesser sq in the direction of the upper sq)
        // & with the lower bits of the higher square (sq - 1), this will give the moves inbetween
        let dir = get_pos_ray_dir(lower,higher);

        // inverted to keep compatibility with b.util[2]
        let inbetween = !(MT.rays[dir][lower as usize] & (SQUARES[higher as usize] - 1));

        if b.colour_to_move == 0 {
            gen_wpawn_quiet(&mut moves, b.pieces[b.colour_to_move] & !pinned, inbetween, b.util[2]);
        } else {
            gen_bpawn_quiet(&mut moves, b.pieces[b.colour_to_move] & !pinned, inbetween, b.util[2]);
        }
        gen_knight_quiet(&mut moves, b.pieces[KNIGHT+b.colour_to_move] & !pinned, b, inbetween);
        gen_rook_quiet(&mut moves, b.pieces[ROOK+b.colour_to_move] & !pinned, b, inbetween);
        gen_bishop_quiet(&mut moves, b.pieces[BISHOP+b.colour_to_move] & !pinned, b, inbetween);
        gen_queen_quiet(&mut moves, b.pieces[QUEEN+b.colour_to_move] & !pinned, b, inbetween);
    }


    moves
}

fn gen_wpawn_quiet(moves: &mut Vec<Move>, pawns: u64, occ: u64, blockers: u64) {
    // get all the pawns that are able to push forward (not including promo pawns)
    // ANDing current pawns with all empty squares shifted down a rank
    let mut push = (pawns & !(R8 | R7)) & !(occ >> 8);
    // get double push, same deal but only with the pawns on the starting rank that can push already
    // occ bb shifted down two ranks
    // kinda ugly solution to in check moves: "blockers" bb will contain the occupied spaces in
    // between the double push squares (which cannot be seen when in check from occ bb)
    let mut double = (pawns & R2) & !(occ >> 16) & !(blockers >> 8);
    // get all pawns that can promo
    let mut promo = (pawns & R7) & !(occ >> 8);

    while push > 0 {
        let from = push.trailing_zeros();
        let to = from+8;
        moves.push(Move::new(from, to, 0, 0, QUIET));
        push &= push - 1;
    }

    while double > 0 {
        let from = double.trailing_zeros();
        let to = from + 16;
        moves.push(Move::new(from, to, 0 , 0, DOUBLE));
        double &= double - 1;
    }

    while promo > 0 {
        let from = promo.trailing_zeros();
        let to = from + 8;
        moves.push(Move::new(from, to, 0, 8, PROMO));
        moves.push(Move::new(from, to, 0, 4, PROMO));
        moves.push(Move::new(from, to, 0, 2, PROMO));
        moves.push(Move::new(from, to, 0, 6, PROMO));
        promo &= promo - 1;
    }
}

fn gen_wpawn_attack(moves: &mut Vec<Move>, pawns: u64, b: &Board, opp: u64) {
    // shift all opponent pieces down right to match all pawns that wouldnt promote
    let mut up_lefts = (pawns & !FA & !R7) & (opp >> 7);
    let mut up_left_promos = (pawns & !FA & R7) & (opp >> 7);
    let mut up_rights = (pawns & !FH & !R7) & (opp >> 9);
    let mut up_right_promos = (pawns & !FH & R7) & (opp >> 9);

    while up_lefts > 0 {
        let from = up_lefts.trailing_zeros();
        let to = from + 7;
        moves.push(Move::new(from, to, 0, get_xpiece(b, to), CAP));
        up_lefts &= up_lefts - 1;
    }

    while up_left_promos > 0 {
        let from = up_left_promos.trailing_zeros();
        let to = from + 7;
        let xpiece = get_xpiece(b, to);
        moves.push(Move::new(from, to, 0, xpiece, Q_PROMO_CAP));
        moves.push(Move::new(from, to, 0, xpiece, R_PROMO_CAP));
        moves.push(Move::new(from, to, 0, xpiece, N_PROMO_CAP));
        moves.push(Move::new(from, to, 0, xpiece, B_PROMO_CAP));
        up_left_promos &= up_left_promos - 1;
    }

    while up_rights > 0 {
        let from = up_rights.trailing_zeros();
        let to = from + 9;
        moves.push(Move::new(from, to, 0, get_xpiece(b, to), CAP));
        up_rights &= up_rights - 1;
    }

    while up_right_promos > 0 {
        let from = up_right_promos.trailing_zeros();
        let to = from + 9;
        let xpiece = get_xpiece(b, to);
        moves.push(Move::new(from, to, 0, xpiece, Q_PROMO_CAP));
        moves.push(Move::new(from, to, 0, xpiece, R_PROMO_CAP));
        moves.push(Move::new(from, to, 0, xpiece, N_PROMO_CAP));
        moves.push(Move::new(from, to, 0, xpiece, B_PROMO_CAP));
        up_right_promos &= up_right_promos - 1;
    }

    // if ep is set, check to see if any pawns can take and pawn is present (for check movegen)
    // SQUARES[64] == 0 so this works even if b.ep == 64
    if SQUARES[b.ep] & (opp << 8) > 0 {
        // shift all pawns up left and check
        if SQUARES[b.ep] & ((pawns & !FA) << 7) > 0 {
            moves.push(Move::new((b.ep-7) as u32, b.ep as u32, 0, 1, EP));
        }

        // shift all pawns up right
        if SQUARES[b.ep] & ((pawns & !FH) << 9) > 0 {
            moves.push(Move::new((b.ep-9) as u32, b.ep as u32, 0, 1, EP));
        }
    }
}

fn gen_bpawn_quiet(moves: &mut Vec<Move>, pawns: u64, occ: u64, blockers: u64) {
    let mut push = (pawns & !(R1 | R2)) & !(occ << 8);
    let mut double = (pawns & R7) & !(occ << 16) & !(blockers << 8);
    let mut promo = (pawns & R2) & !(occ << 8);

    while push > 0 {
        let from = push.trailing_zeros();
        let to = from - 8;
        moves.push(Move::new(from, to, 1, 0, QUIET));
        push &= push - 1;
    }

    while double > 0 {
        let from = double.trailing_zeros();
        let to = from - 16;
        moves.push(Move::new(from, to, 1, 0, DOUBLE));
        double &= double - 1;
    }

    while promo > 0 {
        let from = promo.trailing_zeros();
        let to = from - 8;
        moves.push(Move::new(from, to, 1, 9, PROMO));
        moves.push(Move::new(from, to, 1, 5, PROMO));
        moves.push(Move::new(from, to, 1, 3, PROMO));
        moves.push(Move::new(from, to, 1, 7, PROMO));
        promo &= promo - 1;
    }
}

fn gen_bpawn_attack(moves: &mut Vec<Move>, pawns: u64, b: &Board, opp: u64) {
    let mut down_rights = (pawns & !FH & !R2) & (opp << 7);

    while down_rights > 0 {
        let from = down_rights.trailing_zeros();
        let to = from - 7;
        moves.push(Move::new(from, to, 1, get_xpiece(b, to), CAP));
        down_rights &= down_rights - 1;
    }

    let mut down_right_promos = (pawns & !FH & R2) & (opp << 7);

    while down_right_promos > 0 {
        let from = down_right_promos.trailing_zeros();
        let to = from - 7;
        let xpiece = get_xpiece(b, to);
        moves.push(Move::new(from, to, 1, xpiece, Q_PROMO_CAP));
        moves.push(Move::new(from, to, 1, xpiece, R_PROMO_CAP));
        moves.push(Move::new(from, to, 1, xpiece, N_PROMO_CAP));
        moves.push(Move::new(from, to, 1, xpiece, B_PROMO_CAP));
        down_right_promos &= down_right_promos - 1;
    }

    let mut down_lefts = (pawns & !FA & !R2) & (opp << 9);

    while down_lefts > 0 {
        let from = down_lefts.trailing_zeros();
        let to = from - 9;
        moves.push(Move::new(from, to, 1, get_xpiece(b, to), CAP));
        down_lefts &= down_lefts - 1;
    }

    let mut down_lefts_promos = (pawns & !FA & R2) & (opp << 9);

    while down_lefts_promos > 0 {
        let from = down_lefts_promos.trailing_zeros();
        let to = from - 9;
        let xpiece = get_xpiece(b, to);
        moves.push(Move::new(from, to, 1, xpiece, Q_PROMO_CAP));
        moves.push(Move::new(from, to, 1, xpiece, R_PROMO_CAP));
        moves.push(Move::new(from, to, 1, xpiece, N_PROMO_CAP));
        moves.push(Move::new(from, to, 1, xpiece, B_PROMO_CAP));
        down_lefts_promos &= down_lefts_promos - 1;
    }

    // if ep is set, check to see if any pawns can take
    if SQUARES[b.ep] & (opp >> 8) != 0 {
        // shift all pawns down right and check
        if SQUARES[b.ep] & ((pawns & !FH) >> 7) > 0{
            moves.push(
                Move::new(
                    (b.ep+7) as u32,
                    b.ep as u32,
                    1,
                    0,
                    EP)
            );
            // shift all pawns down left
        }

        if SQUARES[b.ep] & ((pawns & !FA) >> 9) > 0 {
            moves.push(
                Move::new(
                    (b.ep+9) as u32,
                    b.ep as u32,
                    1,
                    0,
                    EP)
            );
        }
    }
}

fn gen_knight_quiet(moves: &mut Vec<Move>, mut knights: u64, b: &Board, occ: u64) {
    while knights > 0 {
        let from = knights.trailing_zeros();
        let mut quiet = MT.knight_moves[from as usize] & !occ;
        while quiet > 0 {
            moves.push(Move::new(from, quiet.trailing_zeros(),
                                 2+b.colour_to_move as u32, 0, QUIET)
            );
            quiet &= quiet -1;
        }
        knights &= knights - 1;
    }
}

fn gen_knight_attacks(moves: &mut Vec<Move>, mut knights: u64, b: &Board, opp: u64) {
    while knights > 0 {
        let from = knights.trailing_zeros();
        let mut attacks = MT.knight_moves[from as usize] & opp;
        while attacks > 0 {
            let to = attacks.trailing_zeros();
            moves.push(Move::new(from, to, 2+b.colour_to_move as u32,
                                 get_xpiece(b, to), CAP)
            );
            attacks &= attacks -1;
        }
        knights &= knights - 1;
    }
}

fn gen_rook_quiet(moves: &mut Vec<Move>, mut rooks: u64, b: &Board, occ: u64) {
    while rooks > 0 {
        let from = rooks.trailing_zeros();
        let mut quiet = MT.get_rook_moves(
            b.util[2],
            rooks.trailing_zeros() as usize
        ) & !occ;

        // print_bb(quiet);

        while quiet > 0 {
            moves.push(Move::new(from, quiet.trailing_zeros(),
                                 (4 + b.colour_to_move) as u32, 0, QUIET)
            );
            quiet &= quiet - 1;
        }

        rooks &= rooks - 1;
    }
}

fn gen_rook_attacks(moves: &mut Vec<Move>, mut rooks: u64, b: &Board, opp: u64) {
    while rooks > 0 {
        let from = rooks.trailing_zeros();
        let mut attack = MT.get_rook_moves(
            b.util[2],
            rooks.trailing_zeros() as usize
        ) & opp;

        // print_bb(attack);
        // if attack > 0 {
        //     print_bb(attack);
        // }

        while attack > 0 {
            let to = attack.trailing_zeros();
            moves.push(Move::new(from, to,
                                 (4 + b.colour_to_move) as u32,
                                 get_xpiece(b, to), CAP)
            );
            attack &= attack - 1;
        }

        rooks &= rooks - 1;
    }
}


fn gen_bishop_quiet(moves: &mut Vec<Move>, mut bishops: u64, b: &Board, occ: u64) {
    while bishops > 0 {
        let from = bishops.trailing_zeros();
        let mut quiet = MT.get_bishop_moves(
            b.util[2],
            bishops.trailing_zeros() as usize
        ) & !occ;

        while quiet > 0 {
            moves.push(Move::new(from, quiet.trailing_zeros(),
                                 (6 + b.colour_to_move) as u32, 0, QUIET)
            );
            quiet &= quiet - 1;
        }

        bishops &= bishops - 1;
    }
}

fn gen_bishop_attacks(moves: &mut Vec<Move>, mut bishops: u64, b: &Board, opp: u64) {
    while bishops > 0 {
        let from = bishops.trailing_zeros();
        let mut attack = MT.get_bishop_moves(
            b.util[2],
            bishops.trailing_zeros() as usize
        ) & opp;

        // if attack > 0 {
        //     print_bb(attack);
        // }

        while attack > 0 {
            let to = attack.trailing_zeros();
            moves.push(Move::new(from, to,
                                 (6 + b.colour_to_move) as u32,
                                 get_xpiece(b, to), CAP)
            );
            attack &= attack - 1;
        }

        bishops &= bishops - 1;
    }
}

fn gen_queen_quiet(moves: &mut Vec<Move>, mut queens: u64, b: &Board, occ: u64) {
    while queens > 0 {
        let from = queens.trailing_zeros();
        let mut quiet = MT.get_rook_moves(b.util[2],queens.trailing_zeros() as usize);
        quiet |= MT.get_bishop_moves(b.util[2], queens.trailing_zeros() as usize );
        quiet &= !occ;

        // print_bb(quiet);

        while quiet > 0 {
            moves.push(Move::new(from, quiet.trailing_zeros(),
                                 (8 + b.colour_to_move) as u32, 0, QUIET)
            );
            quiet &= quiet - 1;
        }

        queens &= queens - 1;
    }
}

fn gen_queen_attacks(moves: &mut Vec<Move>, mut queens: u64, b: &Board, opp: u64) {
    while queens > 0 {
        let from = queens.trailing_zeros();
        let mut attack = MT.get_rook_moves(b.util[2], queens.trailing_zeros() as usize);
        attack |= MT.get_bishop_moves(b.util[2], queens.trailing_zeros() as usize);
        attack &= opp;

        // print_bb(attack);
        // if attack > 0 {
        //     print_bb(attack);
        // }

        while attack > 0 {
            let to = attack.trailing_zeros();
            moves.push(Move::new(from, to,
                                 (8 + b.colour_to_move) as u32,
                                 get_xpiece(b, to), CAP)
            );
            attack &= attack - 1;
        }

        queens &= queens - 1;
    }
}

fn gen_king_quiet(moves: &mut Vec<Move>, b: &Board) {
    let from = b.pieces[10+b.colour_to_move].trailing_zeros();
    let mut quiet = MT.king_moves[from as usize] & !b.util[2];
    while quiet > 0 {
        moves.push(Move::new(from, quiet.trailing_zeros(),
                             (10 + b.colour_to_move) as u32, 0, QUIET));
        quiet &= quiet - 1;
    }
}

fn gen_king_attack(moves: &mut Vec<Move>, b: &Board) {
    let from = b.pieces[10+b.colour_to_move].trailing_zeros();
    let mut attack = MT.king_moves[from as usize] & b.util[1-b.colour_to_move];
    while attack > 0 {
        let to = attack.trailing_zeros();
        moves.push(Move::new(from, to, (10+b.colour_to_move) as u32,
                             get_xpiece(b, to), CAP));
        attack &= attack - 1;
    }
}

fn gen_king_castle(moves: &mut Vec<Move>, b: &Board) {
    let king = b.pieces[10 + b.colour_to_move];

    // get castle rights for the colour to move
    let kingside = (b.castle_state >> (1 + (2 * (1-b.colour_to_move)) )) & 1;
    let queenside = (b.castle_state >> (2 * (1-b.colour_to_move)) ) & 1;

    // dbg!(kingside, queenside);

    // if castle right is set and the way is clear
    if kingside > 0 && b.util[2] & (0x60 << (b.colour_to_move*56)) == 0 {
        // println!("kingside");
        let from = king.trailing_zeros();
        moves.push(Move::new(from, from+2, (10+b.colour_to_move) as u32,
                   0, KINGSIDE+(b.colour_to_move as u32))
        );
    }

    if queenside > 0 && b.util[2] & (0xE << (b.colour_to_move*56)) == 0 {
        // println!("queenside");
        let from = king.trailing_zeros();
        moves.push(Move::new(from, from-2, (10+b.colour_to_move) as u32,
                             0, QUEENSIDE+(b.colour_to_move as u32))
        );
    }
}

fn gen_king_in_check(moves: &mut Vec<Move>, b: &Board) {
    let from = b.pieces[10+b.colour_to_move].trailing_zeros();

    // get all squares possible king move squares that arent attacked
    let occ = b.util[2] & !b.pieces[KING+b.colour_to_move];
    let possible = possible_king_moves(b, from as usize, b.colour_to_move^1, occ);

    // println!("{b}");
    // print_bb(possible);

    let mut quiets = possible & !b.util[2];
    let mut attacks = possible & b.util[b.colour_to_move^1];

    while quiets > 0 {
        let to = quiets.trailing_zeros();
        quiets &= quiets - 1;
        // if sq_attacked(b, to as usize, occ, b.colour_to_move^1, MT) { continue; }
        moves.push(Move::new(from, to,(10 + b.colour_to_move) as u32, 0, QUIET));
    }

    while attacks > 0 {
        let to = attacks.trailing_zeros();
        attacks &= attacks - 1;
        // if sq_attacked(b, to as usize, b.util[2], b.colour_to_move^1, MT) { continue; }
        moves.push(Move::new(from, to, (10+b.colour_to_move) as u32,
                             get_xpiece(b, to), CAP));
    }
}

pub fn possible_king_moves(
    b: &Board,
    ksq: usize,
    opp_colour: usize,
    occ: u64
) -> u64 {
    // possible starts out as any move that doesnt move over its own pieces
    let mut possible = MT.king_moves[ksq] & !b.util[b.colour_to_move];

    // pawn moves and double pushes
    possible &= if opp_colour == 0 {
        !( ((b.pieces[0] & !FA) << 7) | ((b.pieces[0] & !FH) << 9) )
    } else {
        !( ((b.pieces[1] & !FH) >> 7) | ((b.pieces[1] & !FA) >> 9) )
    };

    // enemy king
    possible &= !(MT.king_moves[b.pieces[KING+ opp_colour].trailing_zeros() as usize]);

    // knight moves
    let mut knights = b.pieces[KNIGHT+ opp_colour];
    while knights > 0 {
        possible &= !(MT.knight_moves[knights.trailing_zeros() as usize]);
        knights &= knights - 1;
    }
    // rook/queen
    let mut rook_queen = b.pieces[ROOK+ opp_colour] | b.pieces[QUEEN+ opp_colour];
    while rook_queen > 0 {
        possible &= !(MT.get_rook_moves(occ, rook_queen.trailing_zeros() as usize));
        rook_queen &= rook_queen -1
    }
    // bishop/queen
    let mut bishop_queen = b.pieces[BISHOP+ opp_colour] | b.pieces[QUEEN+ opp_colour];
    while bishop_queen > 0 {
        possible &= !(MT.get_bishop_moves(occ, bishop_queen.trailing_zeros() as usize));
        bishop_queen &= bishop_queen-1
    }

    possible
}

pub fn get_piece(b: &Board, index: u32) -> u32 {
    let sq = SQUARES[index as usize];
    let start = b.colour_to_move;
    for piece in (start..12).step_by(2) {
        if sq & b.pieces[piece] > 0 {
            return piece as u32;
        }
    }

    12
}

pub fn get_xpiece(b: &Board, index: u32) -> u32 {
    let sq = SQUARES[index as usize];
    let start = 1- b.colour_to_move;
    for xpiece in (start..12).step_by(2) {
        if sq & b.pieces[xpiece] > 0 {
            return xpiece as u32;
        }
    }

    12
}

// occ is the bb that is fed to the magic bb function (for hiding the king in check move gen)
pub fn sq_attacked(b: &Board, sq: usize, occ: u64, colour_of_attacker: usize) -> bool {
    MT.pawn_attacks[colour_of_attacker ^ 1][sq] & b.pieces[PAWN + colour_of_attacker] > 0 ||
    MT.knight_moves[sq] & b.pieces[KNIGHT + colour_of_attacker] > 0 ||
    MT.king_moves[sq] & b.pieces[KING + colour_of_attacker] > 0 ||
    MT.get_bishop_moves(occ, sq) & (b.pieces[QUEEN + colour_of_attacker] | b.pieces[BISHOP + colour_of_attacker]) > 0 ||
    MT.get_rook_moves(occ, sq) & (b.pieces[ROOK + colour_of_attacker] | b.pieces[QUEEN + colour_of_attacker]) > 0
}

#[inline]
pub fn moved_into_check(b: &Board, m: &Move) -> bool {
    let ksq = b.pieces[KING+(b.colour_to_move^1)].trailing_zeros() as usize;
    // the m from sq is inline with the king return sq_attacked, otherwise false
    SQUARES[m.from() as usize] & MT.superrays[ksq] > 0 &&
        sq_attacked(b, ksq, b.util[2], b.colour_to_move)
}

#[inline]
pub fn _moved_into_check(b: &Board, m: &Move) -> bool {
    sq_attacked(b, b.pieces[KING+(b.colour_to_move^1)].trailing_zeros() as usize,
        b.util[2], b.colour_to_move)

}

#[inline]
pub fn is_in_check(b: &Board) -> bool {
    sq_attacked(b, b.pieces[KING+b.colour_to_move].trailing_zeros() as usize,
                b.util[2], b.colour_to_move^1)
}

pub fn get_attackers(b: &Board, colour_to_move: usize, sq: usize) -> u64 {
    let mut attackers = 0;
    let pawns = b.pieces[PAWN + colour_to_move];
    attackers |= MT.pawn_attacks[colour_to_move^1][sq] & pawns;

    let knights = b.pieces[KNIGHT + colour_to_move];
    attackers |= MT.knight_moves[sq] & knights;

    let king = b.pieces[KING + colour_to_move];
    attackers |= MT.king_moves[sq] & king;

    let bishop_queen = b.pieces[QUEEN + colour_to_move] | b.pieces[BISHOP + colour_to_move];
    attackers |= MT.get_bishop_moves(b.util[2], sq) & bishop_queen;

    let rook_queen = b.pieces[ROOK + colour_to_move] | b.pieces[QUEEN + colour_to_move];
    attackers |= MT.get_rook_moves(b.util[2], sq) & rook_queen;

    attackers
}
/*
TODO an idea could be to delay as much information gathering as possible an put it into is legal
move or a similar function (expensive things like get_xpiece and what not)
finding out xpiece could also be put into copy_make()
*/

// this function is called before the move is made!
pub fn is_legal_move(b: &Board, m: &Move) -> bool {
    match m.move_type() {
        // check castle moves to see if the king passes through an attacked square
        WKINGSIDE =>  !sq_attacked(b, 5, b.util[2], 1)  & !sq_attacked(b, 6, b.util[2], 1),
        WQUEENSIDE => !sq_attacked(b, 3, b.util[2], 1)  & !sq_attacked(b, 2, b.util[2], 1),
        BKINGSIDE =>  !sq_attacked(b, 61, b.util[2], 0) & !sq_attacked(b, 62, b.util[2], 0),
        BQUEENSIDE => !sq_attacked(b, 59, b.util[2], 0) & !sq_attacked(b, 58, b.util[2], 0),
        _ => true
    }
}

fn get_pos_ray_dir(lower: u32, higher: u32) -> usize {
    for dir in 0..4 {
        if SQUARES[higher as usize] & MT.rays[dir][lower as usize] > 0 { return dir; }
    }
    for dir in 4..8 {
        if SQUARES[lower as usize] & MT.rays[dir][higher as usize] > 0 { return dir-4; }
    }

    panic!("Could not find ray dir")

}