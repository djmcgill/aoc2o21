use nom::{character::complete::{alpha1, char, i32}, multi::separated_list1};

#[derive(Debug, Clone, Copy)]
enum Command {
    Forward(i32),
    Down(i32),
    Up(i32),
}
impl Command {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, command) = alpha1(input)?;
        let (input, _) = char(' ')(input)?;
        let (input, distance) = i32(input)?;
        let command = match command {
            "forward" => Command::Forward(distance),
            "up" => Command::Up(distance),
            "down" => Command::Down(distance),
            _ => panic!()
        };
        Ok((input, command))
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = include_str!("input.txt");
    let (_, commands) = separated_list1(char('\n'), Command::parse)(input)?;
    let mut depth = 0;
    let mut pos = 0;
    let mut aim = 0;
    for command in commands {
        match command {
            Command::Forward(x) => {
                pos += x;
                depth += aim*x;
            }
            Command::Down(x) => aim += x,
            Command::Up(x) => aim -= x,
        }
    }
    println!("{}", depth*pos);
    Ok(())
}
