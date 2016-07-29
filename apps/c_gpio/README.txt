This application is for testing GPIO interrupts in the nRF51822 EK.

To run this application, hook up a button connected to VDD to GPIO pin 1
(the top right pin on the top left header).

When it boots, you should see one of the two LEDs blink 5 times, then
go silent. This is to show that the app has booted correctly.

Then, when you push the button, the other LED should blink.


