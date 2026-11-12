use crate::diagnostic::{Diagnostic, Result};
use crate::limits::{MAX_OBJECT_BYTES, check_budget};

const SHT_PROGBITS: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_DYNAMIC: u32 = 6;
const SHT_NOBITS: u32 = 8;
const SHT_REL: u32 = 9;
const SHT_DYNSYM: u32 = 11;
const SHT_INIT_ARRAY: u32 = 14;
const SHT_FINI_ARRAY: u32 = 15;
const SHT_PREINIT_ARRAY: u32 = 16;
const SHT_GROUP: u32 = 17;
const SHF_WRITE: u64 = 0x1;
const SHF_ALLOC: u64 = 0x2;
const SHF_EXECINSTR: u64 = 0x4;
const SHF_GROUP: u64 = 0x200;
const SHF_TLS: u64 = 0x400;
const SHF_COMPRESSED: u64 = 0x800;

struct Section {
    name: u32,
    kind: u32,
    flags: u64,
    address: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    alignment: u64,
    entry_size: u64,
}

fn invalid(detail: &str) -> Diagnostic {
    Diagnostic::global("E_OUTPUT", format!("invalid ELF64 object: {detail}"))
}

fn unsupported(detail: &str) -> Diagnostic {
    Diagnostic::global(
        "E_OUTPUT",
        format!("unsupported assembler output: {detail}"),
    )
}

// These fixed-width reads are used only inside a validated 64-byte header,
// 64-byte section entry, or 24-byte symbol entry.
fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn file_range(bytes: &[u8], offset: u64, size: u64) -> Result<&[u8]> {
    let start = usize::try_from(offset).map_err(|_| invalid("file offset exceeds host range"))?;
    let size = usize::try_from(size).map_err(|_| invalid("file size exceeds host range"))?;
    let end = start
        .checked_add(size)
        .ok_or_else(|| invalid("file range overflows"))?;
    bytes
        .get(start..end)
        .ok_or_else(|| invalid("file range extends beyond assembler output"))
}

fn string_table<'a>(bytes: &'a [u8], section: &Section) -> Result<&'a [u8]> {
    if section.kind != SHT_STRTAB || section.flags & SHF_COMPRESSED != 0 {
        return Err(invalid("expected an uncompressed string table"));
    }
    let table = file_range(bytes, section.offset, section.size)?;
    if !table.is_empty() && (table.first() != Some(&0) || table.last() != Some(&0)) {
        return Err(invalid("string table must start and end with NUL"));
    }
    Ok(table)
}

// Only call with a table returned by string_table: its final NUL ensures every
// in-range index has a terminator, including suffix and non-UTF-8 byte names.
// Return the tail without rescanning for NUL; callers compare only fixed names.
fn string_tail(table: &[u8], offset: u32) -> Result<&[u8]> {
    if table.is_empty() && offset == 0 {
        return Ok(b"");
    }
    table
        .get(offset as usize..)
        .filter(|tail| !tail.is_empty())
        .ok_or_else(|| invalid("string index is outside its table"))
}

pub(crate) fn verify_elf64_rel_x86_64_bytes(bytes: &[u8]) -> Result<()> {
    check_budget(
        bytes.len(),
        MAX_OBJECT_BYTES,
        "assembler object bytes",
        None,
    )?;
    if bytes.len() < 64
        || bytes.get(0..4) != Some(b"\x7fELF")
        || bytes[4] != 2
        || bytes[5] != 1
        || bytes[6] != 1
        || u16_at(bytes, 16) != 1
        || u16_at(bytes, 18) != 62
        || u32_at(bytes, 20) != 1
        || u16_at(bytes, 52) != 64
    {
        return Err(Diagnostic::global(
            "E_OUTPUT",
            "assembler output is not an ELF64 little-endian x86-64 ET_REL object",
        ));
    }

    // The producer emits linkable functions without process entry points or
    // program headers. These are output-profile limits, not general ELF rules.
    if u64_at(bytes, 24) != 0 {
        return Err(unsupported("entry points are unsupported"));
    }
    if u64_at(bytes, 32) != 0 || u16_at(bytes, 56) != 0 {
        return Err(unsupported("program headers are unsupported"));
    }

    let section_offset = u64_at(bytes, 40);
    let section_count = u16_at(bytes, 60) as usize;
    let names_index = u16_at(bytes, 62) as usize;
    if section_offset < 64
        || u16_at(bytes, 58) != 64
        || section_count == 0
        || section_count >= 0xff00
    {
        return Err(invalid(
            "missing, unsupported, or invalid section header table",
        ));
    }
    if names_index == 0xffff {
        return Err(invalid("extended section names index is unsupported"));
    }
    let table = file_range(bytes, section_offset, (section_count as u64) * 64)?;
    let sections: Vec<_> = table
        .as_chunks::<64>()
        .0
        .iter()
        .map(|entry| Section {
            name: u32_at(entry, 0),
            kind: u32_at(entry, 4),
            flags: u64_at(entry, 8),
            address: u64_at(entry, 16),
            offset: u64_at(entry, 24),
            size: u64_at(entry, 32),
            link: u32_at(entry, 40),
            info: u32_at(entry, 44),
            alignment: u64_at(entry, 48),
            entry_size: u64_at(entry, 56),
        })
        .collect();
    if table[..64].iter().any(|byte| *byte != 0) {
        return Err(invalid("section zero must be empty for ordinary numbering"));
    }
    let mut occupied = Vec::new();
    for section in &sections {
        if section.kind == 0 {
            continue;
        }
        // Section groups can make a linker discard a valid COMDAT group,
        // including the required entry point. The current producer emits no
        // groups, so reject this valid but unsupported output shape.
        if section.kind == SHT_GROUP || section.flags & SHF_GROUP != 0 {
            return Err(unsupported("section groups are unsupported"));
        }
        // SHF_TLS adds per-execution-flow storage to the linked object. AIC
        // has no thread-local state, so this valid but unsupported shape must
        // not be installed for the downstream consumer.
        if section.flags & SHF_TLS != 0 {
            return Err(unsupported("thread-local storage is unsupported"));
        }
        // These sections add function calls to the linked program's
        // initialization or termination sequence. The current producer emits
        // none of them, so treat the valid ELF types as unsupported output.
        if matches!(
            section.kind,
            SHT_INIT_ARRAY | SHT_FINI_ARRAY | SHT_PREINIT_ARRAY
        ) {
            return Err(unsupported(
                "initialization and termination arrays are unsupported",
            ));
        }
        // AIC has no memory or global state. Nonempty sections allocated at
        // runtime must therefore be read-only executable code. Keep the
        // producer's empty .data/.bss sections, but reject allocated storage
        // and writable code before installing the object.
        if section.size != 0
            && section.flags & SHF_ALLOC != 0
            && (section.flags & SHF_EXECINSTR == 0 || section.flags & SHF_WRITE != 0)
        {
            return Err(unsupported(
                "writable or non-code allocated sections are unsupported",
            ));
        }
        // The current producer uses immediate constants and stack slots, with
        // branches resolved within one section. These are output-shape limits,
        // not restrictions imposed on general ELF relocatable objects.
        if matches!(section.kind, SHT_REL | SHT_RELA) {
            return Err(unsupported("relocation sections are unsupported"));
        }
        if matches!(section.kind, SHT_DYNAMIC | SHT_DYNSYM) {
            return Err(unsupported("dynamic tables are unsupported"));
        }
        if section.alignment != 0 && !section.alignment.is_power_of_two() {
            return Err(invalid("section alignment is not zero or a power of two"));
        }
        if section.alignment > 1 && section.address % section.alignment != 0 {
            return Err(invalid("section address violates its alignment"));
        }
        // NOBITS has a conceptual offset and consumes no bytes in the file.
        if section.kind != SHT_NOBITS {
            file_range(bytes, section.offset, section.size)?;
            if section.size != 0 {
                occupied.push((section.offset, section.offset + section.size));
            }
        }
    }
    occupied.sort_unstable();
    if occupied.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err(invalid("section payloads overlap"));
    }
    if names_index != 0 {
        let names = sections
            .get(names_index)
            .ok_or_else(|| invalid("section names index is outside the section table"))?;
        let names = string_table(bytes, names)?;
        for section in sections.iter().filter(|s| s.kind != 0) {
            string_tail(names, section.name)?;
        }
    }

    let mut symbol_table_seen = false;
    let mut entry_found = false;
    for section in sections.iter().filter(|s| s.kind == SHT_SYMTAB) {
        if symbol_table_seen || section.entry_size != 24 || section.size % 24 != 0 {
            return Err(invalid("unsupported or invalid symbol table layout"));
        }
        symbol_table_seen = true;
        if section.flags & SHF_COMPRESSED != 0 {
            return Err(invalid("compressed symbol tables are unsupported"));
        }
        let symbols = file_range(bytes, section.offset, section.size)?;
        if symbols.len() < 24 || symbols[..24].iter().any(|byte| *byte != 0) {
            return Err(invalid("missing or invalid null symbol"));
        }
        let first_nonlocal = section.info as usize;
        if first_nonlocal == 0 || first_nonlocal > symbols.len() / 24 {
            return Err(invalid("invalid first non-local symbol index"));
        }
        let strings = sections
            .get(section.link as usize)
            .ok_or_else(|| invalid("symbol table string-table link is out of range"))?;
        let strings = string_table(bytes, strings)?;
        for (index, symbol) in symbols.as_chunks::<24>().0.iter().enumerate() {
            let binding = symbol[4] >> 4;
            if (binding == 0) != (index < first_nonlocal) {
                return Err(invalid("symbol binding disagrees with sh_info ordering"));
            }
            let name = string_tail(strings, u32_at(symbol, 0))?;
            let target_index = u16_at(symbol, 6) as usize;
            // Symbol zero is the required undefined null entry. No other
            // unresolved symbols or linker-allocated common storage are emitted.
            if index != 0 && target_index == 0 {
                return Err(unsupported("undefined symbols are unsupported"));
            }
            if target_index == 0xfff2 || symbol[4] & 0xf == 5 {
                return Err(unsupported("common symbols are unsupported"));
            }
            if target_index == 0xffff {
                return Err(invalid("extended symbol section indexes are unsupported"));
            }
            if target_index < 0xff00 && target_index >= sections.len() {
                return Err(invalid("symbol section index is out of range"));
            }
            if name.starts_with(b"aic_main\0") {
                if entry_found || binding != 1 || symbol[4] & 0xf != 2 || symbol[5] & 3 != 0 {
                    return Err(invalid(
                        "aic_main must be one global default-visible function",
                    ));
                }
                let code = sections
                    .get(target_index)
                    .filter(|_| target_index != 0 && target_index < 0xff00)
                    .ok_or_else(|| invalid("aic_main is not defined in an ordinary section"))?;
                let value = u64_at(symbol, 8);
                let size = u64_at(symbol, 16);
                if code.kind != SHT_PROGBITS
                    || code.flags & (SHF_ALLOC | SHF_EXECINSTR) != (SHF_ALLOC | SHF_EXECINSTR)
                    || code.flags & SHF_COMPRESSED != 0
                    || size == 0
                    || value.checked_add(size).is_none_or(|end| end > code.size)
                {
                    return Err(invalid("aic_main is outside allocated executable code"));
                }
                entry_found = true;
            }
        }
    }
    if !entry_found {
        return Err(invalid("missing defined aic_main function"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: usize = 168;
    const TEXT: usize = TABLE + 64;
    const STRINGS: usize = TABLE + 128;
    const SYMBOLS: usize = TABLE + 192;
    const NAMES: usize = TABLE + 256;

    fn put16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn put64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn object() -> Vec<u8> {
        let mut bytes = vec![0; TABLE + 5 * 64];
        bytes[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        put16(&mut bytes, 16, 1);
        put16(&mut bytes, 18, 62);
        put32(&mut bytes, 20, 1);
        put64(&mut bytes, 40, TABLE as u64);
        put16(&mut bytes, 52, 64);
        put16(&mut bytes, 58, 64);
        put16(&mut bytes, 60, 5);
        put16(&mut bytes, 62, 4);
        bytes[64] = 0xc3;
        bytes[65..75].copy_from_slice(b"\0aic_main\0");
        bytes[128..161].copy_from_slice(b"\0.text\0.strtab\0.symtab\0.shstrtab\0");
        for (at, name, kind, offset, size, align) in [
            (TEXT, 1, SHT_PROGBITS, 64, 1, 1),
            (STRINGS, 7, SHT_STRTAB, 65, 10, 1),
            (SYMBOLS, 15, SHT_SYMTAB, 80, 48, 8),
            (NAMES, 23, SHT_STRTAB, 128, 33, 1),
        ] {
            put32(&mut bytes, at, name);
            put32(&mut bytes, at + 4, kind);
            put64(&mut bytes, at + 24, offset);
            put64(&mut bytes, at + 32, size);
            put64(&mut bytes, at + 48, align);
        }
        put64(&mut bytes, TEXT + 8, 6);
        put32(&mut bytes, SYMBOLS + 40, 2);
        put32(&mut bytes, SYMBOLS + 44, 1);
        put64(&mut bytes, SYMBOLS + 56, 24);
        put32(&mut bytes, 104, 1);
        bytes[108] = 0x12;
        put16(&mut bytes, 110, 1);
        put64(&mut bytes, 120, 1);
        bytes
    }

    fn with_extra_section(kind: u32, flags: u64, size: u64) -> Vec<u8> {
        let mut bytes = object();
        let section_table_end = TABLE + 6 * 64;
        let payload_size = if kind == SHT_NOBITS {
            0
        } else {
            usize::try_from(size).unwrap()
        };
        bytes.resize(section_table_end + payload_size, 0);
        put16(&mut bytes, 60, 6);

        let section = TABLE + 5 * 64;
        put32(&mut bytes, section + 4, kind);
        put64(&mut bytes, section + 8, flags);
        put64(&mut bytes, section + 24, section_table_end as u64);
        put64(&mut bytes, section + 32, size);
        put64(&mut bytes, section + 48, 1);
        bytes
    }

    fn with_extra_symbol(symbol: &[u8]) -> Vec<u8> {
        let mut bytes = object();
        let names = bytes[128..161].to_vec();
        let names_offset = bytes.len();
        bytes.extend_from_slice(&names);
        put64(&mut bytes, NAMES + 24, names_offset as u64);
        put64(&mut bytes, SYMBOLS + 32, 72);
        bytes[128..152].copy_from_slice(symbol);
        bytes
    }

    #[test]
    fn object_byte_budget_counts_inactive_trailing_bytes() {
        let mut bytes = object();
        bytes.resize(MAX_OBJECT_BYTES, 0);
        verify_elf64_rel_x86_64_bytes(&bytes).unwrap();
        bytes.push(0);
        assert_eq!(
            verify_elf64_rel_x86_64_bytes(&bytes)
                .err()
                .map(|error| error.code),
            Some("E_LIMIT")
        );
    }

    #[test]
    fn entry_and_program_headers_follow_current_object_profile() {
        let mut bytes = object();
        // ELF readers ignore EI_PAD. An absent program table has no entries
        // whose size needs interpreting, even when e_phentsize is nonzero.
        bytes[9..16].fill(0xff);
        put16(&mut bytes, 54, 56);
        verify_elf64_rel_x86_64_bytes(&bytes).unwrap();

        for (name, offset, mutation, detail) in [
            (
                "entry point",
                24,
                1u64.to_le_bytes().to_vec(),
                "entry points",
            ),
            (
                "program table offset",
                32,
                64u64.to_le_bytes().to_vec(),
                "program headers",
            ),
            (
                "program table count",
                56,
                1u16.to_le_bytes().to_vec(),
                "program headers",
            ),
            (
                "extended program table count",
                56,
                0xffffu16.to_le_bytes().to_vec(),
                "program headers",
            ),
        ] {
            let mut bytes = object();
            bytes[offset..offset + mutation.len()].copy_from_slice(&mutation);
            let error = verify_elf64_rel_x86_64_bytes(&bytes).unwrap_err();
            assert_eq!(error.code, "E_OUTPUT", "{name}: {error}");
            assert!(error.detail.starts_with("unsupported assembler output:"));
            assert!(error.detail.contains(detail), "{name}: {error}");
        }
    }

    #[test]
    fn section_address_must_satisfy_section_alignment() {
        for alignment in [2, 8] {
            let mut bytes = object();
            put64(&mut bytes, TEXT + 16, 1);
            put64(&mut bytes, TEXT + 48, alignment);
            let error = verify_elf64_rel_x86_64_bytes(&bytes).unwrap_err();
            assert_eq!(error.code, "E_OUTPUT");
            assert!(
                error
                    .detail
                    .contains("section address violates its alignment")
            );
        }

        // ELF defines alignment values zero and one as no constraint.
        for alignment in [0, 1] {
            let mut bytes = object();
            put64(&mut bytes, TEXT + 16, 1);
            put64(&mut bytes, TEXT + 48, alignment);
            verify_elf64_rel_x86_64_bytes(&bytes).unwrap();
        }
    }

    #[test]
    fn section_groups_are_outside_the_object_profile() {
        let mut group = object();
        group.resize(TABLE + 6 * 64, 0);
        put16(&mut group, 60, 6);
        put32(&mut group, TABLE + 5 * 64 + 4, SHT_GROUP);

        let mut grouped_member = object();
        put64(&mut grouped_member, TEXT + 8, SHF_GROUP);

        for bytes in [group, grouped_member] {
            let error = verify_elf64_rel_x86_64_bytes(&bytes).unwrap_err();
            assert_eq!(error.code, "E_OUTPUT");
            assert!(error.detail.starts_with("unsupported assembler output:"));
            assert!(error.detail.contains("section groups are unsupported"));
        }
    }

    #[test]
    fn initialization_and_termination_arrays_are_outside_the_object_profile() {
        for kind in [SHT_INIT_ARRAY, SHT_FINI_ARRAY, SHT_PREINIT_ARRAY] {
            let mut bytes = object();
            bytes.resize(TABLE + 6 * 64, 0);
            put16(&mut bytes, 60, 6);
            put32(&mut bytes, TABLE + 5 * 64 + 4, kind);

            let error = verify_elf64_rel_x86_64_bytes(&bytes).unwrap_err();
            assert_eq!(error.code, "E_OUTPUT", "section type {kind}: {error}");
            assert!(error.detail.starts_with("unsupported assembler output:"));
            assert!(
                error
                    .detail
                    .contains("initialization and termination arrays are unsupported"),
                "section type {kind}: {error}"
            );
        }
    }

    #[test]
    fn thread_local_storage_flags_are_outside_the_object_profile() {
        let mut bytes = object();
        put64(&mut bytes, TEXT + 8, 6 | SHF_TLS);

        let error = verify_elf64_rel_x86_64_bytes(&bytes).unwrap_err();
        assert_eq!(error.code, "E_OUTPUT");
        assert!(error.detail.starts_with("unsupported assembler output:"));
        assert!(error.detail.contains("thread-local storage is unsupported"));
    }

    #[test]
    fn nonempty_writable_or_non_code_allocated_sections_are_outside_the_object_profile() {
        for (kind, flags, size) in [
            (SHT_PROGBITS, SHF_ALLOC, 8),
            (SHT_PROGBITS, SHF_ALLOC | SHF_WRITE, 8),
            (SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR | SHF_WRITE, 8),
            (SHT_NOBITS, SHF_ALLOC | SHF_WRITE, 8),
        ] {
            let bytes = with_extra_section(kind, flags, size);
            let error = verify_elf64_rel_x86_64_bytes(&bytes).unwrap_err();
            assert_eq!(error.code, "E_OUTPUT");
            assert!(error.detail.starts_with("unsupported assembler output:"));
            assert!(
                error
                    .detail
                    .contains("writable or non-code allocated sections are unsupported")
            );
        }
    }

    #[test]
    fn executable_code_and_empty_or_nonallocated_sections_remain_accepted() {
        for bytes in [
            with_extra_section(SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR, 1),
            with_extra_section(SHT_PROGBITS, SHF_ALLOC | SHF_WRITE, 0),
            with_extra_section(SHT_PROGBITS, SHF_WRITE, 8),
        ] {
            verify_elf64_rel_x86_64_bytes(&bytes).unwrap();
        }
    }

    #[test]
    fn accepts_function_shape_nobits_and_string_table_rules() {
        let mut bytes = object();
        verify_elf64_rel_x86_64_bytes(&bytes).unwrap();
        // NOBITS occupies no file payload, even with a nonzero conceptual size.
        bytes.resize(TABLE + 6 * 64, 0);
        put16(&mut bytes, 60, 6);
        put32(&mut bytes, TABLE + 5 * 64 + 4, SHT_NOBITS);
        put64(&mut bytes, TABLE + 5 * 64 + 24, u64::MAX);
        put64(&mut bytes, TABLE + 5 * 64 + 32, u64::MAX);
        verify_elf64_rel_x86_64_bytes(&bytes).unwrap();

        // Other fields of an inactive SHT_NULL entry are undefined.
        put32(&mut bytes, TABLE + 5 * 64 + 4, 0);
        put32(&mut bytes, TABLE + 5 * 64, u32::MAX);
        put64(&mut bytes, TABLE + 5 * 64 + 48, 3);
        verify_elf64_rel_x86_64_bytes(&bytes).unwrap();

        // ELF permits absent section names, suffix sharing, and non-UTF-8 names.
        put16(&mut bytes, 62, 0);
        for index in 0..6 {
            put32(&mut bytes, TABLE + index * 64, 0);
        }
        verify_elf64_rel_x86_64_bytes(&bytes).unwrap();
        assert_eq!(string_tail(b"\0prefixsuffix\0", 7).unwrap(), b"suffix\0");
        assert_eq!(string_tail(b"\0\xff\0", 1).unwrap(), b"\xff\0");
        assert_eq!(string_tail(b"", 0).unwrap(), b"");
        assert!(string_tail(b"", 1).is_err());
        assert_eq!(string_tail(b"\0", 0).unwrap(), b"\0");
        assert!(string_tail(b"\0", 1).is_err());
    }

    #[test]
    fn entry_name_matching_preserves_nul_suffix_and_byte_semantics() {
        for (strings, offset, accepted) in [
            (b"\0aic_main\0".as_slice(), 1, true),
            (b"\0prefixaic_main\0".as_slice(), 7, true),
            (b"\0\xffaic_main\0".as_slice(), 2, true),
            (b"\0aic_main_extra\0".as_slice(), 1, false),
            (b"\0aic_mainx\0".as_slice(), 1, false),
            (b"\0aic_main\0".as_slice(), 9, false),
            (b"\0aic_main\0".as_slice(), 10, false),
            (b"".as_slice(), 0, false),
        ] {
            let mut bytes = object();
            let start = bytes.len();
            bytes.extend_from_slice(strings);
            put64(&mut bytes, STRINGS + 24, start as u64);
            put64(&mut bytes, STRINGS + 32, strings.len() as u64);
            put32(&mut bytes, 104, offset);
            assert_eq!(
                verify_elf64_rel_x86_64_bytes(&bytes).is_ok(),
                accepted,
                "{strings:?} offset={offset}"
            );
        }
    }

    #[test]
    fn rejects_malformed_tables_references_and_function_symbols() {
        let cases = [
            (
                "section offset overflow",
                40,
                u64::MAX.to_le_bytes().to_vec(),
            ),
            ("section entry size", 58, 63u16.to_le_bytes().to_vec()),
            ("extended section count", 60, 0u16.to_le_bytes().to_vec()),
            (
                "reserved section count",
                60,
                0xff00u16.to_le_bytes().to_vec(),
            ),
            ("section names index", 62, 5u16.to_le_bytes().to_vec()),
            ("extended names index", 62, 0xffffu16.to_le_bytes().to_vec()),
            ("null section", TABLE + 4, 1u32.to_le_bytes().to_vec()),
            ("null section flags", TABLE + 8, 1u64.to_le_bytes().to_vec()),
            ("payload offset", TEXT + 24, u64::MAX.to_le_bytes().to_vec()),
            ("payload size", TEXT + 32, u64::MAX.to_le_bytes().to_vec()),
            ("alignment", TEXT + 48, 3u64.to_le_bytes().to_vec()),
            (
                "payload overlap",
                STRINGS + 24,
                64u64.to_le_bytes().to_vec(),
            ),
            ("section name", TEXT, 33u32.to_le_bytes().to_vec()),
            ("string table final NUL", 74, vec![1]),
            ("symbol string index", 104, 10u32.to_le_bytes().to_vec()),
            (
                "symbol table link",
                SYMBOLS + 40,
                5u32.to_le_bytes().to_vec(),
            ),
            (
                "symbol table link type",
                SYMBOLS + 40,
                1u32.to_le_bytes().to_vec(),
            ),
            (
                "symbol entry size",
                SYMBOLS + 56,
                23u64.to_le_bytes().to_vec(),
            ),
            (
                "symbol table size",
                SYMBOLS + 32,
                47u64.to_le_bytes().to_vec(),
            ),
            ("nonlocal index", SYMBOLS + 44, 3u32.to_le_bytes().to_vec()),
            ("binding order", SYMBOLS + 44, 2u32.to_le_bytes().to_vec()),
            ("null symbol", 80, vec![1]),
            ("weak function", 108, vec![0x22]),
            ("wrong symbol type", 108, vec![0x11]),
            ("hidden function", 109, vec![2]),
            ("undefined function", 110, 0u16.to_le_bytes().to_vec()),
            ("function section index", 110, 5u16.to_le_bytes().to_vec()),
            (
                "extended symbol section index",
                110,
                0xffffu16.to_le_bytes().to_vec(),
            ),
            ("function value", 112, 1u64.to_le_bytes().to_vec()),
            ("zero function size", 120, 0u64.to_le_bytes().to_vec()),
            ("function size", 120, u64::MAX.to_le_bytes().to_vec()),
            ("non-executable code", TEXT + 8, 2u64.to_le_bytes().to_vec()),
            (
                "compressed symbols",
                SYMBOLS + 8,
                SHF_COMPRESSED.to_le_bytes().to_vec(),
            ),
            (
                "compressed strings",
                STRINGS + 8,
                SHF_COMPRESSED.to_le_bytes().to_vec(),
            ),
            ("missing function", 66, vec![b'x']),
        ];
        for (name, offset, replacement) in cases {
            let mut bytes = object();
            bytes[offset..offset + replacement.len()].copy_from_slice(&replacement);
            let error = verify_elf64_rel_x86_64_bytes(&bytes).expect_err(name);
            assert_eq!(error.code, "E_OUTPUT", "{name}");
        }
        let mut bytes = object();
        put64(&mut bytes, 112, 1);
        put64(&mut bytes, 120, u64::MAX);
        assert!(verify_elf64_rel_x86_64_bytes(&bytes).is_err());

        let bytes = object();
        let duplicate = with_extra_symbol(&bytes[104..128]);
        assert!(
            verify_elf64_rel_x86_64_bytes(&duplicate)
                .unwrap_err()
                .detail
                .contains("one global")
        );

        let mut special = [0; 24];
        special[4] = 0x10;
        put16(&mut special, 6, 0xfff1); // A non-target SHN_ABS symbol is valid.
        verify_elf64_rel_x86_64_bytes(&with_extra_symbol(&special)).unwrap();
        put16(&mut special, 6, 0xffff);
        assert!(
            verify_elf64_rel_x86_64_bytes(&with_extra_symbol(&special))
                .unwrap_err()
                .detail
                .contains("extended symbol")
        );

        let mut bytes = object();
        bytes.resize(TABLE + 6 * 64, 0);
        bytes.extend_from_within(80..128);
        put16(&mut bytes, 60, 6);
        let second_table = TABLE + 5 * 64;
        bytes.copy_within(SYMBOLS..SYMBOLS + 64, second_table);
        put64(&mut bytes, second_table + 24, (TABLE + 6 * 64) as u64);
        assert!(
            verify_elf64_rel_x86_64_bytes(&bytes)
                .unwrap_err()
                .detail
                .contains("symbol table layout")
        );
    }

    #[test]
    fn rejects_link_requirements_outside_current_producer_shape() {
        for (kind, detail) in [
            (SHT_REL, "relocation sections"),
            (SHT_RELA, "relocation sections"),
            (SHT_DYNAMIC, "dynamic tables"),
            (SHT_DYNSYM, "dynamic tables"),
        ] {
            for size in [0, 24] {
                let mut bytes = object();
                bytes.resize(TABLE + 6 * 64 + 24, 0);
                put16(&mut bytes, 60, 6);
                let section = TABLE + 5 * 64;
                put32(&mut bytes, section + 4, kind);
                put64(&mut bytes, section + 24, (TABLE + 6 * 64) as u64);
                put64(&mut bytes, section + 32, size);
                let error = verify_elf64_rel_x86_64_bytes(&bytes).unwrap_err();
                assert_eq!(error.code, "E_OUTPUT");
                assert!(error.detail.starts_with("unsupported assembler output:"));
                assert!(error.detail.contains(detail));
            }
        }

        for binding in [0, 1, 2] {
            for target_index in [0, 0xfff2] {
                let mut symbol = [0; 24];
                symbol[4] = binding << 4;
                put16(&mut symbol, 6, target_index);
                let mut bytes = with_extra_symbol(&symbol);
                if binding == 0 {
                    // Keep local symbols before the existing global aic_main.
                    let global = bytes[104..128].to_vec();
                    bytes.copy_within(128..152, 104);
                    bytes[128..152].copy_from_slice(&global);
                    put32(&mut bytes, SYMBOLS + 44, 2);
                }
                let error = verify_elf64_rel_x86_64_bytes(&bytes).unwrap_err();
                assert_eq!(error.code, "E_OUTPUT");
                assert!(error.detail.starts_with("unsupported assembler output:"));
                assert!(error.detail.contains(if target_index == 0 {
                    "undefined symbols"
                } else {
                    "common symbols"
                }));
            }
        }
        let mut symbol = [0; 24];
        symbol[4] = 0x15; // STT_COMMON is also unsupported with an ordinary index.
        put16(&mut symbol, 6, 1);
        assert!(
            verify_elf64_rel_x86_64_bytes(&with_extra_symbol(&symbol))
                .unwrap_err()
                .detail
                .contains("common symbols")
        );
    }

    #[test]
    fn rejects_every_truncated_prefix_without_panicking() {
        let bytes = object();
        for end in 0..bytes.len() {
            assert!(verify_elf64_rel_x86_64_bytes(&bytes[..end]).is_err());
        }
    }
}
