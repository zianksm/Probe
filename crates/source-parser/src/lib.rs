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

pub type FullSourceItem = SourceItem<isize, isize, isize, char, isize>;
pub type PartialSourceItem =
    SourceItem<Option<isize>, Option<isize>, Option<isize>, Option<char>, Option<isize>>;

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

        s.s = ref_char.get(index::S).map(|a|a.parse::<isize>().ok()).flatten();
        s.l = ref_char.get(index::L).map(|a|a.parse::<isize>().ok()).flatten();
        s.f = ref_char.get(index::F).map(|a|a.parse::<isize>().ok()).flatten();
        s.j = ref_char.get(index::J).map(|a|a.parse::<char>().ok()).flatten();
        s.m = ref_char.get(index::M).map(|a|a.parse::<isize>().ok()).flatten();

        s
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a full source item with explicit values
    fn create_full_source_item(s: isize, l: isize, f: isize, j: char, m: isize) -> FullSourceItem {
        FullSourceItem { s, l, f, j, m }
    }

    #[test]
    fn test_complete_entry() {
        let input = "38:124:1:i:2";
        let partial = PartialSourceItem::from_raw(input);
        
        assert_eq!(partial.s, Some(38));
        assert_eq!(partial.l, Some(124));
        assert_eq!(partial.f, Some(1));
        assert_eq!(partial.j, Some('i'));
        assert_eq!(partial.m, Some(2));
        
        let prev = create_full_source_item(0, 0, 0, '-', 0);
        let full = FullSourceItem::from_partial(&prev, partial);
        
        assert_eq!(full.s, 38);
        assert_eq!(full.l, 124);
        assert_eq!(full.f, 1);
        assert_eq!(full.j, 'i');
        assert_eq!(full.m, 2);
    }
    
    #[test]
    fn test_missing_source() {
        let input = ":68:0:o:";
        let partial = PartialSourceItem::from_raw(input);
        
        assert_eq!(partial.s, None);
        assert_eq!(partial.l, Some(68));
        assert_eq!(partial.f, Some(0));
        assert_eq!(partial.j, Some('o'));
        assert_eq!(partial.m, None);
        
        let prev = create_full_source_item(38, 124, 1, 'i', 2);
        let full = FullSourceItem::from_partial(&prev, partial);
        
        assert_eq!(full.s, 38); // Inherited
        assert_eq!(full.l, 68);
        assert_eq!(full.f, 0);
        assert_eq!(full.j, 'o');
        assert_eq!(full.m, 2); // Inherited
    }
    
    #[test]
    fn test_missing_source_and_length() {
        let input = "::3:i:";
        let partial = PartialSourceItem::from_raw(input);
        
        assert_eq!(partial.s, None);
        assert_eq!(partial.l, None);
        assert_eq!(partial.f, Some(3));
        assert_eq!(partial.j, Some('i'));
        assert_eq!(partial.m, None);
        
        let prev = create_full_source_item(52, 75, 2, '-', 0);
        let full = FullSourceItem::from_partial(&prev, partial);
        
        assert_eq!(full.s, 52); // Inherited
        assert_eq!(full.l, 75); // Inherited
        assert_eq!(full.f, 3);
        assert_eq!(full.j, 'i');
        assert_eq!(full.m, 0); // Inherited
    }
    
    #[test]
    fn test_missing_multiple_fields() {
        let input = "73:::";
        let partial = PartialSourceItem::from_raw(input);
        
        assert_eq!(partial.s, Some(73));
        assert_eq!(partial.l, None);
        assert_eq!(partial.f, None);
        assert_eq!(partial.j, None);
        assert_eq!(partial.m, None);
        
        let prev = create_full_source_item(52, 75, 3, 'i', 0);
        let full = FullSourceItem::from_partial(&prev, partial);
        
        assert_eq!(full.s, 73);
        assert_eq!(full.l, 75); // Inherited
        assert_eq!(full.f, 3);  // Inherited
        assert_eq!(full.j, 'i'); // Inherited
        assert_eq!(full.m, 0);  // Inherited
    }
    
    #[test]
    fn test_just_source() {
        let input = "21";
        let partial = PartialSourceItem::from_raw(input);
        
        assert_eq!(partial.s, Some(21));
        assert_eq!(partial.l, None);
        assert_eq!(partial.f, None);
        assert_eq!(partial.j, None);
        assert_eq!(partial.m, None);
        
        let prev = create_full_source_item(85, 96, 1, 'o', 3);
        let full = FullSourceItem::from_partial(&prev, partial);
        
        assert_eq!(full.s, 21);
        assert_eq!(full.l, 96); // Inherited
        assert_eq!(full.f, 1);  // Inherited
        assert_eq!(full.j, 'o'); // Inherited
        assert_eq!(full.m, 3);  // Inherited
    }
    
    #[test]
    fn test_missing_trailing_fields() {
        let input = "38:124:1";
        let partial = PartialSourceItem::from_raw(input);
        
        assert_eq!(partial.s, Some(38));
        assert_eq!(partial.l, Some(124));
        assert_eq!(partial.f, Some(1));
        assert_eq!(partial.j, None);
        assert_eq!(partial.m, None);
        
        let prev = create_full_source_item(0, 0, 0, '-', 0);
        let full = FullSourceItem::from_partial(&prev, partial);
        
        assert_eq!(full.s, 38);
        assert_eq!(full.l, 124);
        assert_eq!(full.f, 1);
        assert_eq!(full.j, '-'); // Inherited
        assert_eq!(full.m, 0);   // Inherited
    }
    
    #[test]
    fn test_full_sourcemap_parsing() {
        // Let's parse this complex source map:
        // 38:124:1:i:2;:68:0:o:;52:75:2:-:0;::3:i:;73:::;85:96:1:o:3;21
        let source_map_str = "38:124:1:i:2;:68:0:o:;52:75:2:-:0;::3:i:;73:::;85:96:1:o:3;21";
        let entries = source_map_str.split(';').collect::<Vec<_>>();
        
        let mut prev = create_full_source_item(0, 0, 0, '-', 0);
        let mut results = Vec::new();
        
        for entry in entries {
            let partial = PartialSourceItem::from_raw(entry);
            let full = FullSourceItem::from_partial(&prev, partial);
            results.push(full.clone());
            prev = full;
        }
        
        // Verify each entry in the parsed results
        assert_eq!(results[0], create_full_source_item(38, 124, 1, 'i', 2));
        assert_eq!(results[1], create_full_source_item(38, 68, 0, 'o', 2));
        assert_eq!(results[2], create_full_source_item(52, 75, 2, '-', 0));
        assert_eq!(results[3], create_full_source_item(52, 75, 3, 'i', 0));
        assert_eq!(results[4], create_full_source_item(73, 75, 3, 'i', 0));
        assert_eq!(results[5], create_full_source_item(85, 96, 1, 'o', 3));
        assert_eq!(results[6], create_full_source_item(21, 96, 1, 'o', 3));
    }
    
    #[test]
    fn test_inheritance_chain() {
        // 10:20:1:i:2;::2:o:;:::i:3;::::;75
        let source_map_str = "10:20:1:i:2;::2:o:;:::i:3;::::;75";
        let entries = source_map_str.split(';').collect::<Vec<_>>();
        
        let mut prev = create_full_source_item(0, 0, 0, '-', 0);
        let mut results = Vec::new();
        
        for entry in entries {
            let partial = PartialSourceItem::from_raw(entry);
            let full = FullSourceItem::from_partial(&prev, partial);
            results.push(full.clone());
            prev = full;
        }
        
        // Verify each entry
        assert_eq!(results[0], create_full_source_item(10, 20, 1, 'i', 2));
        assert_eq!(results[1], create_full_source_item(10, 20, 2, 'o', 2));
        assert_eq!(results[2], create_full_source_item(10, 20, 2, 'i', 3));
        assert_eq!(results[3], create_full_source_item(10, 20, 2, 'i', 3)); // All fields inherit
        assert_eq!(results[4], create_full_source_item(75, 20, 2, 'i', 3)); // Only s changes
    }
    
    #[test]
    fn test_intermittent_entries() {
        // 38:124:1:i:2;:68:::;52::::4;:75:2:-:;:::::5
        let source_map_str = "38:124:1:i:2;:68:::;52::::4;:75:2:-:;::::5";
        let entries = source_map_str.split(';').collect::<Vec<_>>();
        
        let mut prev = create_full_source_item(0, 0, 0, '-', 0);
        let mut results = Vec::new();
        
        for entry in entries {
            let partial = PartialSourceItem::from_raw(entry);
            let full = FullSourceItem::from_partial(&prev, partial);
            results.push(full.clone());
            prev = full;
        }
        
        // Verify each entry
        assert_eq!(results[0], create_full_source_item(38, 124, 1, 'i', 2));
        assert_eq!(results[1], create_full_source_item(38, 68, 1, 'i', 2));
        assert_eq!(results[2], create_full_source_item(52, 68, 1, 'i', 4));
        assert_eq!(results[3], create_full_source_item(52, 75, 2, '-', 4));
        assert_eq!(results[4], create_full_source_item(52, 75, 2, '-', 5));
    }
}