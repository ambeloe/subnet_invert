extern crate core;

use std::env;
use std::net::Ipv4Addr;
use std::process::ExitCode;
use std::str::FromStr;
use cidr::{Cidr, Ipv4Cidr};

fn main() -> ExitCode {
    let mut subnets: Vec<Ipv4Cidr> = Vec::new();
    for a in env::args().skip(1) {
        let ip = Ipv4Cidr::from_str(a.as_str());
        if ip.is_ok() {
            subnets.push(ip.unwrap());
        } else {
            println!("invalid subnet: {}", a);
            println!("provide one or more subnets separated by a space, and the minimum \"inverse\" will be output (subnet_invert 64.0.0.0/3 ==> 0.0.0.0/2 96.0.0.0/3 128.0.0.0/1)");
            return  ExitCode::FAILURE
        }
    }
    
    let mut supernet: Vec<Ipv4Cidr> = Vec::new();
    for s in &subnets {
        let mut inv: Vec<Ipv4Cidr> = Vec::new();
        
        invert_subnet(
            *s,
            Ipv4Cidr::new(
                Ipv4Addr::from_bits(0), 
                0
            ).unwrap(),
            &mut inv
        );
        
        supernet = merge_subnets(supernet, inv);
    }
    
    supernet.sort_by_key(|c| c.network_length());

    // let mut blocked: u128 = 0;
    // let mut allowed: u128 = 0;
    // for s in &subnets{
    //     allowed += 2u128.pow((32 - s.network_length()) as u32);
    // }
    // for s in &supernet {
    //     blocked += 2u128.pow((32 - s.network_length()) as u32);
    // }
    // println!("input {}, output : {}, total: {}", allowed, blocked, allowed+blocked);
    
    for s in supernet {
        print!("{} ", s);
    }

    ExitCode::SUCCESS
}

/// target: subnet you want to find the inverse of
/// pos: where to start the search from (probably want this to be 0.0.0.0/0 if calling it yourself)
fn invert_subnet(target :Ipv4Cidr, pos:Ipv4Cidr, output: &mut Vec<Ipv4Cidr>){
    //smallest subnet possible
    if target == pos || pos.first_address() == pos.last_address(){
        return
    }
    
    //split bigger subnet into two halves
    let front = Ipv4Cidr::new(pos.first_address(), pos.network_length()+1).unwrap();
    let back = Ipv4Cidr::new(
        Ipv4Addr::from_bits(
            pos.first_address().to_bits()+
                (((pos.last_address().to_bits()-pos.first_address().to_bits())/2)+1),
        ),
        pos.network_length()+1
    ).unwrap();
    
    let fc = front.contains(&target.first_address()) && front.contains(&target.last_address());
    let bc = back.contains(&target.first_address()) && back.contains(&target.last_address());
    
    if fc && !bc { //in front
        output.push(back);
        invert_subnet(target, front, output);
    } else if bc && !fc { //in back
        output.push(front);
        invert_subnet(target, back, output);
    } else if fc && bc {
        return
    } else {
        output.push(pos)
    }
}

fn merge_subnets(a: Vec<Ipv4Cidr>, b: Vec<Ipv4Cidr>) -> Vec<Ipv4Cidr> {
    let mut e: Vec<Ipv4Cidr> = Vec::new();
    
    e.extend(a);
    e.extend(b);
    
    e
}
