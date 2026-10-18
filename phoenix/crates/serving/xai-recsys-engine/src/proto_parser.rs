// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use bytes::{Bytes, BytesMut};
use http_body::Body;
use http_body_util::BodyExt;
use num::cast::AsPrimitive;
use prost::encoding::encode_varint;
use std::cmp::min;
use tonic::{Code, Status};

pub fn common_prefix_size<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    a.iter().zip(b).take_while(|&(x, y)| x == y).count()
}

pub fn decode_names(
    all_names: Vec<u8>,
    prefix_sizes: Vec<usize>,
    suffix_sizes: Vec<usize>,
) -> Result<Vec<Vec<u8>>, Status> {
    if prefix_sizes.len() != suffix_sizes.len() {
        return Err(Status::invalid_argument(format!(
            "unequal prefix/suffix size counts: {}/{}",
            prefix_sizes.len(),
            suffix_sizes.len()
        )));
    }
    if !prefix_sizes.is_empty() && prefix_sizes[0] != 0 {
        return Err(Status::invalid_argument("non-zero first prefix size"));
    }
    let mut names = Vec::<Vec<u8>>::with_capacity(prefix_sizes.len());
    let mut pos = 0;
    for (p_size, s_size) in prefix_sizes.into_iter().zip(suffix_sizes) {
        if pos + s_size > all_names.len() {
            return Err(Status::invalid_argument("bad names string"));
        }
        names.push(if p_size == 0 {
            all_names[pos..pos + s_size].to_vec()
        } else {
            let prev = &names[names.len() - 1];
            if p_size > prev.len() {
                return Err(Status::invalid_argument("bad prefix size"));
            }
            let s0 = &prev[..p_size];
            let s1 = &all_names[pos..pos + s_size];
            [s0, s1].concat()
        });
        pos += s_size;
    }
    if pos != all_names.len() {
        return Err(Status::invalid_argument("bad names string"));
    }
    Ok(names)
}

pub fn encode_names(names: Vec<Vec<u8>>) -> (Vec<u8>, Vec<usize>, Vec<usize>) {
    let mut all_names = Vec::new();
    let mut prefix_sizes = Vec::with_capacity(names.len());
    let mut suffix_sizes = Vec::with_capacity(names.len());
    let mut prev = &b""[..];
    for name in &names {
        let p_size = common_prefix_size(prev, name);
        all_names.extend_from_slice(&name[p_size..]);
        prefix_sizes.push(p_size);
        suffix_sizes.push(name.len() - p_size);
        prev = name;
    }
    (all_names, prefix_sizes, suffix_sizes)
}

pub trait BytesMutProtoExt {
    fn put_varints<T: AsPrimitive<u64>, const N: usize>(&mut self, values: [T; N]);
    fn put_slices<const N: usize>(&mut self, values: [&[u8]; N]);
    fn put_string(&mut self, key: usize, value: &[u8]);
    fn put_repeated_ints<T: AsPrimitive<u64>, const N: usize>(
        &mut self,
        keys: [usize; N],
        valuez: [&[T]; N],
    );
}

impl BytesMutProtoExt for BytesMut {
    fn put_varints<T: AsPrimitive<u64>, const N: usize>(&mut self, values: [T; N]) {
        for value in values {
            encode_varint(value.as_(), self);
        }
    }

    fn put_slices<const N: usize>(&mut self, values: [&[u8]; N]) {
        for value in values {
            self.extend_from_slice(value);
        }
    }

    fn put_string(&mut self, key: usize, value: &[u8]) {
        self.put_varints([(key << 3) | 2, value.len()]);
        self.extend_from_slice(value);
    }

    fn put_repeated_ints<T: AsPrimitive<u64>, const N: usize>(
        &mut self,
        keys: [usize; N],
        valuez: [&[T]; N],
    ) {
        for (key, values) in keys.into_iter().zip(valuez) {
            let mut buf = Vec::new();
            for value in values {
                encode_varint(value.as_(), &mut buf);
            }
            self.put_string(key, &buf);
        }
    }
}

pub fn repeated_ints<T>(ints: &mut Vec<T>) -> impl FnMut(usize, &[u8]) + Send
where
    T: 'static + Copy + Send,
    u64: AsPrimitive<T>,
{
    let mut int_ctx = VarintContext::new();
    int_ctx.init();
    move |_, data| {
        let mut pos = 0;
        while pos < data.len() {
            let consumed = int_ctx.parse(&data[pos..]);
            pos += consumed;
            if !int_ctx.done() {
                break;
            }
            ints.push(int_ctx.x().as_());
            int_ctx.init();
        }
    }
}

pub fn bytes(bytes: &mut Vec<u8>) -> impl FnMut(usize, &[u8]) + Send {
    |left, data| {
        if bytes.is_empty() {
            bytes.reserve(left);
        }
        bytes.extend_from_slice(data);
    }
}

pub fn repeated_bytes(bytes: &mut Vec<Vec<u8>>) -> impl FnMut(usize, &[u8]) + Send {
    let mut tmp = Vec::new();
    move |left, data| {
        if tmp.is_empty() {
            tmp.reserve(left);
        }
        tmp.extend_from_slice(data);
        if left == data.len() {
            bytes.push(std::mem::take(&mut tmp));
        }
    }
}

pub struct FixedSizeContext {
    x: u64,
    i: i32,
    e: i32,
}

impl Default for FixedSizeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedSizeContext {
    pub fn new() -> Self {
        Self { x: 0, i: -1, e: -1 }
    }

    pub fn init(&mut self, i: i32, e: i32) {
        self.x = 0;
        self.i = i;
        self.e = e;
    }

    pub fn parse(&mut self, input: &[u8]) -> usize {
        for (i, &elem) in input.iter().enumerate() {
            self.x |= (elem as u64) << self.i;
            self.i += if self.i < self.e { 8 } else { -8 };
            if self.i == self.e {
                return i + 1;
            }
        }
        input.len()
    }

    pub fn left(&self) -> i32 {
        self.e - self.i
    }

    pub fn done(&self) -> bool {
        self.i == self.e
    }

    pub fn x(&self) -> u64 {
        self.x
    }
}

pub struct VarintContext {
    x: u64,
    i: i32,
}

impl Default for VarintContext {
    fn default() -> Self {
        Self::new()
    }
}

impl VarintContext {
    pub fn new() -> Self {
        Self { x: 0, i: -1 }
    }

    pub fn init(&mut self) {
        self.x = 0;
        self.i = 0;
    }

    pub fn parse(&mut self, input: &[u8]) -> usize {
        for (i, &elem) in input.iter().enumerate() {
            self.x |= ((elem & 0x7F) as u64) << self.i;
            self.i += 7;
            if (elem & 0x80) == 0 {
                self.i = -1;
                return i + 1;
            }
        }
        input.len()
    }

    pub fn done(&self) -> bool {
        self.i == -1
    }

    pub fn x(&self) -> u64 {
        self.x
    }
}

type FuncStart<'a> = Box<dyn FnMut() + Send + 'a>;
type FuncU32<'a> = Box<dyn FnMut(u32) + Send + 'a>;
type FuncU64<'a> = Box<dyn FnMut(u64) + Send + 'a>;
type FuncInt<'a> = Box<dyn FnMut(usize) + Send + 'a>;
type FuncStr<'a> = Box<dyn FnMut(usize, &[u8]) + Send + 'a>;
type FuncBytes<'a> = Box<dyn FnMut(usize, Bytes) + Send + 'a>;

pub enum Func<'a> {
    Msg(Vec<(u64, Func<'a>)>),
    Start(FuncStart<'a>),
    U32(FuncU32<'a>),
    U64(FuncU64<'a>),
    Int(FuncInt<'a>),
    Str(FuncStr<'a>),
    Bytes(FuncBytes<'a>),
}

pub trait FuncConvertible<'a, Args> {
    fn into(self, x: u64) -> (u64, Func<'a>);
}

impl<'a> FuncConvertible<'a, ()> for Vec<(u64, Func<'a>)> {
    fn into(self, x: u64) -> (u64, Func<'a>) {
        assert_ne!(x, 0);
        ((x << 3) | 2, Func::Msg(self))
    }
}

impl<'a, F: FnMut() + Send + 'a> FuncConvertible<'a, ()> for F {
    fn into(self, x: u64) -> (u64, Func<'a>) {
        assert_eq!(x, 0);
        (0, Func::Start(Box::new(self)))
    }
}

impl<'a, F: FnMut(u32) + Send + 'a> FuncConvertible<'a, (u32,)> for F {
    fn into(self, x: u64) -> (u64, Func<'a>) {
        assert_ne!(x, 0);
        ((x << 3) | 5, Func::U32(Box::new(self)))
    }
}

impl<'a, F: FnMut(u64) + Send + 'a> FuncConvertible<'a, (u64,)> for F {
    fn into(self, x: u64) -> (u64, Func<'a>) {
        assert_ne!(x, 0);
        ((x << 3) | 1, Func::U64(Box::new(self)))
    }
}

impl<'a, F: FnMut(usize) + Send + 'a> FuncConvertible<'a, (usize,)> for F {
    fn into(self, x: u64) -> (u64, Func<'a>) {
        assert_ne!(x, 0);
        (x << 3, Func::Int(Box::new(self)))
    }
}

impl<'a, F: FnMut(usize, &[u8]) + Send + 'a> FuncConvertible<'a, (usize, &[u8])> for F {
    fn into(self, x: u64) -> (u64, Func<'a>) {
        assert_ne!(x, 0);
        ((x << 3) | 2, Func::Str(Box::new(self)))
    }
}

impl<'a, F: FnMut(usize, Bytes) + Send + 'a> FuncConvertible<'a, (usize, Bytes)> for F {
    fn into(self, x: u64) -> (u64, Func<'a>) {
        assert_ne!(x, 0);
        ((x << 3) | 2, Func::Bytes(Box::new(self)))
    }
}

pub fn proto<'a, F, Args>(x: u64, arg: F) -> (u64, Func<'a>)
where
    F: FuncConvertible<'a, Args>,
{
    arg.into(x)
}

#[macro_export]
macro_rules! proto {
    [ $( ( $key:expr , $val:expr ) ),* $(,)? ] => {
        vec![ $( proto( $key , $val ) ),* ]
    };
}

pub enum BodyKind {
    Request,
    Response,
}

#[allow(clippy::collapsible_if)]
pub async fn parse<'a, T: Body<Data = Bytes, Error = Status> + Send + Unpin>(
    funcs: Vec<(u64, Func<'a>)>,
    mut body: T,
    kind: BodyKind,
) -> Result<(), Status> {
    #[derive(Debug)]
    enum State {
        Frame,
        Key,
        U32,
        U64,
        Int,
        Size,
        Str,
    }

    let mut root_func = Func::Msg(funcs);
    let mut status = match kind {
        BodyKind::Request => Status::ok(""),
        BodyKind::Response => Status::invalid_argument("no gRPC status"),
    };

    let mut state = State::Frame;
    let mut func = &mut root_func;
    let mut fixed_ctx = FixedSizeContext::new();
    fixed_ctx.init(32, -8);
    let mut int_ctx = VarintContext::new();
    let mut lefts = Vec::new();
    let mut left = 0;
    let mut index = 0;

    loop {
        match body.frame().await {
            Some(Ok(frame)) => {
                if frame.is_trailers() {
                    if let Ok(trailers) = frame.into_trailers()
                        && let Some(s) = Status::from_header_map(&trailers)
                    {
                        status = s;
                    }
                } else if let Ok(chunk) = frame.into_data() {
                    let mut pos = 0;
                    while pos < chunk.len() {
                        if matches!(state, State::Key | State::Int | State::Size) {
                            let consumed = int_ctx.parse(&chunk[pos..]);
                            if consumed > left {
                                return Err(Status::invalid_argument(
                                    "varint overflows available size",
                                ));
                            }
                            left -= consumed;
                            pos += consumed;
                            if !int_ctx.done() {
                                continue;
                            }
                        }
                        if matches!(state, State::Frame | State::U64 | State::U32) {
                            let consumed = fixed_ctx.parse(&chunk[pos..]);
                            if !matches!(state, State::Frame) {
                                if consumed > left {
                                    return Err(Status::invalid_argument(
                                        "fixed size field overflows available size",
                                    ));
                                }
                                left -= consumed;
                            }
                            pos += consumed;
                            if !fixed_ctx.done() {
                                continue;
                            }
                        }
                        let handle_str = matches!(state, State::Str)
                            || (matches!(state, State::Size) && int_ctx.x() == 0);
                        let reset_state = matches!(state, State::U32 | State::U64 | State::Int);
                        match state {
                            State::Frame => {
                                if fixed_ctx.x() > 0xFFFFFFFF {
                                    return Err(Status::invalid_argument(format!(
                                        "unsupported frame compression {}",
                                        fixed_ctx.x() >> 32
                                    )));
                                }
                                if let Func::Msg(funcs) = func {
                                    if let Some(i) = funcs.iter().position(|&(x, _)| x == 0) {
                                        if let Func::Start(ref mut f) = funcs[i].1 {
                                            f();
                                        }
                                    }
                                }
                                left = fixed_ctx.x() as usize;
                                if left == 0 {
                                    fixed_ctx.init(32, -8);
                                } else {
                                    int_ctx.init();
                                    state = State::Key;
                                }
                            }
                            State::Key => {
                                index = usize::MAX;
                                if int_ctx.x() > 7 {
                                    if let Func::Msg(funcs) = func {
                                        if let Some(i) =
                                            funcs.iter().position(|&(x, _)| x == int_ctx.x())
                                        {
                                            if let Func::Msg(funcs) = func {
                                                index = i;
                                                func = &mut funcs[i].1;
                                            }
                                        }
                                    }
                                }
                                let wire = int_ctx.x() & 7;
                                state = match wire {
                                    0 => {
                                        int_ctx.init();
                                        State::Int
                                    }
                                    1 => {
                                        fixed_ctx.init(0, 64);
                                        State::U64
                                    }
                                    2 => {
                                        int_ctx.init();
                                        State::Size
                                    }
                                    5 => {
                                        fixed_ctx.init(0, 32);
                                        State::U32
                                    }
                                    _ => {
                                        return Err(Status::invalid_argument(format!(
                                            "bad wire type {wire}"
                                        )));
                                    }
                                };
                            }
                            State::U32 => {
                                if let Func::U32(f) = func {
                                    f(fixed_ctx.x() as u32)
                                }
                            }
                            State::U64 => {
                                if let Func::U64(f) = func {
                                    f(fixed_ctx.x())
                                }
                            }
                            State::Int => {
                                if let Func::Int(f) = func {
                                    f(int_ctx.x() as usize)
                                }
                            }
                            State::Size => {
                                state = State::Str;
                                let l = int_ctx.x() as usize;
                                if l > left {
                                    return Err(Status::invalid_argument(format!(
                                        "size={l} field overflows available size={left}"
                                    )));
                                }
                                lefts.push((left - l, index));
                                left = l;
                            }
                            State::Str => {}
                        }
                        if handle_str {
                            match func {
                                Func::Msg(funcs) if index != usize::MAX => {
                                    if let Some(i) = funcs.iter().position(|&(x, _)| x == 0) {
                                        if let Func::Start(ref mut f) = funcs[i].1 {
                                            f();
                                        }
                                    }
                                }
                                _ => {
                                    let len = min(left, chunk.len() - pos);
                                    if let Func::Str(f) = func {
                                        f(left, &chunk[pos..pos + len]);
                                    } else if let Func::Bytes(f) = func {
                                        f(left, chunk.slice(pos..pos + len));
                                    }
                                    pos += len;
                                    left -= len;
                                    if left != 0 {
                                        continue;
                                    }
                                }
                            }
                        }
                        if handle_str || reset_state {
                            while left == 0 && !lefts.is_empty() {
                                (left, _) = lefts.pop().expect("non-empty");
                            }
                            func = &mut root_func;
                            for &(_, i) in &lefts {
                                if let Func::Msg(funcs) = func {
                                    func = &mut funcs[i].1;
                                }
                            }
                            state = if left == 0 && lefts.is_empty() {
                                fixed_ctx.init(32, -8);
                                State::Frame
                            } else {
                                int_ctx.init();
                                State::Key
                            };
                        }
                    }
                }
            }
            Some(Err(e)) => return Err(e),
            None => break,
        }
    }
    if left != 0 || !matches!(state, State::Frame) {
        return Err(Status::invalid_argument(format!(
            "message ended {left} bytes early at depth={} and state={state:?}",
            lefts.len()
        )));
    }
    if fixed_ctx.left() != -40 {
        return Err(Status::invalid_argument(
            "message ended inside frame header",
        ));
    }
    match status.code() {
        Code::Ok => Ok(()),
        _ => Err(status),
    }
}
