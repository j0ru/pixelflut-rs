use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::{space1, line_ending},
    character::complete,
    combinator::opt,
    branch::alt,
    combinator::map_res,
    sequence::{tuple, preceded, terminated},
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
    let (input, (red, green, blue, alpha)) = tuple((hex_primary, hex_primary, hex_primary, opt(hex_primary)))(input)?;
    Ok((input, Color { red, green, blue, alpha: alpha.unwrap_or(255)}))
}

fn size(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag("SIZE")(input)?;
    Ok((input, Command::SIZE))
}

fn help(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag("HELP")(input)?;
    Ok((input, Command::HELP))
}

fn px(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag("PX")(input)?;
    let (input, (_, x, _, y, maybe_color)) = tuple((space1, complete::u32, space1, complete::u32, opt(preceded(space1, hex_color))))(input)?;
    Ok((input, Command::PX(x,y, maybe_color)))
}

#[derive(Debug, PartialEq)]
pub enum Command {
    SIZE,
    HELP,
    PX(u32, u32, Option<Color>),
    NONE
}

impl Command {
    pub fn parse(input: &str) -> Command {
        match terminated(alt((size, help, px)), line_ending)(input) {
            Ok((_, c)) => c,
            Err(_) => Command::NONE
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn size() {
        assert_eq!(Command::parse("SIZE\n"), Command::SIZE);
    }
    
    #[test]
    fn help() {
        assert_eq!(Command::parse("HELP\n"), Command::HELP);
    }

    #[test]
    fn no_newline() {
        assert_eq!(Command::parse("HELP"), Command::NONE);
    }

    #[test]
    fn px_get() {
        assert_eq!(Command::parse("PX 1 1\n"), Command::PX(1, 1, None));
        assert_eq!(Command::parse("PX 1a 1\n"), Command::NONE);
        assert_eq!(Command::parse("PX -1 1\n"), Command::NONE);
    }

    #[test]
    fn px_set() {
        assert_eq!(Command::parse("PX 1 1 ff00ff\n"), Command::PX(1, 1, Some(Color {red: 255, green: 0, blue: 255, alpha: 255})));
        assert_eq!(Command::parse("PX 1 1 aa00ffaa\n"), Command::PX(1, 1, Some(Color {red: 170, green: 0, blue: 255, alpha: 170})));
    }
}