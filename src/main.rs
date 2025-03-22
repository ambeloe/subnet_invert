extern crate core;

use cidr::{Cidr, Ipv4Cidr};
use std::env;
use std::net::Ipv4Addr;
use std::process::ExitCode;
use std::str::FromStr;

fn main() -> ExitCode {
    let mut subnets: Vec<Ipv4Cidr> = Vec::new();
    for a in env::args().skip(1) {
        let ip = Ipv4Cidr::from_str(a.as_str());
        if ip.is_ok() {
            subnets.push(ip.unwrap());
        } else {
            println!("invalid subnet: {}", a);
            println!(
                "provide one or more subnets separated by a space, and the minimum \"inverse\" will be output (subnet_invert 64.0.0.0/3 ==> 0.0.0.0/2 96.0.0.0/3 128.0.0.0/1)"
            );
            return ExitCode::FAILURE;
        }
    }

    let mut supernet: Vec<Ipv4Cidr> = Vec::new();
    invert_subnet(
        &subnets,
        Ipv4Cidr::new(Ipv4Addr::from_bits(0), 0).unwrap(),
        &mut supernet,
    );

    supernet.sort_by_key(|c| c.network_length());

    let mut blocked: u128 = 0;
    let mut allowed: u128 = 0;
    for s in &subnets {
        allowed += 2u128.pow((32 - s.network_length()) as u32);
    }
    for s in &supernet {
        blocked += 2u128.pow((32 - s.network_length()) as u32);
    }
    println!(
        "input {}, output : {}, total: {}",
        allowed,
        blocked,
        allowed + blocked
    );

    for s in supernet {
        print!("{} ", s);
    }

    ExitCode::SUCCESS
}

/// target: subnet you want to find the inverse of
/// pos: where to start the search from (probably want this to be 0.0.0.0/0 if calling it yourself)
fn invert_subnet(target: &Vec<Ipv4Cidr>, pos: Ipv4Cidr, output: &mut Vec<Ipv4Cidr>) {
    //smallest subnet possible
    if pos.first_address() == pos.last_address() {
        return;
    }

    //split bigger subnet into two halves
    let front = Ipv4Cidr::new(pos.first_address(), pos.network_length() + 1).unwrap();
    let back = Ipv4Cidr::new(
        Ipv4Addr::from_bits(
            pos.first_address().to_bits()
                + (((pos.last_address().to_bits() - pos.first_address().to_bits()) / 2) + 1),
        ),
        pos.network_length() + 1,
    )
    .unwrap();

    let mut fc: bool = false;
    let mut bc: bool = false;
    for i in target {
        if *i == pos {
            continue;
        }
        fc |= front.contains(&i.first_address()) && front.contains(&i.last_address());
        bc |= back.contains(&i.first_address()) && back.contains(&i.last_address());
    }

    if fc && !bc {
        output.push(back);
        invert_subnet(target, front, output);
    } else if bc && !fc {
        output.push(front);
        invert_subnet(target, back, output);
    } else if fc && bc {
        invert_subnet(target, front, output);
        invert_subnet(target, back, output);
    } else {
        return
    }
}
