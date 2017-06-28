Test yield_for_timeout
====================

This tests the yield_for_timeout function. It sets a one shot alarm with a
timeout longer than than that alarm, then sets a one shot alarm with a timeout
shorter than that alarm. An LED is set and cleared based on the result
of the yield_for_timeout function. The user should see the LED stay off for
500ms, then turn on for 1s.
