use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::ops::Not;
use std::str::Chars;

use anyhow::{ensure, Error, Result};
use rand::{thread_rng, Rng};

use crate::Context;

/// Roll dice
#[poise::command(slash_command)]
pub async fn roll(ctx: Context<'_>, #[description = "What to roll"] roll: String) -> Result<()> {
    ctx.say(eval(shunt(&roll)?)?).await?;

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Dice,
    KeepHighest,
    KeepLowest,
    Add,
    Sub,
    Mul,
    Div,
    LParen,
    RParen,
}

impl Operator {
    fn precedence(&self) -> u8 {
        match self {
            Operator::Dice => 4,
            Operator::KeepHighest | Operator::KeepLowest => 3,
            Operator::Mul | Operator::Div => 2,
            Operator::Add | Operator::Sub => 1,
            _ => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Roll {
    total: i32,
    rolls: Vec<i32>,
}

impl Roll {
    /// Roll m n-sided dice and create a Roll with the results
    fn mdn(m: i32, n: i32) -> Result<Self> {
        ensure!(m > 0, n > 0);

        let mut total: i32 = 0;
        let mut rolls = Vec::new();

        let mut rng = thread_rng();

        for _ in 0..m {
            let roll = rng.gen_range(1..=n);
            total += roll;
            rolls.push(roll);
        }

        Ok(Self { total, rolls })
    }

    fn keep_highest(&mut self, n: usize) -> Token {
        self.rolls
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
        while self.rolls.len() > n {
            if let Some(roll) = self.rolls.pop() {
                self.total -= roll;
            }
        }
        Token::Int(self.total)
    }

    fn keep_lowest(&mut self, n: usize) -> Token {
        self.rolls
            .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        while self.rolls.len() > n {
            if let Some(roll) = self.rolls.pop() {
                self.total -= roll;
            }
        }
        Token::Int(self.total)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Int(i32),
    Float(f32),
    Op(Operator),
    DiceRoll(Roll),
}

impl Token {
    fn precedence(&self) -> u8 {
        match self {
            Token::Op(op) => op.precedence(),
            _ => 0,
        }
    }

    fn int(self) -> Result<i32> {
        match self {
            Self::Int(n) => Ok(n),
            Self::DiceRoll(roll) => Ok(roll.total),
            _ => Err(Error::msg(format!("{:?}: not an integer", self))),
        }
    }

    fn roll(self) -> Result<Roll> {
        match self {
            Self::DiceRoll(roll) => Ok(roll),
            _ => Err(Error::msg(format!("{:?}: not a roll", self))),
        }
    }

    fn add(self, rhs: Token) -> Result<Token> {
        match self {
            Token::Int(_) | Token::DiceRoll(_) => match rhs {
                Token::Int(_) | Token::DiceRoll(_) => Ok(Token::Int(self.int()? + rhs.int()?)),
                Token::Float(rhs) => Ok(Token::Float(self.int()? as f32 + rhs)),
                _ => Err(Error::msg(format!("{:?}: not an operand", self))),
            },
            Token::Float(lhs) => match rhs {
                Token::Int(_) | Token::DiceRoll(_) => Ok(Token::Float(lhs + rhs.int()? as f32)),
                Token::Float(rhs) => Ok(Token::Float(lhs + rhs)),
                _ => Err(Error::msg(format!("{:?}: not an operand", self))),
            },
            _ => Err(Error::msg(format!("{:?}: not an operand", self))),
        }
    }

    fn sub(self, rhs: Token) -> Result<Token> {
        match self {
            Token::Int(_) | Token::DiceRoll(_) => match rhs {
                Token::Int(_) | Token::DiceRoll(_) => Ok(Token::Int(self.int()? - rhs.int()?)),
                Token::Float(rhs) => Ok(Token::Float(self.int()? as f32 - rhs)),
                _ => Err(Error::msg(format!("{:?}: not an operand", self))),
            },
            Token::Float(lhs) => match rhs {
                Token::Int(_) | Token::DiceRoll(_) => Ok(Token::Float(lhs - rhs.int()? as f32)),
                Token::Float(rhs) => Ok(Token::Float(lhs - rhs)),
                _ => Err(Error::msg(format!("{:?}: not an operand", self))),
            },
            _ => Err(Error::msg(format!("{:?}: not an operand", self))),
        }
    }

    fn mul(self, rhs: Token) -> Result<Token> {
        match self {
            Token::Int(_) | Token::DiceRoll(_) => match rhs {
                Token::Int(_) | Token::DiceRoll(_) => Ok(Token::Int(self.int()? * rhs.int()?)),
                Token::Float(rhs) => Ok(Token::Float(self.int()? as f32 * rhs)),
                _ => Err(Error::msg(format!("{:?}: not an operand", self))),
            },
            Token::Float(lhs) => match rhs {
                Token::Int(_) | Token::DiceRoll(_) => Ok(Token::Float(lhs * rhs.int()? as f32)),
                Token::Float(rhs) => Ok(Token::Float(lhs * rhs)),
                _ => Err(Error::msg(format!("{:?}: not an operand", self))),
            },
            _ => Err(Error::msg(format!("{:?}: not an operand", self))),
        }
    }

    fn div(self, rhs: Token) -> Result<Token> {
        match self {
            Token::Int(_) | Token::DiceRoll(_) => match rhs {
                Token::Int(_) | Token::DiceRoll(_) => {
                    Ok(Token::Float(self.int()? as f32 / rhs.int()? as f32))
                }
                Token::Float(rhs) => Ok(Token::Float(self.int()? as f32 / rhs)),
                _ => Err(Error::msg(format!("{:?}: not an operand", self))),
            },
            Token::Float(lhs) => match rhs {
                Token::Int(_) | Token::DiceRoll(_) => Ok(Token::Float(lhs / rhs.int()? as f32)),
                Token::Float(rhs) => Ok(Token::Float(lhs / rhs)),
                _ => Err(Error::msg(format!("{:?}: not an operand", self))),
            },
            _ => Err(Error::msg(format!("{:?}: not an operand", self))),
        }
    }
}

impl TryFrom<char> for Token {
    type Error = Error;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            'd' => Ok(Token::Op(Operator::Dice)),
            '*' => Ok(Token::Op(Operator::Mul)),
            '/' => Ok(Token::Op(Operator::Div)),
            '+' => Ok(Token::Op(Operator::Add)),
            '-' => Ok(Token::Op(Operator::Sub)),
            '(' => Ok(Token::Op(Operator::LParen)),
            ')' => Ok(Token::Op(Operator::RParen)),
            _ => Err(Error::msg(format!("unknown token: {value}"))),
        }
    }
}

impl TryFrom<&str> for Token {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "kh" => Ok(Token::Op(Operator::KeepHighest)),
            "kl" => Ok(Token::Op(Operator::KeepLowest)),
            _ => Err(Error::msg(format!("unknown token: {value}"))),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{n}"),
            Token::Float(n) => write!(f, "{n}"),
            Token::DiceRoll(n) => write!(f, "{}", n.total),
            _ => Err(std::fmt::Error),
        }
    }
}

// this dijkstra guy was a real one
pub fn shunt(input: &str) -> Result<Vec<Token>> {
    let mut out: Vec<Token> = Vec::new();
    let mut ops: Vec<Token> = Vec::new();

    let mut iter = input.chars().peekable();
    while let Some(token) = iter.next() {
        match token {
            '0'..='9' => out.push(num(token, &mut iter)?),
            'd' | 'k' | '*' | '/' | '+' | '-' => {
                let token: Token = if token == 'k' {
                    let mut keep = token.to_string();
                    keep.push(
                        iter.next()
                            .ok_or_else(|| Error::msg("k must be followed by h or l"))?,
                    );
                    keep.as_str().try_into()?
                } else {
                    token.try_into()?
                };
                while ops.last().is_some_and(|token_2| {
                    token_2 != &Token::Op(Operator::LParen)
                        && token_2.precedence() >= token.precedence()
                }) {
                    let op2 = ops.pop().unwrap();
                    out.push(op2);
                }
                ops.push(token);
            }
            '(' => ops.push(token.try_into()?),
            ')' => {
                while ops
                    .last()
                    .is_some_and(|op| op != &Token::Op(Operator::LParen))
                {
                    out.push(ops.pop().unwrap());
                }
                if ops
                    .last()
                    .is_some_and(|op| op == &Token::Op(Operator::LParen))
                {
                    ops.pop();
                }
            }
            ' ' => continue,
            _ => return Err(Error::msg("invalid expression")),
        }
    }

    while let Some(op) = ops.pop() {
        out.push(op);
    }

    Ok(out)
}

fn num(first: char, iter: &mut Peekable<Chars>) -> Result<Token> {
    let mut saw_dot: bool = false;
    let mut num = first.to_string();

    while iter
        .peek()
        .is_some_and(|char| char.is_ascii_digit() || saw_dot.not() && *char == '.')
    {
        let char = iter.next().unwrap();
        if saw_dot.not() && char == '.' {
            saw_dot = true;
        }
        num.push(char);
    }

    match saw_dot {
        true => Ok(Token::Float(num.parse()?)),
        false => Ok(Token::Int(num.parse()?)),
    }
}

pub fn eval(input: Vec<Token>) -> Result<String> {
    let mut stack: Vec<Token> = Vec::new();

    for token in input {
        match token {
            Token::Op(operator) => {
                let (rhs, mut lhs) = (stack.pop(), stack.pop());
                if lhs.is_none() && rhs.as_ref().is_some_and(|_| operator == Operator::Dice) {
                    lhs = Some(Token::Int(1));
                } else if lhs.is_none() && rhs.is_none() {
                    return Err(Error::msg(format!(
                        "not enough operands for operator: {:?}",
                        operator
                    )));
                }
                let (lhs, rhs) = (lhs.unwrap(), rhs.unwrap());
                stack.push(match operator {
                    Operator::Dice => Token::DiceRoll(Roll::mdn(lhs.int()?, rhs.int()?)?),
                    Operator::KeepHighest => lhs.roll()?.keep_highest(rhs.int()?.try_into()?),
                    Operator::KeepLowest => lhs.roll()?.keep_lowest(rhs.int()?.try_into()?),
                    Operator::Add => lhs.add(rhs)?,
                    Operator::Sub => lhs.sub(rhs)?,
                    Operator::Mul => lhs.mul(rhs)?,
                    Operator::Div => lhs.div(rhs)?,
                    _ => return Err(Error::msg(format!("{:?}: not an operator", operator))),
                });
            }
            _ => stack.push(token),
        }
    }

    ensure!(stack.len() == 1);
    Ok(stack[0].to_string())
}
