#![feature(command_access)]

use procmd::{cmd, PipeCommand};
use std::process::Command;

fn assert_eq_commands(a: &Command, b: &Command) {
    assert_eq!(a.get_program(), b.get_program());
    assert!(a.get_args().eq(b.get_args()));
    assert!(a.get_envs().eq(b.get_envs()));
}

#[test]
fn simple() {
    let a = cmd!("ls", "-a", "-l");
    let mut b = Command::new("ls");
    b.args(&["-a", "-l"]);
    assert_eq_commands(&a, &b);
}

#[test]
fn piped() {
    let a = cmd!("ls" => "grep", "test" => "wc", "-l");
    let b = PipeCommand::new([cmd!("ls"), cmd!("grep", "test"), cmd!("wc", "-l")]);
    assert_eq_commands(&a.commands[0], &b.commands[0]);
    assert_eq_commands(&a.commands[1], &b.commands[1]);
    assert_eq_commands(&a.commands[2], &b.commands[2]);
}
