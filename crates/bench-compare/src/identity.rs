//! What a measured function of a WebAssembly module computes, as a hash that layout leaves as it
//! was: its [`crate::module::Stream`], with each `call` target replaced by the hash of the callee and
//! each address by the hash of its symbol.
//!
//! - A function that can never return ends the run that calls it, which then gives no figure: its
//!   code counts for nothing, and a `call` of it for one mark. So panics and what they format stay
//!   out of the hash of the code around them.
//! - The hash of a symbol covers its bytes, with each address in them replaced by the distance from
//!   its symbol and the hash of that symbol. A symbol of a writable segment hashes its name as well:
//!   two statics of equal bytes differ.
//! - The line and the column of a `core::panic::Location` count for nothing: a comment moves them,
//!   and they reach only the message of a panic.
//! - The table index of a function counts as a mark: only a `call_indirect` can run the function, and
//!   [`CodeHash::indirect`] tells which functions reach one.
//! - Symbols or functions that refer to each other take one hash each after as many rounds as they
//!   are: a change in one reaches all of them.

use crate::fingerprint::Fingerprint;
use crate::module::{
    Control, DataSymbol, HoleKind, Module, ModuleError, Relocated, Stream, Symbol,
};

/// Marks in a hash: what stood there has no bytes of its own.
mod mark {
    /// A `call` of a function that cannot return.
    pub(super) const DIVERGES: u128 = 0xd1;
    /// A symbol that holds a `core::panic::Location`.
    pub(super) const LOCATION: u128 = 0xd3;
    /// A symbol without an address, which a relocation cannot name.
    pub(super) const OTHER: u128 = 0xd4;
    /// The table index of a function in the data.
    pub(super) const TABLE: u8 = 0xd5;
}

/// What a function computes, with all that it can reach.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodeHash {
    /// The hash.
    pub hash: u128,
    /// Whether it can reach a `call_indirect`, a `ref.func` or another instruction that decides at
    /// run time what it reaches: the hash then does not cover all that the function can run.
    pub indirect: bool,
}

/// The state of a node in the walk of Tarjan's algorithm.
#[derive(Clone, Copy, Debug, Default)]
struct Walk {
    /// The order of its first visit.
    index: Option<u32>,
    /// The lowest order that it reaches.
    low: u32,
    /// Whether it is on the stack of the walk.
    on_stack: bool,
}

/// Tarjan's algorithm over nodes numbered from 0: it finds each set of nodes that reach each other
/// once.
struct Components {
    walk: Vec<Walk>,
    stack: Vec<u32>,
    next_index: u32,
}

impl Components {
    fn new(nodes: usize) -> Self {
        Self {
            walk: vec![Walk::default(); nodes],
            stack: Vec::new(),
            next_index: 0,
        }
    }

    /// The new components that `start` reaches, in an order where a node's successors come first.
    fn reached_from(&mut self, start: u32, successors: &impl Fn(u32) -> Vec<u32>) -> Vec<Vec<u32>> {
        let mut found = Vec::new();
        if self.walk[start as usize].index.is_none() {
            self.visit(start, successors, &mut found);
        }
        found
    }

    fn visit(
        &mut self,
        node: u32,
        successors: &impl Fn(u32) -> Vec<u32>,
        found: &mut Vec<Vec<u32>>,
    ) {
        let order = self.next_index;
        self.next_index += 1;
        self.walk[node as usize] = Walk {
            index: Some(order),
            low: order,
            on_stack: true,
        };
        self.stack.push(node);
        for successor in successors(node) {
            let Walk {
                index,
                low: _,
                on_stack,
            } = self.walk[successor as usize];
            match index {
                None => {
                    self.visit(successor, successors, found);
                    let low = self.walk[successor as usize].low;
                    let state = &mut self.walk[node as usize];
                    state.low = state.low.min(low);
                }
                Some(index) if on_stack => {
                    let state = &mut self.walk[node as usize];
                    state.low = state.low.min(index);
                }
                Some(_) => {}
            }
        }
        if self.walk[node as usize].low != order {
            return;
        }
        let mut component = Vec::new();
        while let Some(member) = self.stack.pop() {
            self.walk[member as usize].on_stack = false;
            component.push(member);
            if member == node {
                break;
            }
        }
        found.push(component);
    }
}

/// The hash of `symbol`, with the hashes `symbols` of the symbols that it names.
fn symbol_hash(module: &Module<'_>, symbols: &[u128], symbol: u32) -> u128 {
    match module.symbols[symbol as usize] {
        Symbol::Other => mark::OTHER,
        Symbol::Absolute { name, address: _ } => Fingerprint::of(name.as_bytes()),
        Symbol::Data(data) if module.is_location(&data) => mark::LOCATION,
        Symbol::Data(data) => data_hash(module, symbols, &data),
    }
}

/// The hash of the definition `data`.
fn data_hash(module: &Module<'_>, symbols: &[u128], data: &DataSymbol<'_>) -> u128 {
    let mut state = Fingerprint::new();
    state.update(&(data.bytes.len() as u64).to_le_bytes());
    if !module.segments[data.segment].read_only {
        state.update(data.name.as_bytes());
    }
    let mut at = 0;
    for (address, relocated) in module.relocations_in(data.address, data.bytes.len()) {
        let word = (*address - data.address) as usize;
        state.update(&data.bytes[at..word]);
        at = (word + 4).min(data.bytes.len());
        match relocated {
            Relocated::TableIndex => state.update(&[mark::TABLE]),
            Relocated::Address { symbol, addend } => {
                state.update(&addend.to_le_bytes());
                state.update_hash(symbols[*symbol as usize]);
            }
        }
    }
    state.update(&data.bytes[at..]);
    state.finish()
}

/// The symbols that `symbol` names in its bytes.
fn named_symbols(module: &Module<'_>, symbol: u32) -> Vec<u32> {
    let Symbol::Data(data) = module.symbols[symbol as usize] else {
        return Vec::new();
    };
    module
        .relocations_in(data.address, data.bytes.len())
        .iter()
        .filter_map(|(_, relocated)| match relocated {
            Relocated::Address { symbol, addend: _ } => Some(*symbol),
            Relocated::TableIndex => None,
        })
        .collect()
}

/// The hash of each symbol of `module`.
fn symbol_hashes(module: &Module<'_>) -> Vec<u128> {
    let count = module.symbols.len();
    let mut components = Components::new(count);
    let mut hashes = vec![0; count];
    for symbol in 0..count as u32 {
        for component in components.reached_from(symbol, &|node| named_symbols(module, node)) {
            for _ in 0..component.len() {
                let round: Vec<u128> = component
                    .iter()
                    .map(|member| symbol_hash(module, &hashes, *member))
                    .collect();
                for (member, hash) in component.iter().zip(round) {
                    hashes[*member as usize] = hash;
                }
            }
        }
    }
    hashes
}

/// The defined functions that `stream` calls and that can return.
fn stream_callees<'s>(stream: &'s Stream, returns: &'s [bool]) -> impl Iterator<Item = u32> + 's {
    stream.holes.iter().filter_map(|hole| match hole.kind {
        HoleKind::Call(callee) if returns.get(callee as usize).copied().unwrap_or(false) => {
            Some(callee)
        }
        HoleKind::Call(_) | HoleKind::Data(_) => None,
    })
}

/// The defined functions that `function` calls and that can return.
fn callees(module: &Module<'_>, returns: &[bool], function: u32) -> Vec<u32> {
    module.stream(function).map_or_else(Vec::new, |stream| {
        stream_callees(stream, returns)
            .filter(|callee| module.stream(*callee).is_some())
            .collect()
    })
}

/// The functions of a module, with their hashes as far as a query needed them.
pub struct Identity<'a> {
    module: Module<'a>,
    /// Per function index: whether it can return. Imports can.
    returns: Vec<bool>,
    /// Per function index: its finished hash.
    hashes: Vec<Option<CodeHash>>,
    components: Components,
    /// Per symbol: its hash.
    symbols: Vec<u128>,
    /// The symbols with bytes, by address, which a word at run time can address: the address, the
    /// size and the index of each.
    by_address: Vec<(u32, u32, u32)>,
}

impl<'a> Identity<'a> {
    /// The functions of the module `bytes`, which a build with `-C link-arg=--emit-relocs` linked. An
    /// instruction outside the table of [`crate::module`] is a [`ModuleError`].
    pub fn of(bytes: &'a [u8]) -> Result<Self, ModuleError> {
        let module = Module::read(bytes)?;
        let functions = module.functions() as usize;
        let returns = returning(&module);
        let symbols = symbol_hashes(&module);
        let mut by_address: Vec<(u32, u32, u32)> = (0_u32..)
            .zip(&module.symbols)
            .filter_map(|(index, symbol)| match symbol {
                Symbol::Data(data) if !data.bytes.is_empty() => {
                    Some((data.address, data.bytes.len() as u32, index))
                }
                Symbol::Data(_) | Symbol::Absolute { .. } | Symbol::Other => None,
            })
            .collect();
        by_address.sort_unstable();
        Ok(Self {
            module,
            returns,
            hashes: vec![None; functions],
            components: Components::new(functions),
            symbols,
            by_address,
        })
    }

    /// The function at the index `slot` of the table.
    pub fn in_table(&self, slot: u32) -> Result<u32, ModuleError> {
        self.module
            .table
            .get(slot as usize)
            .copied()
            .flatten()
            .ok_or_else(|| ModuleError {
                offset: 0,
                reason: format!("no function at the index {slot} of the table"),
            })
    }

    /// The hash of the function at the function index `function`.
    pub fn function(&mut self, function: u32) -> Result<CodeHash, ModuleError> {
        if function >= self.module.functions() {
            return Err(ModuleError {
                offset: 0,
                reason: format!("no function of the index {function}"),
            });
        }
        if self.hashes[function as usize].is_none() {
            let (module, returns) = (&self.module, &self.returns);
            let found = self
                .components
                .reached_from(function, &|node| callees(module, returns, node));
            for component in found {
                self.finish(&component);
            }
        }
        self.hashes[function as usize].ok_or_else(|| ModuleError {
            offset: 0,
            reason: format!("no hash of the function {function}"),
        })
    }

    /// The hash of the function at the index `slot` of the table.
    pub fn entry(&mut self, slot: u32) -> Result<CodeHash, ModuleError> {
        let function = self.in_table(slot)?;
        self.function(function)
    }

    /// Where the function `ours` differs from the function `theirs` of `other`, in the deepest
    /// function of the first path of calls whose hashes differ; `None` for functions of one hash.
    pub fn difference(
        &mut self,
        ours: u32,
        other: &mut Identity<'_>,
        theirs: u32,
    ) -> Result<Option<String>, ModuleError> {
        if self.function(ours)?.hash == other.function(theirs)?.hash {
            return Ok(None);
        }
        let name = |identity: &Identity<'_>, function: u32| {
            identity
                .module
                .names
                .iter()
                .find(|(index, _)| *index == function)
                .map_or_else(|| function.to_string(), |(_, name)| (*name).to_owned())
        };
        let (Some(mine), Some(yours)) = (
            self.module
                .stream(ours)
                .map(|stream| (stream.bytes.clone(), stream.holes.clone())),
            other
                .module
                .stream(theirs)
                .map(|stream| (stream.bytes.clone(), stream.holes.clone())),
        ) else {
            return Ok(Some(format!(
                "the imports or functions {} and {}",
                name(self, ours),
                name(other, theirs)
            )));
        };
        if mine.0 != yours.0 {
            let at = mine
                .0
                .iter()
                .zip(&yours.0)
                .take_while(|(a, b)| a == b)
                .count();
            let around =
                |bytes: &[u8]| bytes[at.saturating_sub(8)..(at + 16).min(bytes.len())].to_vec();
            return Ok(Some(format!(
                "{}: {} and {} bytes, first at {at}: {:02x?} and {:02x?}",
                name(self, ours),
                mine.0.len(),
                yours.0.len(),
                around(&mine.0),
                around(&yours.0),
            )));
        }
        for (mine_hole, your_hole) in mine.1.iter().zip(&yours.1) {
            match (mine_hole.kind, your_hole.kind) {
                (HoleKind::Call(a), HoleKind::Call(b)) => {
                    if self.returns(a) != other.returns(b) {
                        return Ok(Some(format!(
                            "{}: of its callees {} and {}, one can return",
                            name(self, ours),
                            name(self, a),
                            name(other, b)
                        )));
                    }
                    if self.returns(a)
                        && let Some(why) = self.difference(a, other, b)?
                    {
                        return Ok(Some(why));
                    }
                }
                (HoleKind::Data(a), HoleKind::Data(b)) => {
                    if self.symbols[a as usize] != other.symbols[b as usize] {
                        return Ok(Some(format!(
                            "{}: the symbols {} and {}",
                            name(self, ours),
                            self.symbol_name(a),
                            other.symbol_name(b),
                        )));
                    }
                }
                (HoleKind::Call(_), HoleKind::Data(_)) | (HoleKind::Data(_), HoleKind::Call(_)) => {
                    return Ok(Some(format!(
                        "{}: the kinds of its holes",
                        name(self, ours)
                    )));
                }
            }
        }
        Ok(Some(format!(
            "{}: a function that it reaches through its component",
            name(self, ours)
        )))
    }

    /// The hash of `bytes`, which a closure of the module captures: a word that lands in a symbol
    /// counts as an address.
    #[must_use]
    pub fn captures(&self, bytes: &[u8]) -> u128 {
        let mut state = Fingerprint::new();
        state.update(&(bytes.len() as u64).to_le_bytes());
        let (words, tail) = bytes.as_chunks::<4>();
        for word in words {
            let value = u32::from_le_bytes(*word);
            match self.symbol_at(value) {
                Some((symbol, distance)) => {
                    state.update(&distance.to_le_bytes());
                    state.update_hash(self.symbols[symbol as usize]);
                }
                None => state.update(word),
            }
        }
        state.update(tail);
        state.finish()
    }

    /// The symbol that holds the address `value`, and the distance of the address from it.
    fn symbol_at(&self, value: u32) -> Option<(u32, u32)> {
        let after = self
            .by_address
            .partition_point(|(address, _, _)| *address <= value);
        self.by_address[..after]
            .iter()
            .rev()
            .find(|(address, size, _)| value - address < *size)
            .map(|(address, _, index)| (*index, value - address))
    }

    /// The name of the symbol `symbol`.
    fn symbol_name(&self, symbol: u32) -> String {
        match self.module.symbols[symbol as usize] {
            Symbol::Data(data) => data.name.to_owned(),
            Symbol::Absolute { name, address: _ } => name.to_owned(),
            Symbol::Other => symbol.to_string(),
        }
    }

    /// The name of each named function, by function index, as the module spells it.
    #[must_use]
    pub fn names(&self) -> &[(u32, &'a str)] {
        &self.module.names
    }

    /// The functions that each defined function calls with `call`: its function index and the
    /// function indices, in the order of its instructions.
    #[must_use]
    pub fn edges(&self) -> Vec<(u32, Vec<u32>)> {
        let imports = self.module.imports.len() as u32;
        (0_u32..)
            .zip(&self.module.streams)
            .map(|(index, stream)| {
                let callees = stream
                    .control
                    .iter()
                    .filter_map(|control| match control {
                        Control::Call(callee) => Some(*callee),
                        Control::Block
                        | Control::Loop
                        | Control::If
                        | Control::Else
                        | Control::End
                        | Control::Br(_)
                        | Control::BrIf(_)
                        | Control::BrTable(_)
                        | Control::Return
                        | Control::Unreachable => None,
                    })
                    .collect();
                (imports + index, callees)
            })
            .collect()
    }

    /// Whether the function at the function index `function` can return.
    #[must_use]
    pub fn returns(&self, function: u32) -> bool {
        self.returns
            .get(function as usize)
            .copied()
            .unwrap_or(false)
    }

    /// The functions that `function` reaches through `call`s of functions that can return, itself
    /// first.
    #[must_use]
    pub fn reachable(&self, function: u32) -> Vec<u32> {
        let mut found = vec![function];
        let mut index = 0;
        while let Some(&next) = found.get(index) {
            index += 1;
            for callee in callees(&self.module, &self.returns, next) {
                if !found.contains(&callee) {
                    found.push(callee);
                }
            }
        }
        found
    }

    /// Hashes the functions of `component`, which call each other and whatever is finished.
    fn finish(&mut self, component: &[u32]) {
        let indirect = component.iter().any(|&member| {
            self.module.stream(member).is_some_and(|stream| {
                stream.indirect
                    || stream_callees(stream, &self.returns).any(|callee| {
                        self.hashes[callee as usize].is_some_and(|hash| hash.indirect)
                    })
            })
        });
        let mut hashes: Vec<(u32, u128)> = component.iter().map(|member| (*member, 0)).collect();
        for _ in 0..component.len() {
            let round: Vec<(u32, u128)> = component
                .iter()
                .map(|&member| (member, self.body(member, &hashes)))
                .collect();
            hashes = round;
        }
        for (member, hash) in hashes {
            self.hashes[member as usize] = Some(CodeHash { hash, indirect });
        }
    }

    /// The hash of the body of `function`, with `provisional` hashes for the functions of its
    /// component.
    fn body(&self, function: u32, provisional: &[(u32, u128)]) -> u128 {
        let Some(stream) = self.module.stream(function) else {
            return self.import(function);
        };
        let mut state = Fingerprint::new();
        let mut at = 0;
        for hole in &stream.holes {
            state.update(&stream.bytes[at..hole.at]);
            at = hole.at;
            let hash = match hole.kind {
                HoleKind::Call(callee) if !self.returns(callee) => mark::DIVERGES,
                HoleKind::Call(callee) => provisional
                    .iter()
                    .find(|(member, _)| *member == callee)
                    .map(|(_, hash)| *hash)
                    .or_else(|| self.hashes[callee as usize].map(|hash| hash.hash))
                    .unwrap_or_else(|| self.import(callee)),
                HoleKind::Data(symbol) => self.symbols[symbol as usize],
            };
            state.update_hash(hash);
        }
        state.update(&stream.bytes[at..]);
        state.finish()
    }

    /// The hash of the imported function `function`.
    fn import(&self, function: u32) -> u128 {
        self.module
            .imports
            .get(function as usize)
            .map_or(0, |import| Fingerprint::of(&import.identity()))
    }
}

/// Per function index, whether the function can return: the least fixed point of its control flow,
/// where a `call` of a function that cannot yet return ends its path.
fn returning(module: &Module<'_>) -> Vec<bool> {
    let imports = module.imports.len();
    let mut returns = vec![true; imports];
    returns.resize(imports + module.streams.len(), false);
    loop {
        let mut changed = false;
        for (index, stream) in module.streams.iter().enumerate() {
            if !returns[imports + index] && can_return(&stream.control, &returns) {
                returns[imports + index] = true;
                changed = true;
            }
        }
        if !changed {
            return returns;
        }
    }
}

/// A block of a function while the walk of its control flow is inside it.
struct Frame {
    /// Whether a branch that the walk can reach targets the end of the block.
    reached: bool,
    /// Whether the block is a `loop`, whose branches go back to its start.
    is_loop: bool,
    /// Whether the walk can reach the block's start: an `if` without an `else` can fall past it.
    entered: bool,
    /// Whether the block is an `if` whose `else` the walk has not met.
    open_if: bool,
}

/// Whether the control flow `control` of a function can return, where `returns` tells which callees
/// can.
fn can_return(control: &[Control], returns: &[bool]) -> bool {
    let mut frames = vec![Frame {
        reached: false,
        is_loop: false,
        entered: true,
        open_if: false,
    }];
    let mut live = true;
    let mark_label = |frames: &mut [Frame], label: u32| {
        let index = frames.len().checked_sub(label as usize + 1);
        if let Some(frame) = index.and_then(|index| frames.get_mut(index))
            && !frame.is_loop
        {
            frame.reached = true;
        }
    };
    for instruction in control {
        match instruction {
            Control::Block | Control::Loop | Control::If => frames.push(Frame {
                reached: false,
                is_loop: matches!(instruction, Control::Loop),
                entered: live,
                open_if: matches!(instruction, Control::If),
            }),
            Control::Else => {
                if let Some(frame) = frames.last_mut() {
                    frame.reached |= live;
                    frame.open_if = false;
                    live = frame.entered;
                }
            }
            Control::End => {
                if let Some(frame) = frames.pop() {
                    live = live || frame.reached || (frame.open_if && frame.entered);
                }
            }
            Control::Br(label) => {
                if live {
                    mark_label(&mut frames, *label);
                }
                live = false;
            }
            Control::BrIf(label) => {
                if live {
                    mark_label(&mut frames, *label);
                }
            }
            Control::BrTable(labels) => {
                if live {
                    for label in labels {
                        mark_label(&mut frames, *label);
                    }
                }
                live = false;
            }
            Control::Return => {
                if live {
                    return true;
                }
            }
            Control::Unreachable => live = false,
            Control::Call(callee) => {
                if !returns.get(*callee as usize).copied().unwrap_or(false) {
                    live = false;
                }
            }
        }
    }
    live
}
