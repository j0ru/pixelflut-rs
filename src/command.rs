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

#[derive(Debug)]
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