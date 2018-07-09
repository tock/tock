Since 1.2
=========

* [#1044](https://github.com/tock/tock/pull/1044) creates a `Kernel` struct
  with a method for the kernel's main loop, instead of a global function in
  the kernel's base module. Board configurations (i.e. each board's
  `main.rs`), as a result needs to instantiate a statically allocate this new
  struct.  Arguments to the main loop haven't changed:

  ```rust
  let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new());

  board_kernel.kernel_loop(&hail, &mut chip, &mut PROCESSES, Some(&hail.ipc));
  ```


* [#1032](https://github.com/tock/tock/pull/1032) updates the ADC HIL to
  explicitly specify the resolution of the sample and to clarify that samples
  are always left-aligned in the `u16` buffer. Previously, the only ADC
  implementation happened to be 12 bits and left-aligned, which callers only
  assumed. It also added a method to (if possible) report the reference
  voltage, which can be used to convert raw ADC samples to absolute voltages.
  Implementers of the ADC HIL must implement two new methods:

  ```rust
  /// Function to ask the ADC how many bits of resolution are in the samples
  /// it is returning.
  fn get_resolution_bits(&self) -> usize;

  /// Function to ask the ADC what reference voltage it used when taking the
  /// samples. This allows the user of this interface to calculate an actual
  /// voltage from the ADC reading.
  ///
  /// The returned reference voltage is in millivolts, or `None` if unknown.
  fn get_voltage_reference_mv(&self) -> Option<usize>;
  ```