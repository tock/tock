# Tock Core Notes 08/28/2020

## Attending
 * Branden Ghena
 * Leon Schuermann
 * Phil Levis
 * Pat Pannuto
 * Brad Campbell
 * Johnathan Van Why
 * Vadim Sukhomlinov
 * Hudson Ayers

## Updates
 * Phil: Testing timers on OpenTitan. 32-bit interface hasn't been tested on 64-bit timer. That still needs to be done. Once that's finished, I'll submit the PR.
 * Brad: Refining RISC-V development to make that libtock-c continues to work. Slowly the platform is becoming more reliable and usable.
     * Phil: Are there instructions on compiling libtock-c for RISC-V?
     * Brad: Yes. Basically just one flag. It's in the README.
 * Hudson: Implemented new peripheral approach to nRF52 chips and boards. Seemed to work for everything. Still would like for the initial PR to get reviewed and merged before I do the work for the others. Want to know first if approach needs to change. This change is applicable to individual boards without breaking the others, so that's good.
 * Phil: Now that timer system is almost done, I think we're going to start implementation efforts on Tock 2.0. Starting with the design doc.
     * Leon: I'm happy to help with design doc.
 
 ## SPI Pull Request
 * Hudson: Alistair's PR for splitting up SPI peripheral notation. Stop using SPI slave, move to SPI peripheral and controller. I pointed out OpenTitan uses Device and Host. I think we should just decide on one.
 * Phil: I think we're not going to change OpenTitan, so we should just go with their notation.
 * Brad: I've been seeing peripheral/controller notation around. I'd rather be consistent with them as it seems to be the way things are going. I'm not sure there's one sure convergence yet.
 * Brad: Oh, this the capsule and syscall interface.
 * Hudson: Not the HIL, but if we're updating names, we should update HIL too. I'd say when we merge this, we're committing and should follow up with naming changes for HIL and lower too.
 * Pat: The Controller/Peripheral notion is becoming supported by hacker community, with Sparkfun and Hackaday.
 * Johnathan: Google's docs have a number of possible replacements and provide no guidance here.
 * Phil: Okay, let's just go with controller/peripheral then.
