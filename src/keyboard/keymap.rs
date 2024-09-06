pub static TO_ASCII: [Option<char>; 256] = {
    let mut table = [None; 256];

    table[0x02] = Some('1');
    table[0x03] = Some('2');
    table[0x04] = Some('3');
    table[0x05] = Some('4');
    table[0x06] = Some('5');
    table[0x07] = Some('6');
    table[0x08] = Some('7');
    table[0x09] = Some('8');
    table[0x0A] = Some('9');
    table[0x0B] = Some('0');
    table[0x1E] = Some('a');
    table[0x30] = Some('b');
    table[0x2E] = Some('c');
    table[0x20] = Some('d');
    table[0x12] = Some('e');
    table[0x21] = Some('f');
    table[0x22] = Some('g');
    table[0x23] = Some('h');
    table[0x17] = Some('i');
    table[0x24] = Some('j');
    table[0x25] = Some('k');
    table[0x26] = Some('l');
    table[0x32] = Some('m');
    table[0x31] = Some('n');
    table[0x18] = Some('o');
    table[0x19] = Some('p');
    table[0x10] = Some('q');
    table[0x13] = Some('r');
    table[0x1F] = Some('s');
    table[0x14] = Some('t');
    table[0x16] = Some('u');
    table[0x2F] = Some('v');
    table[0x11] = Some('w');
    table[0x2D] = Some('x');
    table[0x15] = Some('y');
    table[0x2C] = Some('z');
    table[0x39] = Some(' ');

    table[0x0C] = Some('-');
    table[0x0D] = Some('=');
    table[0x1A] = Some('[');
    table[0x1B] = Some(']');
    table[0x2B] = Some('\\');
    table[0x27] = Some(';');
    table[0x28] = Some('\'');
    table[0x33] = Some(',');
    table[0x34] = Some('.');
    table[0x35] = Some('/');

    table
};

pub static TO_SHIFT_ASCII: [Option<char>; 256] = {
    let mut table = [None; 256];

    table[0x02] = Some('!');
    table[0x03] = Some('@');
    table[0x04] = Some('#');
    table[0x05] = Some('$');
    table[0x06] = Some('%');
    table[0x07] = Some('^');
    table[0x08] = Some('&');
    table[0x09] = Some('*');
    table[0x0A] = Some('(');
    table[0x0B] = Some(')');
    table[0x1E] = Some('A');
    table[0x30] = Some('B');
    table[0x2E] = Some('C');
    table[0x20] = Some('D');
    table[0x12] = Some('E');
    table[0x21] = Some('F');
    table[0x22] = Some('G');
    table[0x23] = Some('H');
    table[0x17] = Some('I');
    table[0x24] = Some('J');
    table[0x25] = Some('K');
    table[0x26] = Some('L');
    table[0x32] = Some('M');
    table[0x31] = Some('N');
    table[0x18] = Some('O');
    table[0x19] = Some('P');
    table[0x10] = Some('Q');
    table[0x13] = Some('R');
    table[0x1F] = Some('S');
    table[0x14] = Some('T');
    table[0x16] = Some('U');
    table[0x2F] = Some('V');
    table[0x11] = Some('W');
    table[0x2D] = Some('X');
    table[0x15] = Some('Y');
    table[0x2C] = Some('Z');
    table[0x39] = Some(' ');

    table[0x0C] = Some('_');
    table[0x0D] = Some('+');
    table[0x1A] = Some('{');
    table[0x1B] = Some('}');
    table[0x2B] = Some('|');
    table[0x27] = Some(':');
    table[0x28] = Some('"');
    table[0x33] = Some('<');
    table[0x34] = Some('>');
    table[0x35] = Some('?');

    table
};
