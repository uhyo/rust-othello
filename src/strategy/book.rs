// Reads book
use std::cmp::min;
use std::io;
use std::io::{Read, Cursor, Seek, SeekFrom};
use std::fs::File;
use rand;
use rand::distributions;
use rand::distributions::IndependentSample;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

use board::{Move, Turn};

static BOOKNAME: &str = "data/opening.db";

// opening.dbのデータ構造:
// FILE:
//   BLOCK*
//
// BLOCK:
//   block_size: u64 (PLAYの数)
//   PLAY+ // playでsortされている
//
// PLAY: (24 octets)
//   padding: 0u8 * 7
//   play: u8
//   score: f64
//   block_pointer: u64 (0 if no next block)


pub struct Book{
    book: Cursor<Vec<u8>>,
    // pointer to the current block.
    block: u64,
    // first index to search.
    first: u64,
    // last index to search.
    last: u64,
    // current index of game.
    index: usize,
    // the very opening of the game
    opening: bool,
    // position transformation
    transform: Transform,
    // run out of the book
    runout: bool,
}

impl Book {
    pub fn new() -> Self {
        let transform = Transform::new();
        // load book
        let mut book = Vec::new();
        match File::open(BOOKNAME) {
            Ok(mut file) => {
                file.read_to_end(&mut book).unwrap();
                let mut book = Cursor::new(book);
                let (block, first, last, runout) = Self::init_cursor(&mut book);
                return Book {
                    book,
                    block,
                    first,
                    last,
                    index: 0,
                    opening: true,
                    transform,
                    runout,
                };
            },
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    // book is not prepared
                    book.write_u64::<BigEndian>(0).unwrap();
                    return Book {
                        book: Cursor::new(book),
                        block: 0,
                        first: 0,
                        last: 0,
                        index: 0,
                        opening: true,
                        transform,
                        runout: true,
                    };
                } else {
                    panic!("{:?}", err);
                }
            },
        }


    }
    pub fn reset(&mut self) {
        let (block, first, last, runout) = Self::init_cursor(&mut self.book);
        self.block = block;
        self.first = first;
        self.last = last;
        self.index = 0;
        self.opening = true;
        self.runout = runout;
    }
    // first, lastを初期状態に
    fn init_cursor(book: &mut Cursor<Vec<u8>>) -> (u64, u64, u64, bool) {
        // 最初のblock
        book.set_position(0);
        let block_size = book.read_u64::<BigEndian>().unwrap();
        if block_size == 0 {
            return (0, 0, 0, true);
        }
        let block = 8;
        let first = 0;
        let last = block_size - 1;
        return (block, first, last, false);
    }
    pub fn gen(&mut self, color: Turn, last_move: Option<Move>) -> Option<(Move, bool)> {
        if self.runout {
            return None;
        }
        if let Some(mv) = last_move {
            // 手を進める
            if !self.go(mv) {
                // もうbookで探索できない
                return None;
            }
        }
        match last_move {
            None => {
                // 最初の1手はC4で決まっているぞ
                let mv = Move::Put {
                    x: 2,
                    y: 3,
                };
                let nx = self.go(mv);
                return Some((mv, nx));
            },
            Some(_) => {
                let block = self.block;
                let first = self.first;
                let last = self.last;

                // 候補から選択
                let next =
                    if first == last {
                        first
                    } else {
                        trace!("Selecting from {} candidates", last - first + 1);
                        // ランダムに選ぶ
                        // 最大でも50個 (XXX どれくらいがいい?)
                        let num = min(50, 4 * (last - first));
                        let mut idx = first;
                        let mut mx = 0.0;
                        let ran = distributions::Range::new(first, last+1);
                        let mut rng = rand::weak_rng();
                        for _ in 0..num {
                            let i = ran.ind_sample(&mut rng);
                            self.book.seek(SeekFrom::Start(block + i * 24 + 8)).unwrap(); // i番目のplayのscore
                            let v = self.book.read_f64::<BigEndian>().unwrap();
                            // 評価値は黒なので
                            if color == Turn::Black {
                                if mx <= v {
                                    // これのほうが評価値がいい
                                    idx = i;
                                    mx = v;
                                }
                            } else {
                                if mx >= v {
                                    idx = i;
                                    mx = v;
                                }
                            }
                        }
                        idx
                    };
                self.book.seek(SeekFrom::Start(block + next * 24 + 7)).unwrap();
                let ret = self.book.read_u8().unwrap();
                trace!("move {}: {:x}", next, ret);
                if ret == 0xFF {
                    // 相手の番で終わる変な定石だ
                    self.runout = true;
                    return None;
                }
                let ret = self.transform.inv(ret);
                let mv2 =
                    if ret == 0x88 {
                        Move::Pass
                    } else {
                        let mvx = (ret >> 4) & 0x07;
                        let mvy = ret & 0x07;
                        Move::Put {
                            x: mvx,
                            y: mvy,
                        }
                    };
                // 自分の手番でさらに絞る
                let rb = self.go(mv2);
                return Some((mv2, rb));
            },
        }
    }
    // 返り値: まだ定石があるか
    pub fn go(&mut self, last_move: Move) -> bool {
        if self.runout {
            return false;
        }
        match last_move {
            Move::Pass => {
                // XXX bookにpassがあったら?
                return false;
            },
            Move::Put {x, y} => {
                let block = self.block;
                let first = self.first;
                let last = self.last;
                let book = &mut self.book;
                if self.opening {
                    // 最初の一手だ
                    // transformをinitする
                    self.transform.init(x, y);
                    self.opening = false;
                } 

                // 開始と終わりを探索
                let v = self.transform.get((x << 4) | y);
                let pli = binary_search(book.get_ref(), block, first, last, v);
                match pli {
                    None => {
                        // もうplayがない
                        self.runout = true;
                        return false;
                    },
                    Some(idx) => {
                        // 次のblockに進む
                        book.seek(SeekFrom::Start(block + 24 * idx + 16)).unwrap();
                        let nblock = book.read_u64::<BigEndian>().unwrap();
                        book.seek(SeekFrom::Start(nblock)).unwrap();
                        let block_size = book.read_u64::<BigEndian>().unwrap();
                        if block_size == 0 {
                            // もうない
                            self.runout = true;
                            return false;
                        } else {
                            self.block = nblock + 8;
                            self.first = 0;
                            self.last = block_size - 1;
                            return true;
                        }
                    },
                }
            },
        }
    }
}

// binary searchでplayを探す
fn binary_search(book: &Vec<u8>, block: u64, orig_first: u64, orig_last: u64, v: u8) -> Option<u64> {
    let mut first = orig_first;
    let mut last = orig_last;
    while first < last {
        let h = (first + last) / 2;
        let vv = book[(block + 24 * h + 7) as usize];

        if v < vv {
            // ここは大きい
            if orig_first < h {
                last = h - 1;
            } else {
                // 見つからなかった
                return None;
            }
        } else if v > vv {
            // ここは小さい
            if h < orig_last {
                first = h + 1;
            } else {
                return None;
            }
        } else {
            // 同じ
            first = h;
            last = h;
            break;
        }
    }
    if first == last && book[(block + 24 * first + 7) as usize] == v {
        Some(first)
    } else {
        None
    }
}

// 座標変換
struct Transform {
    table: Vec<u8>,
}
impl Transform {
    fn new() -> Self {
        let table = vec![0u8; 256];
        Transform {
            table,
        }
    }
    fn init(&mut self, x: u8, y: u8) {
        // C4をどこに移すかで4パターンある
        let mut table = &mut self.table;
        if x == 2 && y == 3 {
            // identity
            for x in 0..8 {
                for y in 0..8 {
                    table[(x << 4) | y] = ((x as u8) << 4) | (y as u8);
                }
            }
        } else if x == 3 && y == 2 {
            // x/y swap
            for x in 0..8 {
                for y in 0..8 {
                    table[(x << 4) | y] = ((y as u8) << 4) | (x as u8);
                }
            }
        } else if x == 5 && y == 4 {
            // ~x/~y
            for x in 0..8 {
                for y in 0..8 {
                    table[(x << 4) | y] = (((7-x) as u8) << 4) | ((7-y) as u8);
                }
            }
        } else {
            //~x/~y swap
            for x in 0..8 {
                for y in 0..8 {
                    table[(x << 4) | y] = (((7-y) as u8) << 4) | ((7-x) as u8);
                }
            }
        }
        // pass
        table[0x88] = 0x88;
    }
    // 順方向（実際の着手をC4基準へ）
    fn get(&self, v: u8) -> u8 {
        self.table[v as usize]
    }
    // 逆方向（C4基準の手を実際の座標へ）
    fn inv(&self, v: u8) -> u8 {
        self.table[v as usize]
    }
}
