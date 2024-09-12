use anyhow::bail;
use combine::parser::choice::choice;
use combine::parser::combinator::recognize;
use combine::parser::repeat::many1;
use combine::{attempt, count_min_max, satisfy, skip_many1, token};
use combine::{eof, optional};
use combine::{Parser, Stream};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum PmuEventModifier {
    UserSpaceCounting,
    KernelCounting,
    HypervisorCounting,
    NonIdleCounting,
    GuestCounting,
    HostCounting,
    PreciseLevel,
    UseMaxPreciseLevel,
    ReadSampleValue,
    Pin,
    GroupWeak,
    GroupExclusive,
    BpfAggregation,
}

impl TryFrom<char> for PmuEventModifier {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'u' => Ok(PmuEventModifier::UserSpaceCounting),
            'k' => Ok(PmuEventModifier::KernelCounting),
            'h' => Ok(PmuEventModifier::HypervisorCounting),
            'I' => Ok(PmuEventModifier::NonIdleCounting),
            'G' => Ok(PmuEventModifier::GuestCounting),
            'H' => Ok(PmuEventModifier::HostCounting),
            'p' => Ok(PmuEventModifier::PreciseLevel),
            'P' => Ok(PmuEventModifier::UseMaxPreciseLevel),
            'S' => Ok(PmuEventModifier::ReadSampleValue),
            'D' => Ok(PmuEventModifier::Pin),
            'W' => Ok(PmuEventModifier::GroupWeak),
            'e' => Ok(PmuEventModifier::GroupExclusive),
            'b' => Ok(PmuEventModifier::BpfAggregation),
            _ => bail!("Invalid modifier: {}", value),
        }
    }
}

fn modifier<Input>() -> impl Parser<Input, Output = PmuEventModifier>
where
    Input: Stream<Token = char>,
{
    satisfy(|c| {
        c == 'u'
            || c == 'k'
            || c == 'h'
            || c == 'I'
            || c == 'G'
            || c == 'H'
            || c == 'p'
            || c == 'P'
            || c == 'S'
            || c == 'D'
            || c == 'W'
            || c == 'e'
            || c == 'b'
    })
    .map(|c| PmuEventModifier::try_from(c).unwrap())
    .message("Invalid modifier")
}

fn modifiers<Input>() -> impl Parser<Input, Output = Vec<PmuEventModifier>>
where
    Input: Stream<Token = char>,
{
    (token(':'), count_min_max(1, 16, modifier())).map(|(_, modifiers)| modifiers)
}

#[derive(Debug, PartialEq)]
pub(crate) struct SymbolicEvent {
    name: String,
    modifier: Vec<PmuEventModifier>,
}

fn parse_name<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
{
    (
        satisfy(|c: char| {
            c.is_alphanumeric() || c == '_' || c == '*' || c == '?' || c == '[' || c == ']'
        }),
        many1(satisfy(|c: char| {
            c.is_alphanumeric()
                || c == '_'
                || c == '*'
                || c == '?'
                || c == '['
                || c == ']'
                || c == '.'
                || c == '!'
                || c == '-'
        })),
    )
        .map(|(first, rest): (char, String)| {
            let mut name = String::new();
            name.push(first);
            name.push_str(&rest);
            name
        })
}

fn parse_symbolic_event<Input>() -> impl Parser<Input, Output = PmuEvent>
where
    Input: Stream<Token = char>,
{
    (parse_name(), optional(modifiers()), eof()).map(|(name, modifiers, _)| {
        PmuEvent::SymbolicEvent(SymbolicEvent {
            name: name,
            modifier: modifiers.unwrap_or_default(),
        })
    })
}

fn parse_tracepoint_event<Input>() -> impl Parser<Input, Output = PmuEvent>
where
    Input: Stream<Token = char>,
{
    (
        parse_name(),
        token(':'),
        parse_name(),
        optional(modifiers()),
        eof(),
    )
        .map(|(category, _, name, modifiers, _)| {
            PmuEvent::TracepointEvent(TracepointEvent {
                category: category,
                name: name,
                modifier: modifiers.unwrap_or_default(),
            })
        })
}

fn parse_raw_event<Input>() -> impl Parser<Input, Output = PmuEvent>
where
    Input: Stream<Token = char>,
{
    (
        token('r'),
        optional(attempt((token('0'), token('x')))),
        recognize::<String, _, _>(skip_many1(satisfy(|c: char| c.is_ascii_hexdigit()))),
        optional(modifiers()),
    )
        .map(|(_, _, value, modifiers)| {
            PmuEvent::RawEvent(RawEvent {
                value: u64::from_str_radix(&value, 16).unwrap(),
                modifier: modifiers.unwrap_or_default(),
            })
        })
}

fn parse_event<Input>() -> impl Parser<Input, Output = PmuEvent>
where
    Input: Stream<Token = char>,
{
    choice((
        attempt(parse_raw_event()),
        attempt(parse_tracepoint_event()),
        attempt(parse_symbolic_event()),
    ))
}

#[derive(Debug, PartialEq)]
pub(crate) struct TracepointEvent {
    category: String,
    name: String,
    modifier: Vec<PmuEventModifier>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct RawEvent {
    value: u64,
    modifier: Vec<PmuEventModifier>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum PmuEvent {
    SymbolicEvent(SymbolicEvent),
    TracepointEvent(TracepointEvent),
    GroupedEvent(Vec<PmuEvent>),
    RawEvent(RawEvent),
}

mod tests {
    use super::*;
    use combine::EasyParser as _;

    #[test]
    fn test_modifier() {
        assert_eq!(
            modifier().easy_parse("u").unwrap(),
            (PmuEventModifier::UserSpaceCounting, "")
        );
        assert_eq!(
            modifier().easy_parse("k").unwrap(),
            (PmuEventModifier::KernelCounting, "")
        );
        assert_eq!(
            modifier().easy_parse("h").unwrap(),
            (PmuEventModifier::HypervisorCounting, "")
        );
        assert_eq!(
            modifier().easy_parse("I").unwrap(),
            (PmuEventModifier::NonIdleCounting, "")
        );
        assert_eq!(
            modifier().easy_parse("G").unwrap(),
            (PmuEventModifier::GuestCounting, "")
        );
        assert_eq!(
            modifier().easy_parse("H").unwrap(),
            (PmuEventModifier::HostCounting, "")
        );
        assert_eq!(
            modifier().easy_parse("p").unwrap(),
            (PmuEventModifier::PreciseLevel, "")
        );
        assert_eq!(
            modifier().easy_parse("P").unwrap(),
            (PmuEventModifier::UseMaxPreciseLevel, "")
        );
        assert_eq!(
            modifier().easy_parse("S").unwrap(),
            (PmuEventModifier::ReadSampleValue, "")
        );
        assert_eq!(
            modifier().easy_parse("D").unwrap(),
            (PmuEventModifier::Pin, "")
        );
        assert_eq!(
            modifier().easy_parse("W").unwrap(),
            (PmuEventModifier::GroupWeak, "")
        );
        assert_eq!(
            modifier().easy_parse("e").unwrap(),
            (PmuEventModifier::GroupExclusive, "")
        );
        assert_eq!(
            modifier().easy_parse("b").unwrap(),
            (PmuEventModifier::BpfAggregation, "")
        );
        assert_eq!(modifier().easy_parse("x").is_err(), true);
    }

    #[test]
    fn test_modifiers() {
        let mut parser = (modifiers(), eof()).map(|(modifiers, _)| modifiers);

        assert_eq!(
            parser.easy_parse(":uppp").unwrap(),
            (
                vec![
                    PmuEventModifier::UserSpaceCounting,
                    PmuEventModifier::PreciseLevel,
                    PmuEventModifier::PreciseLevel,
                    PmuEventModifier::PreciseLevel
                ],
                ""
            )
        );
        let input = ":uvh";
        let result = parser.easy_parse(input);
        let result = result.map_err(|e| e.map_position(|p| p.translate_position(input)));
        let formatted_err = &format!("{}", result.unwrap_err());
        assert_eq!(
            "Parse error at 2
Unexpected `v`
Expected end of input
",
            formatted_err
        );
    }

    #[test]
    fn test_parse_symbolic_event() {
        let mut parser = (parse_symbolic_event(), eof()).map(|(event, _)| event);
        assert_eq!(
            parser.easy_parse("cpu-cycles:P").unwrap(),
            (
                PmuEvent::SymbolicEvent(SymbolicEvent {
                    name: "cpu-cycles".to_string(),
                    modifier: vec![PmuEventModifier::UseMaxPreciseLevel]
                }),
                ""
            )
        );
    }

    #[test]
    fn test_parse_tracepoint_event() {
        let mut parser = (parse_tracepoint_event(), eof()).map(|(event, _)| event);
        assert_eq!(
            parser.easy_parse("sched:sched_switch:P").unwrap(),
            (
                PmuEvent::TracepointEvent(TracepointEvent {
                    category: "sched".to_string(),
                    name: "sched_switch".to_string(),
                    modifier: vec![PmuEventModifier::UseMaxPreciseLevel]
                }),
                ""
            )
        );
    }

    #[test]
    fn test_raw_event() {
        let mut parser = (parse_raw_event(), eof()).map(|(event, _)| event);

        assert_eq!(
            parser.easy_parse("r0xdeadbeefcafe:u").unwrap(),
            (
                PmuEvent::RawEvent(RawEvent {
                    value: 0xdeadbeefcafe,
                    modifier: vec![PmuEventModifier::UserSpaceCounting]
                }),
                ""
            )
        );

        assert_eq!(
            parser.easy_parse("r0500").unwrap(),
            (
                PmuEvent::RawEvent(RawEvent {
                    value: 0x0500,
                    modifier: vec![]
                }),
                ""
            )
        );
    }

    #[test]
    fn test_event() {
        let mut parser = (parse_event(), eof()).map(|(event, _)| event);
        assert_eq!(
            parser.easy_parse("cpu-cycles:P").unwrap(),
            (
                PmuEvent::SymbolicEvent(SymbolicEvent {
                    name: "cpu-cycles".to_string(),
                    modifier: vec![PmuEventModifier::UseMaxPreciseLevel]
                }),
                ""
            )
        );

        assert_eq!(
            parser.easy_parse("sched:sched_switch:P").unwrap(),
            (
                PmuEvent::TracepointEvent(TracepointEvent {
                    category: "sched".to_string(),
                    name: "sched_switch".to_string(),
                    modifier: vec![PmuEventModifier::UseMaxPreciseLevel]
                }),
                ""
            )
        );

        assert_eq!(
            parser.easy_parse("r0xdeadbeefcafe:u").unwrap(),
            (
                PmuEvent::RawEvent(RawEvent {
                    value: 0xdeadbeefcafe,
                    modifier: vec![PmuEventModifier::UserSpaceCounting]
                }),
                ""
            )
        );
    }
}
