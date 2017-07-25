// Reads book
use std::cmp::min;
use std::io::Read;
use std::fs::File;
use rand;
use rand::distributions;
use rand::distributions::IndependentSample;

use board::Move;

static BOOKNAME: &str = "data/opening.db";

pub struct Book{
    book: Vec<u8>,
    // first index to search.
    first: usize,
    // last index to search.
    last: usize,
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
        // load book
        let mut file = File::open(BOOKNAME).unwrap();
        let mut book = Vec::new();
        let len = file.metadata().unwrap().len();
        file.read_to_end(&mut book).unwrap();

        let transform = Transform::new();

        Book {
            book,
            first: 0,
            last: (len as usize)/64 - 1,
            index: 0,
            opening: true,
            transform,
            runout: false,
        }
    }
    pub fn reset(&mut self) {
        self.first = 0;
        self.last = (self.book.len() as usize)/64 - 1;
        self.index = 0;
        self.opening = true;
        self.runout = false;
    }
    pub fn gen(&mut self, last_move: Option<Move>) -> Option<(Move, bool)> {
        if self.runout {
            return None;
        }
        match last_move {
            None => {
                // 最初の1手はC4で決まっているぞ
                self.opening = false;
                self.transform.init(2, 3);
                return Some((Move::Put {
                    x: 2,
                    y: 3,
                }, true));
            },
            Some(mv) => {
                // 手を進める
                if !self.go(mv) {
                    // もうbookで探索できない
                    return None;
                }
                let first = self.first;
                let last = self.last;

                // 候補から選択
                let next =
                    if first == last {
                        first
                    } else {
                        trace!("Selecting from {} candidates", last - first + 1);
                        // ランダムに選ぶ
                        // 最大でも10個 (XXX どれくらいがいい?)
                        let num = min(10, last - first);
                        let mut idx = first;
                        let mut mx = 0u32;
                        let ran = distributions::Range::new(first, last+1);
                        let mut rng = rand::weak_rng();
                        for _ in 0..num {
                            let i = ran.ind_sample(&mut rng);
                            // TODO もうちょっときれいに書けるのでは?
                            let v = ((self.book[i * 64 + 60] as u32) << 24) | ((self.book[i * 64 + 61] as u32) << 16) | ((self.book[i * 64 + 62] as u32) << 8) | (self.book[i * 64 + 63] as u32);
                            if mx <= v {
                                // これのほうが評価値がいい
                                idx = i;
                                mx = v;
                            }
                        }
                        idx
                    };
                let ret = self.book[next * 64 + self.index];
                if ret == 0xFF {
                    // 相手の番で終わる変な定石だ
                    self.runout = true;
                    return None;
                }
                let ret = self.transform.get(ret);
                let mvx = (ret >> 4) & 0x07;
                let mvy = ret & 0x07;
                let mv2 = Move::Put {
                    x: mvx,
                    y: mvy,
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
            Move::Pass => return false,
            Move::Put {x, y} => {
                let first = self.first;
                let last = self.last;
                let book = &self.book;
                if self.opening {
                    // 最初の一手だ
                    // transformをinitする
                    self.transform.init(x, y);
                    self.opening = false;
                    // C4はDBに入っていないから探索しない
                } else {
                    // 開始と終わりを探索
                    let index = self.index;
                    let v = (x << 4) | y;
                    let firsto = find_start(book, first, last, index, v);
                    let lasto = find_last(book, first, last, index, v);
                    if firsto.is_none() || lasto.is_none() {
                        self.runout = true;
                        return false;
                    }
                    // まだありそうだ
                    self.first = firsto.unwrap();
                    self.last = lasto.unwrap();
                    self.index += 1;
                }
                return true;
            },
        }
    }
}

// binary searchでindex番目がvであるやつの開始点をさがす
fn find_start(book: &Vec<u8>, first: usize, last: usize, index: usize, v: u8) -> Option<usize> {
    let mut first = first;
    let mut last = last;
    while first < last {
        let h = (first + last) / 2; // 残り2つのときは前へ
        let vv = book[h * 64 + index];

        if v < vv {
            // ここは大きい
            last = h - 1;
        } else if v > vv {
            // ここは小さい
            first = h + 1;
        } else {
            // 同じなので前半に進む
            last = h;
        }
    }
    if first == last && book[first * 64 + index] == v {
        Some(first)
    } else {
        None
    }
}
fn find_last(book: &Vec<u8>, first: usize, last: usize, index: usize, v: u8) -> Option<usize> {
    let mut first = first;
    let mut last = last;
    while first < last {
        let h = (first + last + 1) / 2;
        let vv = book[h * 64 + index];

        if v < vv {
            // ここは大きい
            last = h - 1;
        } else if v > vv {
            // ここは小さい
            first = h + 1;
        } else {
            // 同じ
            first = h;
        }
    }
    if first == last && book[first * 64 + index] == v {
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
    }
    fn get(&self, v: u8) -> u8 {
        self.table[v as usize]
    }
}
