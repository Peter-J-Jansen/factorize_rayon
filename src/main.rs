//  SPDX-FileCopyrightText: Petrus J. F. M. Jansen
//  SPDX-License-Identifier: CC0-1.0
//! Rayon **find_map_any()** demonstration program
//! 
//! This small program shows how effective and simple Rust's *Rayon* crate **find_map_any()** can
//! be used to utilize all CPU cores. 
//! 
//! However, it also shows that **find_map_any()** still has a bug in that it can get in a loop 
//! such that a thread closure correctly ending with a *not-None* option can at times *never end*.
//! This behavior has been observed to occur not always.  In certain cases it almost never occurs
//! (e.g for `test[1]`), in other cases sometimes (like for `test[2]`), and in even other cases
//! it happens every time (see `test[2]`).
//! 
//! Hopefully the Rayon support team can fix this bug.
//! 
//! P.S.: The `factorize_iterative()` function of mine was an experiment to try out *find_map_any()*
//! with something not immediately trivial, but is of course not meant for production use.  Better and 
//! fast algorithms exist, also in crates.io.  
use std::time::Instant ;
use rayon::prelude::*;
type Un = u128 ;
fn main() {
    struct Test {
        n: u128 ,
        p_0: u128 ,
        q_0: u128 ,
    } 

    let tests = vec!( 
       Test {n:  1_000_003_967 *  1_000_004_249, p_0: 153, q_0: 127} ,
       Test {n: 10_000_004_659 * 10_000_004_873, p_0:  51, q_0:   9} ,
       Test {n: 20_000_004_317 * 20_000_004_487, p_0: 221, q_0: 135} ,   
    ) ;

    for test in &tests {
        let p_q = possible_factor_lsb_pairs((&test).n) ;

        let mut now = Instant::now() ;
        let mut factors = factorize_iterative((&test).n, 1, 1).unwrap() ;
        println!("factorize_iterative( {0}, ...) yields {1} * {2}; took {3} msec, using a single thread." ,
            (&test).n, factors.0, factors.1, now.elapsed().as_millis()) ;

        now = Instant::now() ;
        factors = factorize_iterative((&test).n, (&test).p_0, (&test).q_0).unwrap() ;
        println!("factorize_iterative( {0}, ...) yields {1} * {2}; took {3} msec, using a single thread when starting with the correct p q LSB pair." ,
        (&test).n, factors.0, factors.1, now.elapsed().as_millis()) ;

        now = Instant::now() ;
        factors = (0..p_q.len())
        .into_par_iter()
        .find_map_any(|p_q_index| {
            factorize_iterative((&test).n, (p_q[p_q_index]).0.into(), (p_q[p_q_index]).1.into())
        })
        .unwrap() ; 
        println!("factorize_iterative( {0}, ...) yields {1} * {2}; took {3} msec, using a Rayon thread per core." ,
        (&test).n, factors.0, factors.1, now.elapsed().as_millis()) ;
    }       
}

fn possible_factor_lsb_pairs(n: Un) -> Vec<(u8, u8)> {
    let mut result: Vec<(u8, u8)> = Vec::with_capacity(64) ;
    for i in 0..=u8::MAX {
        for j in 0..=i {
//          if (i * j) % 256 = n % 256 { 
            if i.wrapping_mul(j) == n as u8 {
            result.push((i, j)) ;
            }
        }
    }
    result
}

#[allow(unused_variables)]
fn factorize_iterative(n: Un, p_0: Un, q_0: Un) -> Option<(Un, Un)> {
    let n_msb = size_of_val(&n) * 8 - n.leading_zeros() as usize ;
    let mut try_count: [u8; 128] = [0; 128] ;
    let try_pq_1x_first: Vec<bool> = (0..128).map(|x| x % 2 == 1).collect() ;
    let mut pq: [Un; 128] = [1; 128] ;
    let mut p:  [Un; 128] = [1; 128] ;
    let mut q:  [Un; 128] = [1; 128] ;
    let mut k: usize = 1 ;
    let mut i: u128 = 0 ;
    if p_0 > 1 || q_0 > 1 {
        p[7] = p_0 ;
        q[7] = q_0 ;
        pq[7] = p_0 * q_0 ;
        k = 8 ;
    }
    while (n != pq[k - 1]) || (p[k - 1] == 1) || (q[k - 1] == 1) {
        if n < pq[k - 1] || k > n_msb {
            try_count[k] = 0 ;
            k -= 1 ;
            while try_count[k] > 1  {
                try_count[k] = 0 ;
                k -= 1 ;
                if k == 7 && p_0 > 1 && q_0 > 1 {
//                  println!("factorize_iterative starting from ({},{}) unsuccessful", p_0, q_0) ; 
                    return None ;
                }
            }
        } else {
            if (n & (1 << k)) == (pq[k - 1] & (1 << k)) {
                if try_pq_1x_first[k] == (try_count[k] == 0) {
                    if k > 64 {
                        pq[k] = Un::MAX ;
                    } else {
                        pq[k] = pq[k - 1] + ((p[k - 1] + q[k - 1] + (1 << k)) << k) ;
                    }
                    p[k] = p[k - 1] + (1 << k) ;
                    q[k] = q[k - 1] + (1 << k) ;
                } else {
                    pq[k] = pq[k - 1] ;
                    p[k] = p[k - 1] ;
                    q[k] = q[k - 1] ;
                }
            } else {
                if try_pq_1x_first[k] == (try_count[k] == 0) {
                    pq[k] = pq[k - 1] + (q[k - 1] << k) ;
                    p[k] = p[k - 1] + (1 << k) ;
                    q[k] = q[k - 1] ;
                } else {
                    pq[k] = pq[k - 1] + (p[k - 1] << k) ;
                    p[k] = p[k - 1] ;
                    q[k] = q[k - 1] + (1 << k) ;
                }
            }
            try_count[k] += 1 ;
            k += 1 ;
        }
        i += 1 ;
    }

//  print_npq("    done", n, pq[k - 1], p[k - 1], q[k - 1], k, try_count[k], i) ;

    assert!(n == p[k - 1] * q[k - 1]) ;
    Some((p[k - 1], q[k - 1]))
}

#[allow(dead_code)]
fn print_npq(pfx: &str, n: Un, pq: Un, p:Un, q: Un, k: usize, t: u8, i: u128) {
    let n_msb = size_of_val(&n) * 8 - n.leading_zeros() as usize ;
    println!("{}: n={: >80b} bit_count( n)={}", pfx,  n, (size_of_val( &n) * 8) -  n.leading_zeros() as usize) ;
    println!("{}:pq={: >80b} bit_count(pq)={}", pfx, pq, (size_of_val(&pq) * 8) - pq.leading_zeros() as usize) ;
    println!("{}: k={: >80} n_msb={n_msb} try_count[{k}]={t} iterations={i}", pfx, "-".repeat(k)) ;
    println!("{}: p={: >80b} bit_count( p)={}", pfx,  p, (size_of_val( &p) * 8) -  p.leading_zeros() as usize) ;
    println!("{}: q={: >80b} bit_count( q)={}", pfx,  q, (size_of_val( &q) * 8) -  q.leading_zeros() as usize) ;
    println!("{}: k={:=>80}", pfx, "=") ;
}