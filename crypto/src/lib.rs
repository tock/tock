#![feature(asm,concat_idents,const_fn)]
#![no_std]

extern crate kernel;



extern {
    fn multiply256x256_asm(ar:&mut[u32;16],br:&mut[u32;8],cr:&mut[u32;8]) ;
    fn square256_asm(ar:&mut[u32;16],br:&mut[u32;8]) ;
    fn fe25519_reduceTo256Bits_asm(ar:&mut[u32;8],br:&mut[u32;16]) ;
    fn fe25519_mpyWith121666_asm(br:&mut[u32;8],cr:&mut[u32;8]) ;
}




fn fe25519_cpy (dest:&mut[u32;8],source:&[u32;8]) {

  for x in 0..8 {
    dest[x]=source[x];
  }

}


//unpacks a 32 8bit array into the "standard" 8 32bit array

fn fe25519_unpack (dest:&mut[u32;8],source:&[u8;32]) {

  for x in 0..8 {

    dest[x]=((source[4*x+3]as u32)<<24)+((source[4*x+2]as u32)<<16)+((source[4*x+1]as u32)<<8)+(source[4*x+0]as u32);
  }

}


fn fe25519_sub (out:&mut[u32;8],basevalue:&[u32;8],valuetosubstract:&[u32;8]) {

  let mut accu:i64;
  accu = basevalue[7] as i64;
  accu = accu - (valuetosubstract[7] as i64);

  out[7]=(accu as u32)| 2147483648  ;

  accu = (19 * (((accu >> 31) as i32) - 1)) as i64;

  for ctr in 0..7 {
    accu += basevalue[ctr] as i64;
    accu -=valuetosubstract[ctr] as i64;
    
    out[ctr] = accu as u32;
    accu >>=32;
  }
  accu+=out[7] as i64;
  out[7]  = accu as u32;

}

//////////////////////////////////////////////






fn fe25519_sub_test (out:&mut[u32;8],basevalue:&[u32;8],valuetosubstract:&[u32;8]) {

  let mut accu:i64;
  accu = basevalue[7] as i64;
  accu = accu - (valuetosubstract[7] as i64);

  out[7]=(accu as u32)| 2147483648 ;

  accu = (19 * (((accu >> 31) as i32) - 1)) as i64;

  for ctr in 0..7 {
    accu += basevalue[ctr] as i64;
    accu -=valuetosubstract[ctr] as i64;
    
    out[ctr] = accu as u32;
    accu >>=32;
  }
  accu+=out[7] as i64;
  out[7]  = accu as u32;
  

}





//////////////////////////////////////

fn fe25519_add (out:&mut[u32;8],basevalue:&[u32;8],valuetoadd:&[u32;8]) {

  let mut accu:u64;
  accu = basevalue[7] as u64;
  accu += valuetoadd[7] as u64;

  out[7]=(accu as u32)& 2147483647  ;

  accu = 19 * (((accu >> 31) as u32)) as u64;

  for ctr in 0..7 {
    accu += basevalue[ctr] as u64;
    accu +=valuetoadd[ctr] as u64;
    
    out[ctr] = accu as u32;
    accu >>=32;
  }
  accu+=out[7] as u64;
  out[7]  = accu as u32;

}


fn fe25519_addab (mut basevalue:&mut[u32;8],valuetoadd:&mut[u32;8]) {

  let mut accu:u64;
  let mut out:[u32;8]=[0;8];
  accu = basevalue[7] as u64;
  accu += valuetoadd[7] as u64;

  out[7]=(accu as u32)& 2147483647  ;

  accu = 19 * (((accu >> 31) as u32)) as u64;

  for ctr in 0..7 {
    accu += basevalue[ctr] as u64;
    accu +=valuetoadd[ctr] as u64;
    
    out[ctr] = accu as u32;
    accu >>=32;
  }
  accu+=out[7] as u64;
  out[7]  = accu as u32;
  for ctr in 0..8 {
    
    basevalue[ctr]=out[ctr];
  }

}


fn fe25519_mul (mut out:&mut[u32;8],in1:&mut[u32;8],in2:&mut[u32;8]) {

  let mut temp: [u32;16]=[0;16];
  unsafe { multiply256x256_asm(&mut temp, in1, in2);
  }

  unsafe {
  fe25519_reduceTo256Bits_asm( &mut out,  &mut temp);

  
  }

}

fn fe25519_mul_test (mut out:&mut[u32;8],in1:&mut[u32;8],in2:&mut[u32;8]) {

  let mut temp: [u32;16]=[0;16];
  unsafe { multiply256x256_asm(&mut temp, in1, in2);
  }

  unsafe {
  fe25519_reduceTo256Bits_asm( &mut out,  &mut temp);

  
  }

}


fn fe25519_mulac(in1:&mut[u32;8],mut in2:&mut[u32;8]) {

  let mut temp: [u32;16]=[0;16];
  unsafe { multiply256x256_asm(&mut temp, in1, in2);

  fe25519_reduceTo256Bits_asm(  &mut in2,  &mut temp);
  }

}




fn fe25519_sqr (out:&mut[u32;8],in1:&mut[u32;8]) {

  let mut temp: [u32;16]=[0;16];
  unsafe { square256_asm(&mut temp, in1);

  fe25519_reduceTo256Bits_asm(  out,  &mut temp);
  }

}


//not used
/*
fn fe25519_sqri (out:&mut[u32;8]) {

  let mut temp: [u32;16]=[0;16];
  unsafe { square256_asm(&mut temp, out);

  fe25519_reduceTo256Bits_asm(  out,  &mut temp);
  }

}
*/

fn fe25519_reducecomp (inout:&mut[u32;8]) {

  let  numoftimestosubprime:u32;
  let  initial:u32 =inout[7]>>31;

  let mut accu:u64;


  accu = (initial as u64) * 19 +19;
  
  for ctr in 0..7 {
    accu+=inout[ctr] as u64;
    accu>>=32;
  }
  accu += inout[7] as u64;
  numoftimestosubprime = (accu >> 31) as u32;
  accu=(numoftimestosubprime as u64 )*19;

  for ctr in 0..7 {
    accu+=inout[ctr] as u64;
    inout[ctr]= accu as u32;
    accu>>=32;
  }

  accu += inout[7] as u64;
  inout[7] = (accu & 2147483647) as u32;

}


//dunno if this actually wrks right

fn fe25519_pack(in1:&mut[u32;8],out:&mut[u8;32]){


  fe25519_reducecomp (in1);
  for ctr in 0..8 {
    out[ctr*4]=(in1[ctr]&255) as u8;
    out[ctr*4+1]=((in1[ctr]&(255<<8))>>8) as u8;
    out[ctr*4+2]=((in1[ctr]&(255<<16))>>16) as u8;
    out[ctr*4+3]=((in1[ctr]&(255<<24))>>24) as u8;

  }

}


fn fe25519_invert(r:&mut[u32;8],x:&mut[u32;8],t0:&mut[u32;8],t1:&mut[u32;8],t2:&mut[u32;8]){


      {
 
        /* 2 */ fe25519_sqr(t2, x);

        /* 4 */ fe25519_sqr(t0, t2);

let mut t0_copy:[u32;8]=[0;8];
        for iter in 0..8{
          t0_copy[iter]=t0[iter];
        }
        /* 8 */ fe25519_sqr(t0, &mut t0_copy);

        /* 9 */ fe25519_mul(t1, t0, x);

        /* 11 */ fe25519_mul(r, t1,t2);

    }



        /* 22 */ fe25519_sqr(t0, r);

    let mut t1_copy:[u32;8]=[0;8];
        for iter in 0..8{
          t1_copy[iter]=t1[iter];
        }
    /* 2^5 - 2^0 = 31 */ fe25519_mul(t1, t0, &mut t1_copy);

    /* 2^6 - 2^1 */ fe25519_sqr(t0, t1);

    let mut t0_copy:[u32;8]=[0;8];
    for iter in 0..8{
          t0_copy[iter]=t0[iter];
        }
    /* 2^7 - 2^2 */ fe25519_sqr(t0, &mut t0_copy);

    /* 2^8 - 2^3 */ fe25519_sqr(&mut t0_copy, t0);

    /* 2^9 - 2^4 */ fe25519_sqr(t0, &mut t0_copy);

    /* 2^10 - 2^5 */ fe25519_sqr(&mut t0_copy, t0);
 
    /* 2^10 - 2^0 */ fe25519_mul(&mut t1_copy, &mut t0_copy, t1);
 
    /* 2^11 - 2^1 */ fe25519_sqr(t0, &mut t1_copy);

//I hope this works????? the update values are now t0 and t1_copy

    for iter in 1..5{
      fe25519_sqr(&mut t0_copy, t0);
      fe25519_sqr(t0,&mut t0_copy);

    }

    fe25519_sqr(&mut t0_copy, t0);




    /* 2^20 - 2^0 */ fe25519_mul(t2, &mut t0_copy, &mut t1_copy);

    /* 2^21 - 2^1 */ fe25519_sqr(t0, t2);


    for iter in 1..10{
      fe25519_sqr(&mut t0_copy, t0);
      fe25519_sqr(t0,&mut t0_copy);

    }
    fe25519_sqr(&mut t0_copy, t0);


    /* 2^40 - 2^0 */ fe25519_mul(t0, &mut t0_copy, t2);

    /* 2^41 - 2^1 */ fe25519_sqr( &mut t0_copy,t0);

    for iter in 1..5{
      fe25519_sqr(t0,&mut t0_copy);
      fe25519_sqr(&mut t0_copy, t0);
      

    }
    fe25519_sqr(t0,&mut t0_copy);
////////////////////////

    /* 2^50 - 2^0 */ fe25519_mul(t2, t0, &mut t1_copy);

    /* 2^51 - 2^1 */ fe25519_sqr(t0, t2);

    /* 2^100 - 2^50 */ for iter in 1..25{
    
        fe25519_sqr(&mut t0_copy, t0);
        fe25519_sqr(t0, &mut t0_copy);
    }
    fe25519_sqr(&mut t0_copy, t0);

    /* 2^100 - 2^0 */ fe25519_mul(t1, &mut t0_copy, t2);

    /* 2^101 - 2^1 */ fe25519_sqr(t0,t1);
    //now originals have the expected value

    /* 2^200 - 2^100 */ for iter in 1..50{
    
        fe25519_sqr(&mut t0_copy, t0);
        fe25519_sqr(t0, &mut t0_copy);
    }
    fe25519_sqr(&mut t0_copy, t0);


    /* 2^200 - 2^0 */ fe25519_mul(t0, &mut t0_copy, t1);

    /* 2^250 - 2^50 */ for iter in 0..25{
    
        fe25519_sqr(&mut t0_copy, t0);
        fe25519_sqr(t0, &mut t0_copy);
    }
    /* 2^250 - 2^0 */ fe25519_mul(&mut t0_copy, t0, t2);

    /* 2^255 - 2^5 */for iter in 1..3{
    
         fe25519_sqr(t0, &mut t0_copy);
         fe25519_sqr(&mut t0_copy, t0);
    }
    fe25519_sqr(t0, &mut t0_copy);

    let mut r_copy:[u32;8]=[0;8];
        for iter in 0..8{
          r_copy[iter]=r[iter];
        }
    /* 2^255 - 21 */ fe25519_mul(r, t0, &mut r_copy);


}

///////////////////////////////////////////////////////////////////////////////////////////////////








////////////////////////////////////////////////////////////////////////////////////////////////








fn fe25519_setzero(out:&mut[u32;8]){
  for iter in 0..8{
          out[iter]=0;
  }
}



fn fe25519_setone(out:&mut[u32;8]){

  out[0]=1;
  for iter in 1..8{
          out[iter]=0;
  }
}


//techically never used because we use a modified one

/*
fn fe25519_cswap(in1:&mut[u32;8],in2:&mut[u32;8],condition:i32){

  let mut mask:i32 = condition;
  mask=-mask;
  
  for ctr in 0..8{
    let mut val1:u32 = in1[ctr];
    let mut val2:u32 = in2[ctr];
    let temp:u32 = val1;

    val1 ^= (mask as u32) & (val2 ^ val1);
    val2 ^= (mask as u32) & (val2 ^ temp);

    in1[ctr]=val1;
    in2[ctr]=val2;

  }



}
*/

fn fe25519_cswapu8(in1:&mut[u32;8],in2:&mut[u32;8],condition:u8){

  let mut mask:i32 = condition as i32;
  mask=-mask;

  
  for ctr in 0..8{
    let mut val1:u32 = in1[ctr];
    let mut val2:u32 = in2[ctr];
    let temp:u32 = val1;

    val1 ^= (mask as u32) & (val2 ^ val1);
    val2 ^= (mask as u32) & (val2 ^ temp);

    in1[ctr]=val1;
    in2[ctr]=val2;

  }



}

struct Ladder {
  x0: [u32;8],
  xp: [u32;8],
  zp: [u32;8],
  xq: [u32;8],
  zq: [u32;8],
  s : [u32;8],
  next:i32,
  prev:u8,

}

fn curve25519_ladderstep( pstate:&mut Ladder){


  let mut t1: [u32;8]=[0;8];
  let mut t2: [u32;8]=[0;8];


    fe25519_add(&mut t1,&mut pstate.xp,&mut pstate.zp); // A = X2+Z2

    fe25519_sub(&mut t2,&mut pstate.xp,&mut pstate.zp); // B = X2-Z2

    fe25519_add(&mut pstate.xp,&mut pstate.xq,&mut pstate.zq); // C = X3+Z3

    fe25519_sub_test(&mut pstate.zp,&mut pstate.xq,&mut pstate.zq); // D = X3-Z3
 
    
    fe25519_mul_test(&mut pstate.xq,&mut pstate.zp,&mut t1); // DA= D*A

    fe25519_mul(&mut pstate.zp,&mut pstate.xp,&mut t2); // CB= C*B

    fe25519_add(&mut pstate.xp,&mut pstate.zp,&mut pstate.xq); // T0= DA+CB

    fe25519_sub(&mut pstate.zq,&mut pstate.xq,&mut pstate.zp); // &mut t2= DA-CB

    fe25519_sqr(&mut pstate.xq,&mut pstate.xp); // X5==&mut t1= T0^2

    fe25519_sqr(&mut pstate.xp,&mut pstate.zq); // T3= &mut t2^2

    fe25519_mul(&mut pstate.zq,&mut pstate.xp,&mut pstate.x0); // Z5=X1*t3

    fe25519_sqr(&mut pstate.xp,&mut t1); // AA=A^2

    fe25519_sqr(&mut t1,&mut t2); // BB=B^2

    fe25519_sub(&mut pstate.zp,&mut pstate.xp,&mut t1); // E=AA-BB

    fe25519_mulac(&mut t1,&mut pstate.xp); // X4= AA*BB

    unsafe {fe25519_mpyWith121666_asm (&mut t2,&mut pstate.zp);}// T4 = a24*E



    fe25519_addab(&mut t2,&mut t1); // T5 = BB + t4


    fe25519_mulac(&mut t2,&mut pstate.zp); // Z4 = E*t5

}

fn curve25519_cswap( state:&mut Ladder, b:u8){

  fe25519_cswapu8 (&mut state.xp, &mut state.xq,b );
  fe25519_cswapu8 (&mut state.zp, &mut state.zq,b);

}

fn curve25519_cswapi( state:&mut Ladder){

  fe25519_cswapu8 (&mut state.xp, &mut state.xq,state.prev);
  fe25519_cswapu8 (&mut state.zp, &mut state.zq,state.prev);

}




fn fe25519_getu8(in1:&mut[u32;8], iter:u32)-> u32 {
  let iterx:u32=iter/4;
  let itery:u32=iter-iterx*4;
  let test:u32=(in1[iterx as usize]<<(8*(3-itery)))>>24;


  return test;
}

pub fn crypto_scalarmult_curve25519(r:&mut [u8;32],s:& [u8;32],p:& [u8;32]){

  let mut  state=Ladder{x0:[0;8],xp:[0;8],zp:[0;8],xq:[0;8],zq:[0;8],s:[0;8], next:0, prev:0 };


  fe25519_unpack(&mut state.s,s);

  state.s[0]&=4294967288;
  state.s[7] &= 4294967167;
  state.s[7] |= 1073741824;

  fe25519_unpack (&mut state.x0, p);


    
  fe25519_setone (&mut state.zq);
  
    
  fe25519_cpy (&mut state.xq, &mut state.x0);

 
 fe25519_setone(&mut state.xp);


    fe25519_setzero(&mut state.zp);

    state.next = 254;
    state.prev = 0;


let mut counter:u32=0;
//ice
   while state.next >= 0
  {

      let byteno:u8 = (state.next>> 3) as u8;
    	let bitno:u8 = (state.next & 7) as u8;
      let  bit:u8;
      let swap:u8;
      bit=1 & ( (fe25519_getu8(&mut state.s, (byteno as u32) ) >> bitno)as u8);
      swap = bit ^ state.prev;
     

      state.prev = bit;

      curve25519_cswap(&mut state, swap);

      curve25519_ladderstep(&mut state);
      state.next=state.next-1;

      counter=counter+1;

  }

 
  /////////////////////////////////
  curve25519_cswapi(&mut state);

  let mut zp_copy:[u32;8]=[0;8];
  for ctr in 0..8{
    zp_copy[ctr]=state.zp[ctr];
  }
  fe25519_invert (&mut state.zp,&mut zp_copy, &mut state.xq, &mut state.zq, &mut state.x0);    

  let mut xp_copy:[u32;8]=[0;8];
  for iter in 0..8{
    xp_copy[iter]=state.xp[iter];
  }

  fe25519_mul(&mut state.xp, &mut xp_copy, &mut state.zp);

  fe25519_reducecomp (&mut state.xp);

  fe25519_pack  (&mut state.xp,r);

}


pub fn crypto_scalarmult_curve25519_base(q:&mut [u8;32],n:& [u8;32]){

  let base:[u8;32]=[ 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  crypto_scalarmult_curve25519(q, n, & base);
}




pub fn test1() -> i32{


    let alice:[u8;32]=[119, 7, 109, 10, 115, 24, 165, 125, 60, 22, 193, 114, 81, 178, 102, 69, 223, 76, 47, 135, 235, 192, 153, 42, 177, 119, 251, 165, 29, 185, 44, 42];
  let bob:[u8;32]=[222, 158, 219, 125, 123, 125, 193, 180, 211, 91, 97, 194, 236, 228, 53, 55, 63, 131, 67, 200, 91, 120, 103, 77, 173, 252, 126, 20, 111, 136, 43, 79];
  //let alice1:[u8;32]=[0,0, 109, 10, 115, 24, 165, 125, 60, 22, 193, 114, 81, 178, 102, 69, 223, 76, 47, 135, 235, 192, 153, 42, 177, 119, 251, 165, 29, 185, 44, 42];
  //let bob1:[u8;32]=[0,0, 219, 125, 123, 125, 193, 180, 211, 91, 97, 194, 236, 228, 53, 55, 63, 131, 67, 200, 91, 120, 103, 77, 173, 252, 126, 20, 111, 136, 43, 79];
  //let alice2:[u8;32]=[248,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,127];
  //let bob2:[u8;32]=[8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,64];

  //let blank1:[u8;32]=[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
   //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  //let mut result:[u8;32]=[0;32];
  let mut result1:[u8;32]=[0;32];
  let mut result2:[u8;32]=[0;32];
  let mut resulta:[u8;32]=[0;32];
  let mut resultb:[u8;32]=[0;32];

  crypto_scalarmult_curve25519_base(&mut result1,& alice);
  crypto_scalarmult_curve25519_base(&mut result2,& bob);
  crypto_scalarmult_curve25519(&mut resulta, & alice, & result2);
  crypto_scalarmult_curve25519(&mut resultb, & &bob, & result1);
  //let mut output:i32=0;
  
  for x in 0 ..32 {
    if (resulta[x] !=resultb[x]){
    
        return 1;
    }   
   }
    return 0;
}








pub fn test(output:&mut[u32;16]) {

    //let mut outab:[u32;16]=[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let mut ina:[u32;8]=[3,0,0,0,0,0,0,2];
    let mut inb:[u32;8]=[4,0,0,0,0,0,0,5];
    unsafe {multiply256x256_asm( output, &mut ina, &mut inb); }
    //return outab;
    //println!("we got {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",outab[0], outab[1],outab[2],outab[3], outab[4],outab[5],outab[6], outab[7],outab[8],outab[9], outab[10],outab[11],outab[12], outab[13],outab[14],outab[15]);



}
