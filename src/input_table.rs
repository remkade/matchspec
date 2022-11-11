// Using a table-based parser provides better performance than the
// is_alpha/is_digit/etc functions in nom. This is due to the reduced
// number of comparisons that are performed for each character.

// nom's functions work by checking if a character is within a
// specified range (or set of ranges), which requires multiple
// comparisons. The table-based lookup works by checking if the
// character is in one of the classes defined below, which
// requires only a single comparison.

const IC_NM: u16 = 0;    // No match
const IC_AN: u16 = 1;    // Alphanumeric; a-z,A-Z,0-9
const IC_DU: u16 = 2;    // Dash/Underscore; -, _
const IC_GL: u16 = 4;    // Glob: *
const IC_PE: u16 = 8;    // Period; .
const IC_CO: u16 = 16;   // Comma; ,
const IC_FS: u16 = 32;   // Forward slash; /
const IC_QU: u16 = 64;   // Quotes; ' "
const IC_CL: u16 = 128;  // Colon; :
const IC_BA: u16 = 256;  // Bar; |
const IC_LB: u16 = 512;  // Left bracket; [
const IC_RB: u16 = 1024; // Right bracket; ]

const INPUT_CLASS_BITMASK: u8 = 0x7F; // Mask out the high bit, since
                                      // our table only has 128 entries.

static INPUT_CLASS_TABLE:&'static [u16] = &[
    IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM,
    IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM,
    IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM,
    IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM,
    IC_NM, IC_NM, IC_QU, IC_NM, IC_NM, IC_NM, IC_NM, IC_QU,
    IC_NM, IC_NM, IC_GL, IC_NM, IC_CO, IC_DU, IC_PE, IC_FS,
    IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN,
    IC_AN, IC_AN, IC_CL, IC_NM, IC_NM, IC_NM, IC_NM, IC_NM,
    IC_NM, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN,
    IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN,
    IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN,
    IC_AN, IC_AN, IC_AN, IC_LB, IC_NM, IC_RB, IC_NM, IC_DU,
    IC_NM, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN,
    IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN,
    IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN, IC_AN,
    IC_AN, IC_AN, IC_AN, IC_NM, IC_BA, IC_NM, IC_NM, IC_NM,
];

pub fn filter_char(c: char, cl: u16) -> bool {
    // Only the low byte of the 32bit char is needed -- these functions
    // are only concerned with ASCII characters. The high bit is also
    // ignored with the INPUT_CLASS_BITMASK mask. If the high bit was set,
    // then the character would not be an ASCII character.
    // Ignoring this bit allows the table size to be reduced by 50%.
    (INPUT_CLASS_TABLE[((c as u8) & INPUT_CLASS_BITMASK) as usize] & cl) != 0
}

pub fn is_alphanumeric_with_dashes(c: char) -> bool {
    filter_char(c, IC_AN | IC_DU)
}

pub fn is_alphanumeric_with_dashes_or_period(c: char) -> bool {
    filter_char(c, IC_AN | IC_DU | IC_PE)
}

pub fn is_any_valid_str_with_glob(c: char) -> bool {
    filter_char(c, IC_AN | IC_DU | IC_PE | IC_GL)
}

pub fn is_quote(c: char) -> bool {
    filter_char(c, IC_QU)
}

pub fn is_colon(c: char) -> bool {
    filter_char(c, IC_CL)
}

pub fn is_comma(c: char) -> bool {
    filter_char(c, IC_CO)
}

pub fn is_comma_or_alt(c: char) -> bool {
    filter_char(c, IC_CO | IC_BA)
}

pub fn is_forward_slash(c: char) -> bool {
    filter_char(c, IC_FS)
}

pub fn is_left_bracket(c: char) -> bool {
    filter_char(c, IC_LB)
}

pub fn is_right_bracket(c: char) -> bool {
    filter_char(c, IC_RB)
}
