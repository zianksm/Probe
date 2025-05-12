mod index {

    pub const S: usize = 0;
    pub const L: usize = 1;
    pub const F: usize = 2;
    pub const J: usize = 3;
    pub const M: usize = 4;
}

pub struct SourceMap {
    raw: String,
}

/// struct corresponding to a full source item in a source map. e.g `167:58:0:i:2`
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SourceItem<S, L, F, J, M> {
    /// source offset
    /// Byte offset into the source file - where in your code this bytecode instruction originated
    /// Non-negative integer<br>• Empty (inherits previous)
    pub s: S,
    /// length
    /// Length in bytes of the source segment - how many bytes of your source code correspond to this instruction
    ///  Non-negative integer<br>• Empty (inherits previous)
    pub l: L,
    /// Source file index - which file in a multi-file project (0 for single files)
    /// Non-negative integer<br>• Empty (inherits previous)
    pub f: F,
    ///Jump type:<br>• "i" = jump into a function<br>• "o" = jump out of a function<br>• "-"/empty = regular execution or inherits previous
    pub j: J,
    /// Modifier depth - how deeply nested this code is within Solidity modifiers
    /// Non-negative integer<br>• Empty (inherits previous)
    pub m: M,
}

pub type FullSourceItem = SourceItem<usize, usize, usize, char, usize>;
pub type PartialSourceItem =
    SourceItem<Option<usize>, Option<usize>, Option<usize>, Option<char>, Option<usize>>;

impl FullSourceItem {
    pub fn from_partial(prev: &FullSourceItem, current: PartialSourceItem) -> Self {
        let mut s = Self::default();

        s.s = current.s.unwrap_or_else(|| prev.s);
        s.l = current.l.unwrap_or_else(|| prev.l);
        s.f = current.f.unwrap_or_else(|| prev.f);
        s.j = current.j.unwrap_or_else(|| prev.j);
        s.m = current.m.unwrap_or_else(|| prev.m);
        
        s
    }
}

impl PartialSourceItem {
    pub fn from_raw(ref_str: &str) -> PartialSourceItem {
        let ref_char = ref_str.split(":").collect::<Vec<_>>();

        let mut s = PartialSourceItem::default();

        s.s = ref_char[index::S].parse::<usize>().ok();
        s.l = ref_char[index::L].parse::<usize>().ok();
        s.f = ref_char[index::F].parse::<usize>().ok();
        s.j = ref_char[index::J].parse::<char>().ok();
        s.m = ref_char[index::M].parse::<usize>().ok();

        s
    }
}

// impl TryFrom<Vec<PartialSourceItem>> for Vec<FullSourceItem> {
//     type Error;

//     fn try_from(value: Vec<PartialSourceItem>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

#[cfg(test)]
mod test {

    #[test]
    pub fn a() {
        let a = "::::";
        let b = a.split(":").collect::<Vec<_>>();
        panic!("{:#?}", b);
    }
}
