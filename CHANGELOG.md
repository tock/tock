Since 1.2
=========

* UART HIL Refinements: This release saw several updates to the UART HIL,
  summarized in the [UART HIL tracking issue](https://github.com/tock/tock/issues/1072).

  - [#1073](https://github.com/tock/tock/pull/1073) removes `initialize` from
    the UART HIL. Implementations will need to disentangle board-specific
    initialization code, such as enabling the peripheral or assigning pins,
    from UART configuration code, such as baud rate or parity. Initialization
    is no longer part of the UART HIL and should be performed by the top-level
    board before passing the UART object to any other code. UART configuration
    is now controlled by the new `configure` HIL method:

    ```rust
    /// Configure UART
    ///
    /// Returns SUCCESS, or
    ///
    /// - EOFF: The underlying hardware is currently not available, perhaps
    ///         because it has not been initialized or in the case of a shared
    ///         hardware USART controller because it is set up for SPI.
    /// - EINVAL: Impossible parameters (e.g. a `baud_rate` of 0)
    /// - ENOSUPPORT: The underlying UART cannot satisfy this configuration.
    fn configure(&self, params: UARTParameters) -> ReturnCode;
    ```
